//! Function transpilation helpers
//! EXTREME TDD Round 81: Extracted from statements.rs
//!
//! This module handles function definition transpilation.

use crate::frontend::ast::{Expr, ExprKind, Param, Type, TypeKind};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

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
        let fn_name = format_ident!("{}", name);

        // Check if we need to add lifetime parameter
        let needs_lifetime = super::type_analysis::needs_lifetime_parameter(params, return_type);

        // If lifetime needed, add 'a to type params and modify param/return types
        // DEFECT-028 FIX: Check if type_params already contains a lifetime to avoid duplicates
        let has_existing_lifetime = type_params.iter().any(|p| p.starts_with('\''));
        let mut modified_type_params = type_params.to_vec();
        if needs_lifetime && !has_existing_lifetime {
            modified_type_params.insert(0, "'a".to_string());
        }

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
        } else if name == "main" && Self::has_non_unit_last_expr(body) {
            // BOOK-COMPAT-015: Main functions that end with a non-unit expression
            // should print it instead of trying to return it
            self.generate_main_body_with_print(body, is_async)?
        } else {
            self.generate_body_tokens(body, is_async)?
        };

        // TRANSPILER-007: Clear current function return type after body transpilation
        self.current_function_return_type.replace(None);

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
    fn has_non_unit_last_expr(body: &Expr) -> bool {
        match &body.kind {
            ExprKind::Block(exprs) => {
                if let Some(last) = exprs.last() {
                    Self::has_non_unit_last_expr(last)
                } else {
                    false
                }
            }
            ExprKind::Let { body, .. } | ExprKind::LetPattern { body, .. } => {
                Self::has_non_unit_last_expr(body)
            }
            _ => !Self::is_unit_expr(body),
        }
    }

    /// Check if expression is a unit expression (doesn't return a value)
    fn is_unit_expr(expr: &Expr) -> bool {
        use crate::frontend::ast::Literal;
        // RUCHYRUCHY-002: Add Literal(Unit) check for let statement bodies
        matches!(&expr.kind, ExprKind::Literal(Literal::Unit))
            || matches!(
                &expr.kind,
                ExprKind::Call { func, .. } if matches!(&func.kind, ExprKind::Identifier(name) if name == "println" || name == "print")
            )
            || matches!(
                &expr.kind,
                ExprKind::MacroInvocation { name, .. } if name == "println" || name == "print"
            )
            || matches!(&expr.kind, ExprKind::Assign { .. })
            || matches!(&expr.kind, ExprKind::Return { value: None })
            || matches!(&expr.kind, ExprKind::Return { value: Some(_) })  // Return with value is still unit for main
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    fn create_transpiler() -> Transpiler {
        Transpiler::new()
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
