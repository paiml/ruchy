//! Function transpilation helpers
//! EXTREME TDD Round 81: Extracted from statements.rs
//!
//! This module handles function definition transpilation.

use crate::frontend::ast::{Expr, ExprKind, Param, Type, TypeKind};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

use super::Transpiler;

impl Transpiler {
    /// Transpile function definition
    /// EXTREME TDD Round 81: Extracted from statements.rs
    pub fn transpile_function_impl(
        &self,
        name: &str,
        type_params: &[String],
        params: &[Param],
        body: &Expr,
        is_async: bool,
        return_type: Option<&Type>,
        is_pub: bool,
        attributes: &[crate::frontend::ast::Attribute],
    ) -> Result<TokenStream> {
        // RAWIDENT-1: a Rust-reserved-but-not-Ruchy-keyword name (e.g. `do`, `box`,
        // `typeof`) must be emitted as a raw identifier so rustc accepts the fn.
        let fn_name = Transpiler::safe_ident(name);

        // Check if we need to add lifetime parameter
        let needs_lifetime = super::type_analysis::needs_lifetime_parameter(params, return_type);

        // If lifetime needed, add 'a to type params and modify param/return types
        // DEFECT-028 FIX: Check if type_params already contains a lifetime to avoid duplicates
        let has_existing_lifetime = type_params.iter().any(|p| p.starts_with('\''));
        let mut modified_type_params = type_params.to_vec();
        if needs_lifetime && !has_existing_lifetime {
            modified_type_params.insert(0, "'a".to_string());
        }

        // PRINTSTR-1: string parameters print with Display.
        self.track_string_params(params);
        // NESTPUSHLIT-1: nested-vec records of earlier functions end here.
        self.inner_vec_lists.borrow_mut().clear();

        // TRANSPILER-004 FIX: Track String-typed parameters for proper concat transpilation
        // Before processing function body, register all String parameters in string_vars
        // This enables is_definitely_string() to detect them for `a + b` → `format!()` or `a + &b`
        for param in params {
            if let TypeKind::Named(type_name) = &param.ty.kind {
                if type_name == "String" {
                    self.string_vars.borrow_mut().insert(param.name().clone());
                }
            }
        }

        // DEFECT-024 FIX: Track Option/Result-typed parameters for proper .map() transpilation
        // This enables is_option_or_result_with_context() to detect Option/Result variables
        for param in params {
            let type_str = Transpiler::type_to_string(&param.ty);
            if type_str.starts_with("Option") || type_str.starts_with("Result") {
                self.register_variable_type(&param.name(), &type_str);
            }
        }

        let param_tokens = if needs_lifetime {
            self.generate_param_tokens_with_lifetime(params, body, name)?
        } else {
            self.generate_param_tokens(params, body, name)?
        };
        // PRINTPARAM-1: an inferred String/&str parameter prints with Display.
        self.track_emitted_string_params(&param_tokens);

        // Check for #[test] attribute and override return type if found
        let has_test_attribute = attributes.iter().any(|attr| attr.name == "test");
        let effective_return_type = if has_test_attribute {
            None // Test functions should have unit return type
        } else {
            return_type
        };

        // TRANSPILER-007: Set current function return type for empty vec type inference
        self.current_function_return_type
            .replace(effective_return_type.cloned());

        // DEFECT-012 FIX: Generate body tokens with special handling for String return type
        let body_tokens = if let Some(ret_type) = effective_return_type {
            if super::type_analysis::is_string_type(ret_type)
                && super::type_analysis::body_needs_string_conversion(body)
            {
                self.generate_body_tokens_with_string_conversion(body, is_async)?
            } else {
                self.generate_body_tokens(body, is_async)?
            }
        } else if name == "main" && self.has_non_unit_last_expr(body) {
            // BOOK-COMPAT-015: Main functions that end with a non-unit expression
            // should print it instead of trying to return it
            self.generate_main_body_with_print(body, is_async)?
        } else {
            self.generate_body_tokens(body, is_async)?
        };

        // TRANSPILER-007: Clear current function return type after body transpilation
        self.current_function_return_type.replace(None);

        // RHLGA-1: rustc rejects `async fn main` (E0752) and `.await` in a sync fn (E0728).
        let (body_tokens, is_async) =
            self.adapt_awaiting_main(name, body, body_tokens, is_async, effective_return_type)?;

        let return_type_tokens = if needs_lifetime {
            self.generate_return_type_tokens_with_lifetime(name, effective_return_type, body)?
        } else {
            self.generate_return_type_tokens(name, effective_return_type, body, params)?
        };

        let type_param_tokens = self.generate_type_param_tokens(&modified_type_params)?;
        self.generate_function_signature(
            is_pub,
            is_async,
            &fn_name,
            &type_param_tokens,
            &param_tokens,
            &return_type_tokens,
            &body_tokens,
            attributes,
        )
    }

