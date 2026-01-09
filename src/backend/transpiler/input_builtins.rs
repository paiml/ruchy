//! Input Built-in Function Transpilation
//!
//! This module handles transpilation of input/readline functions:
//! - `input()` - read line from stdin with optional prompt
//! - `readline()` - read line from stdin
//!
//! **EXTREME TDD Round 57**: Extracted from statements.rs for modularization.

use super::Transpiler;
use crate::frontend::ast::Expr;
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::quote;

impl Transpiler {
    /// Handle input functions (input, readline)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    ///
    /// let mut transpiler = Transpiler::new();
    /// let mut parser = Parser::new("input()");
    /// let ast = parser.parse().expect("Failed to parse");
    /// let result = transpiler.transpile(&ast).expect("transpile should succeed in test").to_string();
    /// assert!(result.contains("read_line"));
    /// ```
    /// Complexity: 5 (within Toyota Way limits)
    pub fn try_transpile_input_function(
        &self,
        base_name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        match base_name {
            "input" => {
                if args.len() > 1 {
                    bail!("input expects 0 or 1 arguments (optional prompt)");
                }
                if args.is_empty() {
                    Ok(Some(self.generate_input_without_prompt()))
                } else {
                    let prompt = self.transpile_expr(&args[0])?;
                    Ok(Some(self.generate_input_with_prompt(prompt)))
                }
            }
            "readline" if args.is_empty() => Ok(Some(self.generate_input_without_prompt())),
            _ => Ok(None),
        }
    }

    /// Generate input reading code without prompt
    /// Complexity: 1 (within Toyota Way limits)
    pub fn generate_input_without_prompt(&self) -> TokenStream {
        quote! {
            {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).expect("Failed to read input");
                if input.ends_with('\n') {
                    input.pop();
                    if input.ends_with('\r') {
                        input.pop();
                    }
                }
                input
            }
        }
    }

    /// Generate input reading code with prompt
    /// Complexity: 1 (within Toyota Way limits)
    pub fn generate_input_with_prompt(&self, prompt: TokenStream) -> TokenStream {
        quote! {
            {
                print!("{}", #prompt);
                let _ = std::io::Write::flush(&mut std::io::stdout());
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).expect("Failed to read input");
                if input.ends_with('\n') {
                    input.pop();
                    if input.ends_with('\r') {
                        input.pop();
                    }
                }
                input
            }
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

    // ========================================================================
    // try_transpile_input_function tests
    // ========================================================================

    #[test]
    fn test_try_transpile_input_function_no_args() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_input_function("input", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("read_line"));
        assert!(tokens_str.contains("stdin"));
    }

    #[test]
    fn test_try_transpile_input_function_with_prompt() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("Enter name: ")];
        let result = transpiler.try_transpile_input_function("input", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("print"));
        assert!(tokens_str.contains("read_line"));
        assert!(tokens_str.contains("flush"));
    }

    #[test]
    fn test_try_transpile_input_function_too_many_args() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("prompt1"), string_expr("prompt2")];
        let result = transpiler.try_transpile_input_function("input", &args);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("0 or 1 arguments"));
    }

    #[test]
    fn test_try_transpile_input_function_readline() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_input_function("readline", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("read_line"));
    }

    #[test]
    fn test_try_transpile_input_function_readline_with_args() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("ignored")];
        let result = transpiler.try_transpile_input_function("readline", &args);
        assert!(result.is_ok());
        // readline with args returns None (not handled)
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_try_transpile_input_function_unknown() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_input_function("unknown_input", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ========================================================================
    // generate_input_without_prompt tests
    // ========================================================================

    #[test]
    fn test_generate_input_without_prompt() {
        let transpiler = Transpiler::new();
        let result = transpiler.generate_input_without_prompt();
        let tokens_str = result.to_string();
        assert!(tokens_str.contains("String :: new"));
        assert!(tokens_str.contains("stdin"));
        assert!(tokens_str.contains("read_line"));
        assert!(tokens_str.contains("ends_with"));
        assert!(tokens_str.contains("pop"));
    }

    #[test]
    fn test_generate_input_without_prompt_handles_newline() {
        let transpiler = Transpiler::new();
        let result = transpiler.generate_input_without_prompt();
        let tokens_str = result.to_string();
        // Should check for both \n and \r
        assert!(tokens_str.contains("'\\n'"));
        assert!(tokens_str.contains("'\\r'"));
    }

    // ========================================================================
    // generate_input_with_prompt tests
    // ========================================================================

    #[test]
    fn test_generate_input_with_prompt() {
        let transpiler = Transpiler::new();
        let prompt = quote! { "Enter value: " };
        let result = transpiler.generate_input_with_prompt(prompt);
        let tokens_str = result.to_string();
        assert!(tokens_str.contains("print"));
        assert!(tokens_str.contains("flush"));
        assert!(tokens_str.contains("read_line"));
    }

    #[test]
    fn test_generate_input_with_prompt_flushes_stdout() {
        let transpiler = Transpiler::new();
        let prompt = quote! { ">" };
        let result = transpiler.generate_input_with_prompt(prompt);
        let tokens_str = result.to_string();
        // Must flush stdout before reading
        assert!(tokens_str.contains("std :: io :: Write :: flush"));
        assert!(tokens_str.contains("stdout"));
    }

    #[test]
    fn test_generate_input_with_prompt_strips_newline() {
        let transpiler = Transpiler::new();
        let prompt = quote! { "test" };
        let result = transpiler.generate_input_with_prompt(prompt);
        let tokens_str = result.to_string();
        // Should handle both \n and \r
        assert!(tokens_str.contains("ends_with"));
        assert!(tokens_str.contains("pop"));
    }

    // ========================================================================
    // Integration-style tests
    // ========================================================================

    #[test]
    fn test_input_with_integer_prompt() {
        let transpiler = Transpiler::new();
        let args = vec![int_expr(42)];
        let result = transpiler.try_transpile_input_function("input", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        // Should work with any expression as prompt
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("print"));
    }
}
