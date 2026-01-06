//! Function transpilation helpers
//! EXTREME TDD Round 81: Extracted from statements.rs
//!
//! This module handles function definition transpilation.

use crate::frontend::ast::{Expr, Param, Type, TypeKind};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::format_ident;

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
}
