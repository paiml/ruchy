//! Function Call Helper Transpilation
//!
//! This module handles transpilation of function call helpers:
//! - Result/Option call handling (Ok, Err, Some)
//! - Regular function call transpilation with string coercion
//! - String type coercion for function arguments
//!
//! **EXTREME TDD Round 63**: Extracted from statements.rs for modularization.

use super::Transpiler;
use crate::frontend::ast::Expr;
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

impl Transpiler {
    /// Transpile Ok/Err/Some calls with automatic string conversion
    ///
    /// DEFECT-STRING-RESULT FIX: When Ok/Err/Some are parsed as Call expressions
    /// (e.g., in return positions), convert string literals to String.
    ///
    /// This complements the `ExprKind::Ok/Err/Some` handlers in dispatcher.rs.
    /// Complexity: 5 (within Toyota Way limits)
    pub fn try_transpile_result_call(
        &self,
        base_name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        use crate::frontend::ast::{ExprKind, Literal};

        // Only handle Ok, Err, and Some constructors
        if base_name != "Ok" && base_name != "Err" && base_name != "Some" {
            return Ok(None);
        }

        // Transpile all arguments, converting string literals to String
        let arg_tokens: Result<Vec<_>> = args
            .iter()
            .map(|arg| {
                let base_tokens = self.transpile_expr(arg)?;
                // Convert string literals to String for Result/Option type compatibility
                match &arg.kind {
                    ExprKind::Literal(Literal::String(_)) => {
                        Ok(quote! { #base_tokens.to_string() })
                    }
                    _ => Ok(base_tokens),
                }
            })
            .collect();

        let arg_tokens = arg_tokens?;
        let func_ident = proc_macro2::Ident::new(base_name, proc_macro2::Span::call_site());

        Ok(Some(quote! { #func_ident(#(#arg_tokens),*) }))
    }

    /// Handle regular function calls with string literal conversion
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    ///
    /// let mut transpiler = Transpiler::new();
    /// let mut parser = Parser::new(r#"my_func("test")"#);
    /// let ast = parser.parse().expect("Failed to parse");
    /// let result = transpiler.transpile(&ast).expect("transpile should succeed in test").to_string();
    /// assert!(result.contains("my_func"));
    /// ```
    /// Complexity: 8 (within Toyota Way limits)
    pub fn transpile_regular_function_call(
        &self,
        func_tokens: &TokenStream,
        args: &[Expr],
    ) -> Result<TokenStream> {
        // Get function name for signature lookup
        let func_name = func_tokens.to_string().trim().to_string();
        // Apply type coercion based on function signature
        let arg_tokens: Result<Vec<_>> = if let Some(signature) =
            self.function_signatures.get(&func_name)
        {
            args.iter()
                .enumerate()
                .map(|(i, arg)| {
                    let mut base_tokens = self.transpile_expr(arg)?;

                    // Apply String/&str coercion if needed
                    if let Some(expected_type) = signature.param_types.get(i) {
                        // DEFECT-018 FIX: For Identifier args in loops, use .to_string()
                        // for String params (handles &str->String), or .clone() for others
                        if self.in_loop_context.get()
                            && matches!(&arg.kind, crate::frontend::ast::ExprKind::Identifier(_))
                        {
                            if expected_type == "String" {
                                // Use .to_string() which handles both &str and String
                                return Ok(quote! { #base_tokens.to_string() });
                            }
                            base_tokens = quote! { #base_tokens.clone() };
                        }
                        self.apply_string_coercion(arg, &base_tokens, expected_type)
                    } else {
                        // DEFECT-018 FIX: Auto-clone Identifier arguments in loop contexts
                        // to prevent "use of moved value" errors on subsequent iterations
                        if self.in_loop_context.get()
                            && matches!(&arg.kind, crate::frontend::ast::ExprKind::Identifier(_))
                        {
                            base_tokens = quote! { #base_tokens.clone() };
                        }
                        Ok(base_tokens)
                    }
                })
                .collect()
        } else {
            // No signature info - transpile with type conversions
            args.iter()
                .map(|arg| {
                    let mut base_tokens = self.transpile_expr(arg)?;

                    // BOOK-COMPAT-017: String literals passed to functions should convert to String
                    // Most functions with string params need String not &str
                    if matches!(
                        &arg.kind,
                        crate::frontend::ast::ExprKind::Literal(
                            crate::frontend::ast::Literal::String(_)
                        )
                    ) {
                        return Ok(quote! { #base_tokens.to_string() });
                    }

                    // BOOK-COMPAT-017: Array literals passed to functions should convert to Vec
                    // Most functions with untyped params expect Vec<T> semantics
                    if let crate::frontend::ast::ExprKind::List(elements) = &arg.kind {
                        if !elements.is_empty() {
                            return Ok(quote! { #base_tokens.to_vec() });
                        }
                    }

                    // DEFECT-018 FIX: Auto-clone Identifier arguments in loop contexts
                    if self.in_loop_context.get()
                        && matches!(&arg.kind, crate::frontend::ast::ExprKind::Identifier(_))
                    {
                        base_tokens = quote! { #base_tokens.clone() };
                    }

                    Ok(base_tokens)
                })
                .collect()
        };
        let arg_tokens = arg_tokens?;
        Ok(quote! { #func_tokens(#(#arg_tokens),*) })
    }

    /// Apply type coercion based on expected type
    /// Complexity: 6 (within Toyota Way limits)
    pub fn apply_string_coercion(
        &self,
        arg: &Expr,
        tokens: &TokenStream,
        expected_type: &str,
    ) -> Result<TokenStream> {
        use crate::frontend::ast::{ExprKind, Literal};
        match (&arg.kind, expected_type) {
            // String literal to String parameter: add .to_string()
            (ExprKind::Literal(Literal::String(_)), "String") => Ok(quote! { #tokens.to_string() }),
            // BOOK-COMPAT-017: String literal to Any/Unknown parameter (inferred as String from body)
            // String operations in function body typically infer String type, so convert
            (ExprKind::Literal(Literal::String(_)), "Any" | "Unknown") => {
                Ok(quote! { #tokens.to_string() })
            }
            // BOOK-COMPAT-017: Array literal to Any parameter (function with inferred Vec<T> type)
            // Convert array [1, 2, 3] to vec via .to_vec() for functions expecting Vec
            // But NOT when expected type is explicit array type [T; N]
            (ExprKind::List(elements), "Any" | "Unknown") if !elements.is_empty() => {
                Ok(quote! { #tokens.to_vec() })
            }
            // Array literal to explicit array type [T; N]: keep as-is
            (ExprKind::List(_), expected)
                if expected.starts_with('[') && expected.contains(';') =>
            {
                Ok(tokens.clone())
            }
            // String literal to &str parameter: keep as-is
            (ExprKind::Literal(Literal::String(_)), expected) if expected.starts_with('&') => {
                Ok(tokens.clone())
            }
            // Variable that might be &str to String parameter
            (ExprKind::Identifier(_), "String") => {
                // DEFECT-018 FIX: Use .to_string() which handles both:
                // - &str -> String (converts)
                // - String -> String (via Display trait, allocates but is correct)
                // This is needed because Ruchy string literals default to &str
                Ok(quote! { #tokens.to_string() })
            }
            // No coercion needed
            _ => Ok(tokens.clone()),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, Span};

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn string_expr(s: &str) -> Expr {
        make_expr(ExprKind::Literal(Literal::String(s.to_string())))
    }

    fn int_expr(n: i64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Integer(n, None)))
    }

    fn ident_expr(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    // ========================================================================
    // try_transpile_result_call tests
    // ========================================================================

    #[test]
    fn test_result_call_ok() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(42)];
        let result = transpiler.try_transpile_result_call("Ok", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("Ok"));
        assert!(tokens_str.contains("42"));
    }

    #[test]
    fn test_result_call_ok_with_string() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("success")];
        let result = transpiler.try_transpile_result_call("Ok", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("Ok"));
        assert!(tokens_str.contains("to_string"));
    }

    #[test]
    fn test_result_call_err() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("error message")];
        let result = transpiler.try_transpile_result_call("Err", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("Err"));
        assert!(tokens_str.contains("to_string"));
    }

    #[test]
    fn test_result_call_some() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(100)];
        let result = transpiler.try_transpile_result_call("Some", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("Some"));
    }

    #[test]
    fn test_result_call_some_with_string() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("value")];
        let result = transpiler.try_transpile_result_call("Some", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("Some"));
        assert!(tokens_str.contains("to_string"));
    }

    #[test]
    fn test_result_call_unknown() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(1)];
        let result = transpiler.try_transpile_result_call("Unknown", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_result_call_ok_empty() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_result_call("Ok", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("Ok"));
    }

    #[test]
    fn test_result_call_ok_multiple_args() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(1), int_expr(2)];
        let result = transpiler.try_transpile_result_call("Ok", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
    }

    // ========================================================================
    // transpile_regular_function_call tests
    // ========================================================================

    #[test]
    fn test_regular_function_call_simple() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { my_func };
        let args = vec![int_expr(42)];
        let result = transpiler.transpile_regular_function_call(&func_tokens, &args);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("my_func"));
        assert!(tokens_str.contains("42"));
    }

    #[test]
    fn test_regular_function_call_with_string() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { greet };
        let args = vec![string_expr("hello")];
        let result = transpiler.transpile_regular_function_call(&func_tokens, &args);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("greet"));
        assert!(tokens_str.contains("hello"));
    }

    #[test]
    fn test_regular_function_call_no_args() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { get_value };
        let args: Vec<Expr> = vec![];
        let result = transpiler.transpile_regular_function_call(&func_tokens, &args);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("get_value"));
    }

    #[test]
    fn test_regular_function_call_multiple_args() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { add };
        let args = vec![int_expr(1), int_expr(2), int_expr(3)];
        let result = transpiler.transpile_regular_function_call(&func_tokens, &args);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("add"));
    }

    #[test]
    fn test_regular_function_call_with_identifier() {
        let transpiler = Transpiler::new();
        let func_tokens = quote! { process };
        let args = vec![ident_expr("data")];
        let result = transpiler.transpile_regular_function_call(&func_tokens, &args);
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("process"));
        assert!(tokens_str.contains("data"));
    }

    // ========================================================================
    // apply_string_coercion tests
    // ========================================================================

    #[test]
    fn test_apply_string_coercion_string_literal_to_string() {
        let transpiler = Transpiler::new();
        let arg = string_expr("hello");
        let tokens = quote! { "hello" };
        let result = transpiler.apply_string_coercion(&arg, &tokens, "String");
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("to_string"));
    }

    #[test]
    fn test_apply_string_coercion_string_literal_to_str_ref() {
        let transpiler = Transpiler::new();
        let arg = string_expr("hello");
        let tokens = quote! { "hello" };
        let result = transpiler.apply_string_coercion(&arg, &tokens, "&str");
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        // Should keep as-is, no to_string
        assert!(!tokens_str.contains("to_string"));
    }

    #[test]
    fn test_apply_string_coercion_identifier_to_string() {
        let transpiler = Transpiler::new();
        let arg = ident_expr("name");
        let tokens = quote! { name };
        let result = transpiler.apply_string_coercion(&arg, &tokens, "String");
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(tokens_str.contains("to_string"));
    }

    #[test]
    fn test_apply_string_coercion_int_no_change() {
        let transpiler = Transpiler::new();
        let arg = int_expr(42);
        let tokens = quote! { 42 };
        let result = transpiler.apply_string_coercion(&arg, &tokens, "i32");
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(!tokens_str.contains("to_string"));
    }

    #[test]
    fn test_apply_string_coercion_identifier_to_int() {
        let transpiler = Transpiler::new();
        let arg = ident_expr("x");
        let tokens = quote! { x };
        let result = transpiler.apply_string_coercion(&arg, &tokens, "i32");
        assert!(result.is_ok());
        let tokens_str = result.unwrap().to_string();
        assert!(!tokens_str.contains("to_string"));
    }
}
