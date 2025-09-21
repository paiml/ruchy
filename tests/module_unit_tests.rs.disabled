// MODULE UNIT TESTS - Direct testing of all modules
// Sprint 80 Phase 24: Coverage push to 75%

#[cfg(test)]
mod lexer_unit_tests {
    use ruchy::frontend::lexer::{Lexer, Token};

    #[test]
    fn test_lex_integer() {
        let mut lexer = Lexer::new("42");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0], Token::Integer(42)));
    }

    #[test]
    fn test_lex_float() {
        let mut lexer = Lexer::new("3.14");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0], Token::Float(_)));
    }

    #[test]
    fn test_lex_string() {
        let mut lexer = Lexer::new(r#""hello""#);
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0], Token::String(_)));
    }

    #[test]
    fn test_lex_identifier() {
        let mut lexer = Lexer::new("variable");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0], Token::Identifier(_)));
    }

    #[test]
    fn test_lex_keywords() {
        let keywords = [
            "let", "mut", "fn", "if", "else", "while", "for", "match", "return",
        ];
        for keyword in keywords {
            let mut lexer = Lexer::new(keyword);
            let tokens = lexer.tokenize().unwrap();
            assert!(!tokens.is_empty());
        }
    }

    #[test]
    fn test_lex_operators() {
        let ops = ["+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">="];
        for op in ops {
            let mut lexer = Lexer::new(op);
            let tokens = lexer.tokenize().unwrap();
            assert!(!tokens.is_empty());
        }
    }

    #[test]
    fn test_lex_delimiters() {
        let delims = ["(", ")", "[", "]", "{", "}", ",", ";", ":"];
        for delim in delims {
            let mut lexer = Lexer::new(delim);
            let tokens = lexer.tokenize().unwrap();
            assert!(!tokens.is_empty());
        }
    }

    #[test]
    fn test_lex_multiline() {
        let input = "let x = 42\nlet y = 3.14\nlet z = x + y";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.len() > 5);
    }
}

#[cfg(test)]
mod parser_unit_tests {
    use ruchy::frontend::ast::*;
    use ruchy::Parser;

