//! Comprehensive unit tests for type_conversion_refactored module
//! Target: Increase coverage from 6.38% to 80%+

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::frontend::parser::Parser;
    use crate::frontend::ast::{Expr, ExprKind, Literal, StringPart};
    use proc_macro2::TokenStream;
    use quote::quote;

    fn create_transpiler() -> Transpiler {
        Transpiler::new()
    }

    fn parse_expr(code: &str) -> Expr {
        let mut parser = Parser::new(code);
        parser.parse_expression().expect("Failed to parse expression")
    }

    #[test]
    fn test_str_conversion_integer() {
        let transpiler = create_transpiler();
        let expr = parse_expr("str(42)");
        
        if let ExprKind::Call { func, args } = &expr.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                let result = transpiler.try_transpile_type_conversion_refactored(name, args)
                    .expect("Conversion failed");
                assert!(result.is_some());
                let tokens = result.unwrap();
                let output = tokens.to_string();
                assert!(output.contains("format"));
            }
        }
    }

    #[test]
    fn test_str_conversion_float() {
        let transpiler = create_transpiler();
        let expr = parse_expr("str(3.14)");
        
        if let ExprKind::Call { func, args } = &expr.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                let result = transpiler.try_transpile_type_conversion_refactored(name, args)
                    .expect("Conversion failed");
                assert!(result.is_some());
                let tokens = result.unwrap();
                let output = tokens.to_string();
                assert!(output.contains("format"));
            }
        }
    }

    #[test]
    fn test_str_conversion_bool() {
        let transpiler = create_transpiler();
        let expr = parse_expr("str(true)");
        
        if let ExprKind::Call { func, args } = &expr.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                let result = transpiler.try_transpile_type_conversion_refactored(name, args)
                    .expect("Conversion failed");
                assert!(result.is_some());
            }
        }
    }

    #[test]
    fn test_int_conversion_string_literal() {
        let transpiler = create_transpiler();
        let expr = parse_expr("int(\"123\")");
        
        if let ExprKind::Call { func, args } = &expr.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                let result = transpiler.try_transpile_type_conversion_refactored(name, args)
                    .expect("Conversion failed");
                assert!(result.is_some());
                let tokens = result.unwrap();
                let output = tokens.to_string();
                assert!(output.contains("parse"));
                assert!(output.contains("i64"));
            }
        }
    }

    #[test]
    fn test_int_conversion_float() {
        let transpiler = create_transpiler();
        let expr = parse_expr("int(3.14)");
        
        if let ExprKind::Call { func, args } = &expr.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                let result = transpiler.try_transpile_type_conversion_refactored(name, args)
                    .expect("Conversion failed");
                assert!(result.is_some());
                let tokens = result.unwrap();
                let output = tokens.to_string();
                assert!(output.contains("as i64"));
            }
        }
    }

    #[test]
    fn test_float_conversion_string() {
        let transpiler = create_transpiler();
        let expr = parse_expr("float(\"3.14\")");
        
        if let ExprKind::Call { func, args } = &expr.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                let result = transpiler.try_transpile_type_conversion_refactored(name, args)
                    .expect("Conversion failed");
                assert!(result.is_some());
                let tokens = result.unwrap();
                let output = tokens.to_string();
                assert!(output.contains("parse"));
                assert!(output.contains("f64"));
            }
        }
    }

    #[test]
    fn test_float_conversion_int() {
        let transpiler = create_transpiler();
        let expr = parse_expr("float(42)");
        
        if let ExprKind::Call { func, args } = &expr.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                let result = transpiler.try_transpile_type_conversion_refactored(name, args)
                    .expect("Conversion failed");
                assert!(result.is_some());
                let tokens = result.unwrap();
                let output = tokens.to_string();
                assert!(output.contains("as f64"));
            }
        }
    }

    #[test]
    fn test_bool_conversion_zero() {
        let transpiler = create_transpiler();
        let expr = parse_expr("bool(0)");
        
        if let ExprKind::Call { func, args } = &expr.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                let result = transpiler.try_transpile_type_conversion_refactored(name, args)
                    .expect("Conversion failed");
                assert!(result.is_some());
                let tokens = result.unwrap();
                let output = tokens.to_string();
                assert!(output.contains("!= 0"));
            }
        }
    }

    #[test]
    fn test_bool_conversion_nonzero() {
        let transpiler = create_transpiler();
        let expr = parse_expr("bool(42)");
        
        if let ExprKind::Call { func, args } = &expr.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                let result = transpiler.try_transpile_type_conversion_refactored(name, args)
                    .expect("Conversion failed");
                assert!(result.is_some());
                let tokens = result.unwrap();
                let output = tokens.to_string();
                assert!(output.contains("!= 0"));
            }
        }
    }

    #[test]
    fn test_bool_conversion_empty_string() {
        let transpiler = create_transpiler();
        let expr = parse_expr("bool(\"\")");
        
        if let ExprKind::Call { func, args } = &expr.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                let result = transpiler.try_transpile_type_conversion_refactored(name, args)
                    .expect("Conversion failed");
                assert!(result.is_some());
                let tokens = result.unwrap();
                let output = tokens.to_string();
                assert!(output.contains("is_empty"));
            }
        }
    }

    #[test]
    fn test_bool_conversion_nonempty_string() {
        let transpiler = create_transpiler();
        let expr = parse_expr("bool(\"hello\")");
        
        if let ExprKind::Call { func, args } = &expr.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                let result = transpiler.try_transpile_type_conversion_refactored(name, args)
                    .expect("Conversion failed");
                assert!(result.is_some());
                let tokens = result.unwrap();
                let output = tokens.to_string();
                assert!(output.contains("is_empty"));
            }
        }
    }

    #[test]
    fn test_list_conversion_string() {
        let transpiler = create_transpiler();
        let expr = parse_expr("list(\"hello\")");
        
        if let ExprKind::Call { func, args } = &expr.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                let result = transpiler.try_transpile_type_conversion_refactored(name, args)
                    .expect("Conversion failed");
                assert!(result.is_some());
                let tokens = result.unwrap();
                let output = tokens.to_string();
                assert!(output.contains("chars") || output.contains("collect"));
            }
        }
    }

    #[test]
    fn test_list_conversion_range() {
        let transpiler = create_transpiler();
        let expr = parse_expr("list(0..10)");
        
        if let ExprKind::Call { func, args } = &expr.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                let result = transpiler.try_transpile_type_conversion_refactored(name, args)
                    .expect("Conversion failed");
                assert!(result.is_some());
                let tokens = result.unwrap();
                let output = tokens.to_string();
                assert!(output.contains("collect"));
            }
        }
    }

    #[test]
    fn test_set_conversion_list() {
        let transpiler = create_transpiler();
        let expr = parse_expr("set([1, 2, 3])");
        
        if let ExprKind::Call { func, args } = &expr.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                let result = transpiler.try_transpile_type_conversion_refactored(name, args)
                    .expect("Conversion failed");
                assert!(result.is_some());
                let tokens = result.unwrap();
                let output = tokens.to_string();
                assert!(output.contains("HashSet"));
            }
        }
    }

    #[test]
    fn test_not_type_conversion() {
        let transpiler = create_transpiler();
        let expr = parse_expr("print(42)");
        
        if let ExprKind::Call { func, args } = &expr.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                let result = transpiler.try_transpile_type_conversion_refactored(name, args)
                    .expect("Should not fail");
                assert!(result.is_none()); // Not a type conversion
            }
        }
    }

    #[test]
    fn test_invalid_arg_count() {
        let transpiler = create_transpiler();
        let expr = parse_expr("int(1, 2)");
        
        if let ExprKind::Call { func, args } = &expr.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                let result = transpiler.try_transpile_type_conversion_refactored(name, args);
                assert!(result.is_err()); // Should fail with wrong arg count
            }
        }
    }

    #[test]
    fn test_string_interpolation_to_bool() {
        let transpiler = create_transpiler();
        // Create a string interpolation expression manually
        let expr = Expr {
            kind: ExprKind::Call {
                func: Box::new(Expr {
                    kind: ExprKind::Identifier("bool".to_string()),
                    span: Span::dummy(),
                }),
                args: vec![Expr {
                    kind: ExprKind::StringInterpolation {
                        parts: vec![StringPart::Text("test".to_string())],
                    },
                    span: Span::dummy(),
                }],
            },
            span: Span::dummy(),
        };
        
        if let ExprKind::Call { func, args } = &expr.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                let result = transpiler.try_transpile_type_conversion_refactored(name, args)
                    .expect("Conversion failed");
                assert!(result.is_some());
            }
        }
    }

    #[test]
    fn test_dict_conversion() {
        let transpiler = create_transpiler();
        // Create a simple expression for dict conversion
        let expr = Expr {
            kind: ExprKind::Call {
                func: Box::new(Expr {
                    kind: ExprKind::Identifier("dict".to_string()),
                    span: Span::dummy(),
                }),
                args: vec![Expr {
                    kind: ExprKind::List(vec![]),
                    span: Span::dummy(),
                }],
            },
            span: Span::dummy(),
        };
        
        if let ExprKind::Call { func, args } = &expr.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                let result = transpiler.try_transpile_type_conversion_refactored(name, args)
                    .expect("Conversion failed");
                assert!(result.is_some());
                let tokens = result.unwrap();
                let output = tokens.to_string();
                assert!(output.contains("HashMap"));
            }
        }
    }

    #[test]
    fn test_all_type_conversions_exist() {
        let transpiler = create_transpiler();
        let conversions = vec!["str", "int", "float", "bool", "list", "set", "dict"];
        
        for conv in conversions {
            let code = format!("{}(42)", conv);
            let expr = parse_expr(&code);
            
            if let ExprKind::Call { func, args } = &expr.kind {
                if let ExprKind::Identifier(name) = &func.kind {
                    let result = transpiler.try_transpile_type_conversion_refactored(name, args);
                    assert!(result.is_ok(), "Failed for {}", conv);
                    assert!(result.unwrap().is_some(), "None for {}", conv);
                }
            }
        }
    }
}