    /// BOOK-COMPAT-015: Check if body has a non-unit last expression
    /// (MAINUNIT-1: a call to a unit-returning user function is unit;
    /// CLOSUREUNIT-1: so is a call to a let-bound closure with a unit body)
    fn has_non_unit_last_expr(&self, body: &Expr) -> bool {
        self.has_non_unit_tail(body) && !tail_calls_unit_closure(body)
    }

    /// BOOK-COMPAT-015: the last expression of `body` is not unit.
    fn has_non_unit_tail(&self, body: &Expr) -> bool {
        match &body.kind {
            ExprKind::Block(exprs) => exprs
                .last()
                .is_some_and(|last| self.has_non_unit_tail(last)),
            ExprKind::Let { body, .. } | ExprKind::LetPattern { body, .. } => {
                self.has_non_unit_tail(body)
            }
            _ => !Self::is_unit_expr(body) && !self.is_unit_function_call(body),
        }
    }

    /// MAINUNIT-1: a call to a user function recorded as returning `()`.
    /// (complexity: 2)
    fn is_unit_function_call(&self, expr: &Expr) -> bool {
        matches!(
            &expr.kind,
            ExprKind::Call { func, .. }
                if matches!(&func.kind, ExprKind::Identifier(name) if self.unit_functions.contains(name))
        )
    }

    /// Check if expression is a unit expression (doesn't return a value)
    fn is_unit_expr(expr: &Expr) -> bool {
        use crate::frontend::ast::Literal;
        // RUCHYRUCHY-002: Add Literal(Unit) check for let statement bodies
        matches!(&expr.kind, ExprKind::Literal(Literal::Unit))
            || matches!(
                &expr.kind,
                ExprKind::Call { func, .. } if matches!(&func.kind, ExprKind::Identifier(name) if Self::is_print_name(name))
            )
            || matches!(
                &expr.kind,
                ExprKind::MacroInvocation { name, .. } if Self::is_print_name(name)
            )
            || matches!(&expr.kind, ExprKind::Assign { .. })
            || matches!(&expr.kind, ExprKind::Return { value: None })
            || matches!(&expr.kind, ExprKind::Return { value: Some(_) }) // Return with value is still unit for main
    }

    /// PRINTLNFMT-1: the print builtins, all unit-valued (stdout and stderr).
    fn is_print_name(name: &str) -> bool {
        matches!(name, "println" | "print" | "eprintln" | "eprint")
    }