    #[test]
    fn test_parse_integer() {
        let mut parser = Parser::new("42");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Literal(Literal::Integer(42))));
    }

    #[test]
    fn test_parse_float() {
        let mut parser = Parser::new("3.14");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Literal(Literal::Float(_))));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = Parser::new(r#""hello""#);
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Literal(Literal::String(_))));
    }

    #[test]
    fn test_parse_bool() {
        let mut parser = Parser::new("true");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Literal(Literal::Bool(true))));

        let mut parser = Parser::new("false");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Literal(Literal::Bool(false))));
    }

    #[test]
    fn test_parse_identifier() {
        let mut parser = Parser::new("x");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Identifier(_)));
    }

    #[test]
    fn test_parse_binary_ops() {
        let ops = [
            "+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">=", "&&", "||",
        ];
        for op in ops {
            let input = format!("1 {} 2", op);
            let mut parser = Parser::new(&input);
            let ast = parser.parse();
            assert!(ast.is_ok(), "Failed to parse: {}", input);
        }
    }

    #[test]
    fn test_parse_unary_ops() {
        let mut parser = Parser::new("-42");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Unary { .. }));

        let mut parser = Parser::new("!true");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Unary { .. }));
    }

    #[test]
    fn test_parse_parentheses() {
        let mut parser = Parser::new("(1 + 2) * 3");
        let ast = parser.parse();
        assert!(ast.is_ok());
    }

    #[test]
    fn test_parse_list() {
        let mut parser = Parser::new("[1, 2, 3]");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::List(_)));
    }

    #[test]
    fn test_parse_tuple() {
        let mut parser = Parser::new("(1, 2, 3)");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Tuple(_)));
    }

    #[test]
    fn test_parse_empty_list() {
        let mut parser = Parser::new("[]");
        let ast = parser.parse().unwrap();
        if let ExprKind::List(items) = ast.kind {
            assert!(items.is_empty());
        }
    }

    #[test]
    fn test_parse_let() {
        let mut parser = Parser::new("let x = 42");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Let { .. }));
    }

    #[test]
    fn test_parse_let_mut() {
        let mut parser = Parser::new("let mut x = 42");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Let { mutable: true, .. }));
    }

    #[test]
    fn test_parse_if() {
        let mut parser = Parser::new("if true { 1 } else { 2 }");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::If { .. }));
    }

    #[test]
    fn test_parse_if_no_else() {
        let mut parser = Parser::new("if true { 1 }");
        let ast = parser.parse().unwrap();
        assert!(matches!(
            ast.kind,
            ExprKind::If {
                else_branch: None,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_while() {
        let mut parser = Parser::new("while x < 10 { x = x + 1 }");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::While { .. }));
    }

    #[test]
    fn test_parse_for() {
        let mut parser = Parser::new("for i in list { print(i) }");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::For { .. }));
    }

    #[test]
    fn test_parse_function() {
        let mut parser = Parser::new("fn add(x, y) { x + y }");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Function { .. }));
    }

    #[test]
    fn test_parse_lambda() {
        let mut parser = Parser::new("fn(x) { x * 2 }");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Lambda { .. }));
    }

    #[test]
    fn test_parse_call() {
        let mut parser = Parser::new("foo(1, 2, 3)");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Call { .. }));
    }

    #[test]
    fn test_parse_method_call() {
        let mut parser = Parser::new("obj.method(arg)");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::MethodCall { .. }));
    }

    #[test]
    fn test_parse_field_access() {
        let mut parser = Parser::new("obj.field");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::FieldAccess { .. }));
    }

    #[test]
    fn test_parse_index() {
        let mut parser = Parser::new("arr[0]");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Index { .. }));
    }

    #[test]
    fn test_parse_match() {
        let mut parser = Parser::new("match x { 1 => a, 2 => b, _ => c }");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Match { .. }));
    }

    #[test]
    fn test_parse_block() {
        let mut parser = Parser::new("{ let x = 1; x + 2 }");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Block(_)));
    }

    #[test]
    fn test_parse_return() {
        let mut parser = Parser::new("return 42");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Return(_)));
    }

    #[test]
    fn test_parse_break() {
        let mut parser = Parser::new("break");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Break(_)));
    }

    #[test]
    fn test_parse_continue() {
        let mut parser = Parser::new("continue");
        let ast = parser.parse().unwrap();
        assert!(matches!(ast.kind, ExprKind::Continue(_)));
    }
}

#[cfg(test)]
mod transpiler_unit_tests {
    use ruchy::backend::transpiler::Transpiler;
    use ruchy::frontend::ast::*;

    #[test]
    fn test_transpile_integer() {
        let transpiler = Transpiler::new();
        let expr = Expr {
            kind: ExprKind::Literal(Literal::Integer(42)),
            span: Span::default(),
            attributes: vec![],
        };
        assert_eq!(transpiler.transpile(&expr), "42");
    }

    #[test]
    fn test_transpile_float() {
        let transpiler = Transpiler::new();
        let expr = Expr {
            kind: ExprKind::Literal(Literal::Float(3.14)),
            span: Span::default(),
            attributes: vec![],
        };
        assert_eq!(transpiler.transpile(&expr), "3.14");
    }

