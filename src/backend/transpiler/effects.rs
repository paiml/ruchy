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

    // Test 11: transpile_operation_params with valid Identifier pattern
    #[test]
    fn test_transpile_operation_params_identifier() {
        use crate::frontend::ast::{Param, Pattern};
        let transpiler = Transpiler::new();
        let operation = EffectOperation {
            name: "write".to_string(),
            params: vec![Param {
                pattern: Pattern::Identifier("data".to_string()),
                ty: Type {
                    kind: TypeKind::Named("String".to_string()),
                    span: Span::default(),
                },
                span: Span::default(),
                is_mutable: false,
                default_value: None,
            }],
            return_type: None,
        };
        let result = transpile_operation_params(&transpiler, &operation);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.len(), 1);
        let param_str = params[0].to_string();
        assert!(param_str.contains("data"));
        assert!(param_str.contains("String"));
    }

    // Test 12: transpile_operation_params with non-Identifier pattern (ERROR PATH - line 73)
    #[test]
    fn test_transpile_operation_params_non_identifier_error() {
        use crate::frontend::ast::{Param, Pattern};
        let transpiler = Transpiler::new();
        let operation = EffectOperation {
            name: "destructure".to_string(),
            params: vec![Param {
                pattern: Pattern::Tuple(vec![
                    Pattern::Identifier("a".to_string()),
                    Pattern::Identifier("b".to_string()),
                ]),
                ty: Type {
                    kind: TypeKind::Named("(i32, i32)".to_string()),
                    span: Span::default(),
                },
                span: Span::default(),
                is_mutable: false,
                default_value: None,
            }],
            return_type: None,
        };
        let result = transpile_operation_params(&transpiler, &operation);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Only identifier patterns supported"));
    }

    // Test 13: transpile_single_operation with no params
    #[test]
    fn test_transpile_single_operation_no_params() {
        let transpiler = Transpiler::new();
        let operation = EffectOperation {
            name: "tick".to_string(),
            params: vec![],
            return_type: None,
        };
        let result = transpile_single_operation(&transpiler, &operation);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("fn tick"));
        assert!(tokens.contains("self"));
    }

    // Test 14: transpile_single_operation with one param
    #[test]
    fn test_transpile_single_operation_one_param() {
        use crate::frontend::ast::{Param, Pattern};
        let transpiler = Transpiler::new();
        let operation = EffectOperation {
            name: "log".to_string(),
            params: vec![Param {
                pattern: Pattern::Identifier("message".to_string()),
                ty: Type {
                    kind: TypeKind::Named("String".to_string()),
                    span: Span::default(),
                },
                span: Span::default(),
                is_mutable: false,
                default_value: None,
            }],
            return_type: None,
        };
        let result = transpile_single_operation(&transpiler, &operation);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("fn log"));
        assert!(tokens.contains("message"));
        assert!(tokens.contains("String"));
    }

    // Test 15: transpile_single_operation with multiple params
    #[test]
    fn test_transpile_single_operation_multiple_params() {
        use crate::frontend::ast::{Param, Pattern};
        let transpiler = Transpiler::new();
        let operation = EffectOperation {
            name: "send".to_string(),
            params: vec![
                Param {
                    pattern: Pattern::Identifier("to".to_string()),
                    ty: Type {
                        kind: TypeKind::Named("Address".to_string()),
                        span: Span::default(),
                    },
                    span: Span::default(),
                    is_mutable: false,
                    default_value: None,
                },
                Param {
                    pattern: Pattern::Identifier("message".to_string()),
                    ty: Type {
                        kind: TypeKind::Named("String".to_string()),
                        span: Span::default(),
                    },
                    span: Span::default(),
                    is_mutable: false,
                    default_value: None,
                },
            ],
            return_type: None,
        };
        let result = transpile_single_operation(&transpiler, &operation);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("fn send"));
        assert!(tokens.contains("to : Address"));
        assert!(tokens.contains("message : String"));
    }

    // Test 16: transpile_single_operation with return type
    #[test]
    fn test_transpile_single_operation_with_return() {
        let transpiler = Transpiler::new();
        let operation = EffectOperation {
            name: "fetch".to_string(),
            params: vec![],
            return_type: Some(Type {
                kind: TypeKind::Named("Data".to_string()),
                span: Span::default(),
            }),
        };
        let result = transpile_single_operation(&transpiler, &operation);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("fn fetch"));
        assert!(tokens.contains("-> Data"));
    }

    // Test 17: transpile_effect_operations with single operation
    #[test]
    fn test_transpile_effect_operations_single() {
        let transpiler = Transpiler::new();
        let operations = vec![EffectOperation {
            name: "close".to_string(),
            params: vec![],
            return_type: None,
        }];
        let result = transpile_effect_operations(&transpiler, &operations);
        assert!(result.is_ok());
        let ops = result.unwrap();
        assert_eq!(ops.len(), 1);
        let op_str = ops[0].to_string();
        assert!(op_str.contains("fn close"));
    }

    // Test 18: transpile_effect_operations with multiple operations
    #[test]
    fn test_transpile_effect_operations_multiple() {
        let transpiler = Transpiler::new();
        let operations = vec![
            EffectOperation {
                name: "open".to_string(),
                params: vec![],
                return_type: None,
            },
            EffectOperation {
                name: "close".to_string(),
                params: vec![],
                return_type: None,
            },
            EffectOperation {
                name: "flush".to_string(),
                params: vec![],
                return_type: None,
            },
        ];
        let result = transpile_effect_operations(&transpiler, &operations);
        assert!(result.is_ok());
        let ops = result.unwrap();
        assert_eq!(ops.len(), 3);
        assert!(ops[0].to_string().contains("fn open"));
        assert!(ops[1].to_string().contains("fn close"));
        assert!(ops[2].to_string().contains("fn flush"));
    }

    // Test 19: transpile_effect with one operation
    #[test]
    fn test_transpile_effect_one_operation() {
        let transpiler = Transpiler::new();
        let operations = vec![EffectOperation {
            name: "reset".to_string(),
            params: vec![],
            return_type: None,
        }];
        let result = transpiler.transpile_effect("Resettable", &operations);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("pub trait Resettable"));
        assert!(tokens.contains("fn reset"));
    }

    // Test 20: transpile_effect with multiple operations
    #[test]
    fn test_transpile_effect_multiple_operations() {
        use crate::frontend::ast::{Param, Pattern};
        let transpiler = Transpiler::new();
        let operations = vec![
            EffectOperation {
                name: "read".to_string(),
                params: vec![],
                return_type: Some(Type {
                    kind: TypeKind::Named("String".to_string()),
                    span: Span::default(),
                }),
            },
            EffectOperation {
                name: "write".to_string(),
                params: vec![Param {
                    pattern: Pattern::Identifier("data".to_string()),
                    ty: Type {
                        kind: TypeKind::Named("String".to_string()),
                        span: Span::default(),
                    },
                    span: Span::default(),
                    is_mutable: false,
                    default_value: None,
                }],
                return_type: None,
            },
        ];
        let result = transpiler.transpile_effect("FileSystem", &operations);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("pub trait FileSystem"));
        assert!(tokens.contains("fn read"));
        assert!(tokens.contains("-> String"));
        assert!(tokens.contains("fn write"));
        assert!(tokens.contains("data : String"));
    }

    // Test 21: transpile_effect with invalid name (error path)
    #[test]
    fn test_transpile_effect_invalid_name() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_effect("123InvalidName", &[]);
        assert!(result.is_err());
    }

    // Test 22: transpile_handler with float literal
    #[test]
    fn test_transpile_handler_float() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Float(3.14)),
            Span::default(),
        );
        let result = transpiler.transpile_handler(&expr, &[]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("3.14"));
    }

    // Test 23: transpile_handler with null literal
    #[test]
    fn test_transpile_handler_null() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Null),
            Span::default(),
        );
        let result = transpiler.transpile_handler(&expr, &[]);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("None") || tokens.contains("null"));
    }

    // Test 24: transpile_operation_return_type with bool type
    #[test]
    fn test_transpile_operation_return_type_bool() {
        let transpiler = Transpiler::new();
        let operation = EffectOperation {
            name: "is_ready".to_string(),
            params: vec![],
            return_type: Some(Type {
                kind: TypeKind::Named("bool".to_string()),
                span: Span::default(),
            }),
        };
        let result = transpile_operation_return_type(&transpiler, &operation);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("->"));
        assert!(tokens.contains("bool"));
    }

    // Test 25: transpile_effect with operation having multiple params
    #[test]
    fn test_transpile_effect_multiple_params() {
        use crate::frontend::ast::{Param, Pattern};
        let transpiler = Transpiler::new();
        let operations = vec![EffectOperation {
            name: "process".to_string(),
            params: vec![
                Param {
                    pattern: Pattern::Identifier("input".to_string()),
                    ty: Type {
                        kind: TypeKind::Named("Data".to_string()),
                        span: Span::default(),
                    },
                    span: Span::default(),
                    is_mutable: false,
                    default_value: None,
                },
                Param {
                    pattern: Pattern::Identifier("config".to_string()),
                    ty: Type {
                        kind: TypeKind::Named("Config".to_string()),
                        span: Span::default(),
                    },
                    span: Span::default(),
                    is_mutable: false,
                    default_value: None,
                },
            ],
            return_type: Some(Type {
                kind: TypeKind::Named("Output".to_string()),
                span: Span::default(),
            }),
        }];
        let result = transpiler.transpile_effect("Processor", &operations);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("pub trait Processor"));
        assert!(tokens.contains("fn process"));
        assert!(tokens.contains("input"));
        assert!(tokens.contains("config"));
        assert!(tokens.contains("Data"));
        assert!(tokens.contains("Config"));
        assert!(tokens.contains("Output"));
    }
}