    /// BOOK-COMPAT-015: Generate main body that prints the last expression
    fn generate_main_body_with_print(&self, body: &Expr, is_async: bool) -> Result<TokenStream> {
        match &body.kind {
            ExprKind::Block(exprs) if !exprs.is_empty() => {
                let mut tokens = Vec::new();
                // Transpile all but last expression normally with proper semicolons
                // RUCHYRUCHY-003: Add semicolons to non-final statements
                for expr in exprs.iter().take(exprs.len() - 1) {
                    let expr_tokens = self.transpile_expr(expr)?;
                    let is_let = matches!(
                        &expr.kind,
                        ExprKind::Let { .. } | ExprKind::LetPattern { .. }
                    );
                    if is_let {
                        tokens.push(expr_tokens);
                    } else {
                        tokens.push(quote! { #expr_tokens; });
                    }
                }
                // Wrap last expression in println!
                if let Some(last) = exprs.last() {
                    let last_tokens = self.transpile_expr(last)?;
                    tokens.push(quote! { println!("{:?}", #last_tokens); });
                }
                if is_async {
                    Ok(quote! { async { #(#tokens)* } })
                } else {
                    Ok(quote! { #(#tokens)* })
                }
            }
            _ => {
                // Single expression body - wrap in println!
                let body_tokens = self.transpile_expr(body)?;
                if is_async {
                    Ok(quote! { async { println!("{:?}", #body_tokens); } })
                } else {
                    Ok(quote! { println!("{:?}", #body_tokens); })
                }
            }
        }
    }
}

impl Transpiler {
    /// A `main` that awaits gets a `block_on` body and a sync signature; others pass through.
    /// Complexity: 2
    fn adapt_awaiting_main(
        &self,
        name: &str,
        body: &Expr,
        body_tokens: TokenStream,
        is_async: bool,
        return_type: Option<&Type>,
    ) -> Result<(TokenStream, bool)> {
        if name == "main" && (is_async || Self::tokens_contain_await(&body_tokens)) {
            Ok((self.generate_block_on_main_body(body, return_type)?, false))
        } else {
            Ok((body_tokens, is_async))
        }
    }

    /// True when the emitted Rust uses the `await` keyword (only legal in an async context).
    /// Complexity: 3
    fn tokens_contain_await(tokens: &TokenStream) -> bool {
        tokens.clone().into_iter().any(|tt| match tt {
            proc_macro2::TokenTree::Ident(ident) => ident == "await",
            proc_macro2::TokenTree::Group(group) => Self::tokens_contain_await(&group.stream()),
            _ => false,
        })
    }

    /// Body of a `main` that awaits: the original body runs as an `async move` block driven
    /// by a thread-parking `block_on` (std only, no unsafe); a non-unit tail is printed.
    /// Complexity: 2
    fn generate_block_on_main_body(
        &self,
        body: &Expr,
        return_type: Option<&Type>,
    ) -> Result<TokenStream> {
        let inner = self.generate_body_tokens(body, true)?;
        let tail = match self.declared_non_unit_type(return_type)? {
            // RHLGA-1: a declared return type is main's value; the typed binding pins the
            // async block's output so `?` inside it resolves
            Some(ret) => quote! {
                let __ruchy_main_value: #ret = __ruchy_block_on(async move { #inner });
                __ruchy_main_value
            },
            None => {
                let run = quote! { __ruchy_block_on(async move { #inner }) };
                if self.has_non_unit_last_expr(body) {
                    quote! { let __ruchy_main_value = #run; println!("{:?}", __ruchy_main_value); }
                } else {
                    quote! { #run; }
                }
            }
        };
        let block_on = Self::block_on_fn_tokens();
        Ok(quote! {
            {
                #block_on
                #tail
            }
        })
    }

    /// Tokens of a declared return type other than `()`.
    fn declared_non_unit_type(&self, return_type: Option<&Type>) -> Result<Option<TokenStream>> {
        let Some(ty) = return_type else {
            return Ok(None);
        };
        let tokens = self.transpile_type(ty)?;
        Ok((tokens.to_string().replace(' ', "") != "()").then_some(tokens))
    }

    /// A thread-parking `block_on` (std only, no unsafe) for the awaiting `main`.
    fn block_on_fn_tokens() -> TokenStream {
        quote! {
                fn __ruchy_block_on<F: ::std::future::Future>(fut: F) -> F::Output {
                    struct ThreadWaker(::std::thread::Thread);
                    impl ::std::task::Wake for ThreadWaker {
                        fn wake(self: ::std::sync::Arc<Self>) {
                            self.0.unpark();
                        }
                    }
                    let waker = ::std::task::Waker::from(::std::sync::Arc::new(ThreadWaker(
                        ::std::thread::current(),
                    )));
                    let mut cx = ::std::task::Context::from_waker(&waker);
                    let mut fut = ::std::pin::pin!(fut);
                    loop {
                        if let ::std::task::Poll::Ready(value) = fut.as_mut().poll(&mut cx) {
                            return value;
                        }
                        ::std::thread::park();
                    }
                }
        }
    }
}

/// CLOSUREUNIT-1: the last expression of `body` calls a closure bound by a
/// `let` earlier in `body` whose body is unit (the latest binding of the
/// name wins, so a shadowing value closure is still printed).
fn tail_calls_unit_closure(body: &Expr) -> bool {
    let mut closures = std::collections::HashMap::new();
    tail_with_closures(body, &mut closures)
}

/// CLOSUREUNIT-1: walk the statement chain of `expr` to its tail, recording
/// in `closures` whether each let-bound name is a unit-bodied closure.
fn tail_with_closures<'a>(
    expr: &'a Expr,
    closures: &mut std::collections::HashMap<&'a str, bool>,
) -> bool {
    match &expr.kind {
        ExprKind::Block(exprs) => {
            let Some((last, init)) = exprs.split_last() else {
                return false;
            };
            init.iter()
                .for_each(|e| record_closure_binding(e, closures));
            tail_with_closures(last, closures)
        }
        ExprKind::Let { body, .. } => {
            record_closure_binding(expr, closures);
            tail_with_closures(body, closures)
        }
        ExprKind::Call { func, .. } => matches!(
            &func.kind,
            ExprKind::Identifier(name) if closures.get(name.as_str()) == Some(&true)
        ),
        _ => false,
    }
}

/// CLOSUREUNIT-1: record whether the `let` `expr` binds a unit-bodied closure.
fn record_closure_binding<'a>(
    expr: &'a Expr,
    closures: &mut std::collections::HashMap<&'a str, bool>,
) {
    if let ExprKind::Let { name, value, .. } = &expr.kind {
        let is_unit_closure = matches!(
            &value.kind,
            ExprKind::Lambda { body, .. } if super::function_analysis::is_void_expression(body)
        );
        closures.insert(name.as_str(), is_unit_closure);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    fn create_transpiler() -> Transpiler {
        Transpiler::new()
    }

    /// RHLGA-1: the emitted `block_on` parks until woken. The future is Pending on its
    /// first poll and is woken from another thread, so the park/wake path runs.
    #[test]
    fn test_rhlga1_block_on_parks_until_a_pending_future_is_woken() {
        let block_on = Transpiler::block_on_fn_tokens();
        let program = format!(
            "{block_on}\n{}",
            r#"
struct WakeLater { polls: u32 }
impl std::future::Future for WakeLater {
    type Output = u32;
    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<u32> {
        self.polls += 1;
        if self.polls > 1 {
            return std::task::Poll::Ready(self.polls);
        }
        let waker = cx.waker().clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(20));
            waker.wake();
        });
        std::task::Poll::Pending
    }
}
fn main() {
    println!("{}", __ruchy_block_on(WakeLater { polls: 0 }));
}
"#
        );
        let dir = std::env::temp_dir().join(format!("rhlga1_block_on_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("scratch dir");
        let (src, bin) = (dir.join("main.rs"), dir.join("main"));
        std::fs::write(&src, program).expect("write rust");
        let out = std::process::Command::new("rustc")
            .args(["--edition", "2021", "-A", "warnings", "-o"])
            .arg(&bin)
            .arg(&src)
            .output()
            .expect("rustc runs");
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let run = std::process::Command::new(&bin).output().expect("runs");
        assert_eq!(String::from_utf8_lossy(&run.stdout), "2\n");
    }

    #[test]
    fn test_transpile_simple_function() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"fun add(a: i32, b: i32) -> i32 { a + b }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("fn add"));
    }

    #[test]
    fn test_transpile_async_function() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"async fun fetch() { 42 }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("async"));
    }

    #[test]
    fn test_transpile_pub_function() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"pub fun greet() { "hello" }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("pub"));
    }

    #[test]
    fn test_transpile_test_function() {
        let mut transpiler = create_transpiler();
        // Ruchy uses @test decorator syntax
        let mut parser = Parser::new(r#"@test fun test_foo() { assert(true) }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        // Verify function is generated
        assert!(tokens.contains("test_foo"));
    }

    #[test]
    fn test_transpile_generic_function() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"fun identity<T>(x: T) -> T { x }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("<T>") || tokens.contains("< T >"));
    }

    // ===== EXTREME TDD Round 156 - Function Transpilation Tests =====

    #[test]
    fn test_transpile_function_multiple_params() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"fun calc(a: i32, b: i32, c: i32) -> i32 { a + b + c }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("fn calc"));
    }

    #[test]
    fn test_transpile_function_string_return() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"fun greet(name: String) -> String { "Hello, " + name }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_function_no_return_type() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"fun do_nothing() { }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_function_with_body_block() {
        let mut transpiler = create_transpiler();
        let code = r#"fun complex(x: i32) -> i32 {
            let y = x * 2
            let z = y + 1
            z
        }"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_async_function_with_return() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"async fun fetch_data() -> i32 { 42 }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("async"));
    }

    #[test]
    fn test_transpile_function_with_reference_param() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"fun process(data: &str) -> i32 { 0 }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_function_with_mutable_param() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"fun modify(mut x: i32) { x = x + 1 }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_function_returning_option() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"fun find(x: i32) -> Option<i32> { Some(x) }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_function_returning_result() {
        let mut transpiler = create_transpiler();
        let mut parser =
            Parser::new(r#"fun try_parse(s: String) -> Result<i32, String> { Ok(42) }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_function_with_vec_param() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"fun sum_all(nums: Vec<i32>) -> i32 { 0 }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_function_multiple_generics() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"fun pair<T, U>(a: T, b: U) -> (T, U) { (a, b) }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_function_with_where_clause() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"fun display<T: Display>(x: T) { println(x) }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_recursive_function() {
        let mut transpiler = create_transpiler();
        let code = r#"fun factorial(n: i32) -> i32 {
            if n <= 1 { 1 } else { n * factorial(n - 1) }
        }"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_function_with_default_value() {
        let mut transpiler = create_transpiler();
        let mut parser =
            Parser::new(r#"fun greet(name: String = "World") { println("Hello, " + name) }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_function_pub_async() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"pub async fun api_call() -> String { "response" }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("pub"));
        assert!(tokens.contains("async"));
    }

    #[test]
    fn test_transpile_function_returning_tuple() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"fun split(x: i32) -> (i32, i32) { (x / 2, x % 2) }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_function_with_bool_return() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"fun is_even(n: i32) -> bool { n % 2 == 0 }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_function_empty_body() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"fun noop() { () }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_function_with_float_param() {
        let mut transpiler = create_transpiler();
        let mut parser = Parser::new(r#"fun square(x: f64) -> f64 { x * x }"#);
        let ast = parser.parse().expect("parse");
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }
}