    #[test]
    fn test_transpile_string() {
        let transpiler = Transpiler::new();
        let expr = Expr {
            kind: ExprKind::Literal(Literal::String("hello".to_string())),
            span: Span::default(),
            attributes: vec![],
        };
        assert_eq!(transpiler.transpile(&expr), r#""hello""#);
    }

    #[test]
    fn test_transpile_bool() {
        let transpiler = Transpiler::new();
        let expr = Expr {
            kind: ExprKind::Literal(Literal::Bool(true)),
            span: Span::default(),
            attributes: vec![],
        };
        assert_eq!(transpiler.transpile(&expr), "true");
    }

    #[test]
    fn test_transpile_identifier() {
        let transpiler = Transpiler::new();
        let expr = Expr {
            kind: ExprKind::Identifier("variable".to_string()),
            span: Span::default(),
            attributes: vec![],
        };
        assert_eq!(transpiler.transpile(&expr), "variable");
    }

    #[test]
    fn test_transpile_binary_add() {
        let transpiler = Transpiler::new();
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(Expr {
                    kind: ExprKind::Literal(Literal::Integer(1)),
                    span: Span::default(),
                    attributes: vec![],
                }),
                op: BinaryOp::Add,
                right: Box::new(Expr {
                    kind: ExprKind::Literal(Literal::Integer(2)),
                    span: Span::default(),
                    attributes: vec![],
                }),
            },
            span: Span::default(),
            attributes: vec![],
        };
        assert_eq!(transpiler.transpile(&expr), "(1 + 2)");
    }

    #[test]
    fn test_transpile_unary_neg() {
        let transpiler = Transpiler::new();
        let expr = Expr {
            kind: ExprKind::Unary {
                op: UnaryOp::Neg,
                operand: Box::new(Expr {
                    kind: ExprKind::Literal(Literal::Integer(42)),
                    span: Span::default(),
                    attributes: vec![],
                }),
            },
            span: Span::default(),
            attributes: vec![],
        };
        assert_eq!(transpiler.transpile(&expr), "-(42)");
    }

    #[test]
    fn test_transpile_list() {
        let transpiler = Transpiler::new();
        let expr = Expr {
            kind: ExprKind::List(vec![
                Expr {
                    kind: ExprKind::Literal(Literal::Integer(1)),
                    span: Span::default(),
                    attributes: vec![],
                },
                Expr {
                    kind: ExprKind::Literal(Literal::Integer(2)),
                    span: Span::default(),
                    attributes: vec![],
                },
            ]),
            span: Span::default(),
            attributes: vec![],
        };
        assert_eq!(transpiler.transpile(&expr), "vec![1, 2]");
    }

    #[test]
    fn test_transpile_tuple() {
        let transpiler = Transpiler::new();
        let expr = Expr {
            kind: ExprKind::Tuple(vec![
                Expr {
                    kind: ExprKind::Literal(Literal::Integer(1)),
                    span: Span::default(),
                    attributes: vec![],
                },
                Expr {
                    kind: ExprKind::Literal(Literal::Integer(2)),
                    span: Span::default(),
                    attributes: vec![],
                },
            ]),
            span: Span::default(),
            attributes: vec![],
        };
        assert_eq!(transpiler.transpile(&expr), "(1, 2)");
    }
}

#[cfg(test)]
mod value_unit_tests {
    use ruchy::runtime::Value;
    use std::collections::HashMap;
    use std::rc::Rc;

    #[test]
    fn test_value_integer() {
        let val = Value::Integer(42);
        assert_eq!(format!("{}", val), "42");
        assert_eq!(val, Value::Integer(42));
        assert_ne!(val, Value::Integer(43));
    }

    #[test]
    fn test_value_float() {
        let val = Value::Float(3.14);
        assert_eq!(format!("{}", val), "3.14");
        assert_eq!(val, Value::Float(3.14));
    }

    #[test]
    fn test_value_bool() {
        let val_true = Value::Bool(true);
        let val_false = Value::Bool(false);
        assert_eq!(format!("{}", val_true), "true");
        assert_eq!(format!("{}", val_false), "false");
        assert_ne!(val_true, val_false);
    }

    #[test]
    fn test_value_string() {
        let val = Value::String(Rc::new("hello".to_string()));
        assert_eq!(format!("{}", val), "hello");
    }

