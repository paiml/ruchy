//! Effect system transpilation - SPEC-001-I, SPEC-001-J
use anyhow::bail;
use super::{Result, Transpiler};
use crate::frontend::ast::{EffectOperation, EffectHandler, Expr, Pattern};
use proc_macro2::TokenStream;
use quote::quote;

impl Transpiler {
    /// SPEC-001-I: Transpile effect declaration to Rust trait
    ///
    /// # Errors
    ///
    /// Returns an error if transpilation fails
    pub fn transpile_effect(&self, name: &str, operations: &[EffectOperation]) -> Result<TokenStream> {
        let effect_name = syn::parse_str::<syn::Ident>(name)?;
        let methods = transpile_effect_operations(self, operations)?;

        Ok(quote! {
            pub trait #effect_name {
                #(#methods)*
            }
        })
    }

    /// SPEC-001-J: Transpile effect handler expression
    ///
    /// # Errors
    ///
    /// Returns an error if transpilation fails
    pub fn transpile_handler(&self, expr: &Expr, _handlers: &[EffectHandler]) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        Ok(quote! {
            {
                let _ = #expr_tokens;
                ()
            }
        })
    }
}

fn transpile_effect_operations(
    transpiler: &Transpiler,
    operations: &[EffectOperation],
) -> Result<Vec<TokenStream>> {
    operations
        .iter()
        .map(|op| transpile_single_operation(transpiler, op))
        .collect()
}

fn transpile_single_operation(
    transpiler: &Transpiler,
    op: &EffectOperation,
) -> Result<TokenStream> {
    let op_name = syn::parse_str::<syn::Ident>(&op.name)?;
    let params = transpile_operation_params(transpiler, op)?;
    let return_type = transpile_operation_return_type(transpiler, op)?;
    
    Ok(quote! {
        fn #op_name(&self, #(#params),*) #return_type;
    })
}

fn transpile_operation_params(
    transpiler: &Transpiler,
    op: &EffectOperation,
) -> Result<Vec<TokenStream>> {
    op.params
        .iter()
        .map(|param| {
            let param_name = match &param.pattern {
                Pattern::Identifier(name) => syn::parse_str::<syn::Ident>(name)?,
                _ => bail!("Only identifier patterns supported in effect operation parameters"),
            };
            let param_type = transpiler.transpile_type(&param.ty)?;
            Ok(quote! { #param_name: #param_type })
        })
        .collect()
}

fn transpile_operation_return_type(
    transpiler: &Transpiler,
    op: &EffectOperation,
) -> Result<TokenStream> {
    if let Some(return_type) = &op.return_type {
        let return_tokens = transpiler.transpile_type(return_type)?;
        Ok(quote! { -> #return_tokens })
    } else {
        Ok(quote! {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span, Type, TypeKind};

    // Test 1: transpile_handler with integer literal
    #[test]
    fn test_transpile_handler_integer() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let result = transpiler.transpile_handler(&expr, &[]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("42"));
    }

    // Test 2: transpile_handler with string literal
    #[test]
    fn test_transpile_handler_string() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            Span::default(),
        );
        let result = transpiler.transpile_handler(&expr, &[]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("hello"));
    }

    // Test 3: transpile_handler with identifier
    #[test]
    fn test_transpile_handler_identifier() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Identifier("result".to_string()),
            Span::default(),
        );
        let result = transpiler.transpile_handler(&expr, &[]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("result"));
    }

    // Test 4: transpile_handler with boolean literal
    #[test]
    fn test_transpile_handler_boolean() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::default(),
        );
        let result = transpiler.transpile_handler(&expr, &[]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("true"));
    }

    // Test 5: transpile_operation_return_type with None
    #[test]
    fn test_transpile_operation_return_type_none() {
        let transpiler = Transpiler::new();
        let operation = EffectOperation {
            name: "action".to_string(),
            params: vec![],
            return_type: None,
        };
        let result = transpile_operation_return_type(&transpiler, &operation);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        // Empty return type (no arrow)
        assert!(!tokens.contains("->"));
    }

    // Test 6: transpile_operation_return_type with Some (i32)
    #[test]
    fn test_transpile_operation_return_type_i32() {
        let transpiler = Transpiler::new();
        let operation = EffectOperation {
            name: "get".to_string(),
            params: vec![],
            return_type: Some(Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::default(),
            }),
        };
        let result = transpile_operation_return_type(&transpiler, &operation);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("->"));
        assert!(tokens.contains("i32"));
    }

    // Test 7: transpile_operation_return_type with Some (String)
    #[test]
    fn test_transpile_operation_return_type_string() {
        let transpiler = Transpiler::new();
        let operation = EffectOperation {
            name: "read".to_string(),
            params: vec![],
            return_type: Some(Type {
                kind: TypeKind::Named("String".to_string()),
                span: Span::default(),
            }),
        };
        let result = transpile_operation_return_type(&transpiler, &operation);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("->"));
        assert!(tokens.contains("String"));
    }

    // Test 8: transpile_effect_operations with empty slice
    #[test]
    fn test_transpile_effect_operations_empty() {
        let transpiler = Transpiler::new();
        let result = transpile_effect_operations(&transpiler, &[]);
        assert!(result.is_ok());
        let operations = result.unwrap();
        assert_eq!(operations.len(), 0);
    }

    // Test 9: transpile_effect with no operations
    #[test]
    fn test_transpile_effect_empty() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_effect("EmptyEffect", &[]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("trait EmptyEffect"));
    }

    // Test 10: transpile_effect with valid effect name
    #[test]
    fn test_transpile_effect_valid_name() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_effect("FileIO", &[]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("pub trait FileIO"));
    }
}