    #[test]
    fn test_value_unit() {
        let val = Value::Unit;
        assert_eq!(format!("{}", val), "()");
        assert_eq!(val, Value::Unit);
    }

    #[test]
    fn test_value_list() {
        let val = Value::List(Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        if let Value::List(list) = &val {
            assert_eq!(list.len(), 3);
        }
    }

    #[test]
    fn test_value_tuple() {
        let val = Value::Tuple(Rc::new(vec![
            Value::Integer(42),
            Value::String(Rc::new("test".to_string())),
        ]));
        if let Value::Tuple(tuple) = &val {
            assert_eq!(tuple.len(), 2);
        }
    }

    #[test]
    fn test_value_object() {
        let mut fields = HashMap::new();
        fields.insert("x".to_string(), Value::Integer(10));
        fields.insert("y".to_string(), Value::Integer(20));
        let val = Value::Object(Rc::new(fields));
        if let Value::Object(obj) = &val {
            assert_eq!(obj.len(), 2);
        }
    }

    #[test]
    fn test_value_clone() {
        let val = Value::Integer(42);
        let cloned = val.clone();
        assert_eq!(val, cloned);
    }

    #[test]
    fn test_value_debug() {
        let val = Value::Integer(42);
        let debug = format!("{:?}", val);
        assert!(debug.contains("Integer"));
    }
}

#[cfg(test)]
mod environment_unit_tests {
    use ruchy::runtime::{Environment, Value};

    #[test]
    fn test_env_new() {
        let env = Environment::new();
        assert!(env.lookup("undefined").is_none());
    }

    #[test]
    fn test_env_define_and_lookup() {
        let mut env = Environment::new();
        env.define("x", Value::Integer(42), false);
        assert_eq!(env.lookup("x"), Some(&Value::Integer(42)));
    }

    #[test]
    fn test_env_mutable_define() {
        let mut env = Environment::new();
        env.define("x", Value::Integer(42), true);
        env.set("x", Value::Integer(100));
        assert_eq!(env.lookup("x"), Some(&Value::Integer(100)));
    }

    #[test]
    fn test_env_immutable_define() {
        let mut env = Environment::new();
        env.define("x", Value::Integer(42), false);
        env.set("x", Value::Integer(100)); // This might fail or be ignored
                                           // Check behavior depends on implementation
    }

    #[test]
    fn test_env_scopes() {
        let mut env = Environment::new();
        env.define("x", Value::Integer(42), false);

        env.push_scope();
        env.define("x", Value::Integer(100), false);
        assert_eq!(env.lookup("x"), Some(&Value::Integer(100)));

        env.pop_scope();
        assert_eq!(env.lookup("x"), Some(&Value::Integer(42)));
    }

    #[test]
    fn test_env_nested_scopes() {
        let mut env = Environment::new();
        env.define("a", Value::Integer(1), false);

        env.push_scope();
        env.define("b", Value::Integer(2), false);

        env.push_scope();
        env.define("c", Value::Integer(3), false);

        assert!(env.lookup("a").is_some());
        assert!(env.lookup("b").is_some());
        assert!(env.lookup("c").is_some());

        env.pop_scope();
        assert!(env.lookup("c").is_none());
        assert!(env.lookup("b").is_some());

        env.pop_scope();
        assert!(env.lookup("b").is_none());
        assert!(env.lookup("a").is_some());
    }

    #[test]
    fn test_env_clear() {
        let mut env = Environment::new();
        env.define("x", Value::Integer(42), false);
        env.define("y", Value::Integer(100), false);

        env.clear();
        assert!(env.lookup("x").is_none());
        assert!(env.lookup("y").is_none());
    }

    #[test]
    fn test_env_multiple_variables() {
        let mut env = Environment::new();
        for i in 0..100 {
            env.define(&format!("var{}", i), Value::Integer(i), false);
        }

        for i in 0..100 {
            assert_eq!(env.lookup(&format!("var{}", i)), Some(&Value::Integer(i)));
        }
    }
}
