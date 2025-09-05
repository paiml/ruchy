//! Unit tests for the evaluation module
//! Target: 80% coverage of src/runtime/repl/evaluation.rs

#[cfg(test)]
mod evaluation_unit_tests {
    use ruchy::runtime::repl::Repl;
    use ruchy::runtime::Value;
    use ruchy::frontend::ast::{Expr, ExprKind, Literal, BinaryOp, UnaryOp, Pattern};
    use std::collections::HashMap;
    use std::time::{Duration, Instant};
    use anyhow::Result;

    // Mock binding provider for testing
    struct MockBindings {
        bindings: HashMap<String, Value>,
        scopes: Vec<HashMap<String, Value>>,
    }

    impl MockBindings {
        fn new() -> Self {
            Self {
                bindings: HashMap::new(),
                scopes: vec![HashMap::new()],
            }
        }

        fn with_binding(mut self, name: &str, value: Value) -> Self {
            self.bindings.insert(name.to_string(), value);
            self
        }
    }

    impl BindingProvider for MockBindings {
        fn get_binding(&self, name: &str) -> Option<Value> {
            // Check scopes in reverse order
            for scope in self.scopes.iter().rev() {
                if let Some(val) = scope.get(name) {
                    return Some(val.clone());
                }
            }
            self.bindings.get(name).cloned()
        }

        fn set_binding(&mut self, name: String, value: Value, _is_mutable: bool) -> Result<()> {
            if let Some(scope) = self.scopes.last_mut() {
                scope.insert(name, value);
            }
            Ok(())
        }

        fn push_scope(&mut self) {
            self.scopes.push(HashMap::new());
        }

        fn pop_scope(&mut self) {
            if self.scopes.len() > 1 {
                self.scopes.pop();
            }
        }
    }

    fn create_test_context<'a>(bindings: &'a mut MockBindings) -> EvaluationContext<'a> {
        EvaluationContext {
            bindings,
            functions: &HashMap::new(),
            config: &EvaluationConfig::default(),
        }
    }

    fn create_literal_expr(lit: Literal) -> Expr {
        Expr {
            kind: ExprKind::Literal(lit),
            span: Default::default(),
        }
    }

    // ==================== Literal Tests ====================

    #[test]
    fn test_evaluate_nil_literal() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = create_literal_expr(Literal::Nil);
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_evaluate_bool_literal() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = create_literal_expr(Literal::Bool(true));
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_evaluate_int_literal() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = create_literal_expr(Literal::Int(42));
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_evaluate_float_literal() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = create_literal_expr(Literal::Float(3.14));
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Float(3.14));
    }

    #[test]
    fn test_evaluate_string_literal() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = create_literal_expr(Literal::String("hello".to_string()));
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_evaluate_char_literal() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = create_literal_expr(Literal::Char('a'));
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Char('a'));
    }

    // ==================== Identifier Tests ====================

    #[test]
    fn test_evaluate_identifier() {
        let mut bindings = MockBindings::new().with_binding("x", Value::Int(10));
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Identifier("x".to_string()),
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Int(10));
    }

    #[test]
    fn test_evaluate_undefined_identifier() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Identifier("undefined".to_string()),
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0);
        assert!(result.is_err());
    }

    // ==================== Binary Operation Tests ====================

    #[test]
    fn test_binary_add_integers() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(create_literal_expr(Literal::Int(5))),
                op: BinaryOp::Add,
                right: Box::new(create_literal_expr(Literal::Int(3))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Int(8));
    }

    #[test]
    fn test_binary_subtract_floats() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(create_literal_expr(Literal::Float(10.5))),
                op: BinaryOp::Sub,
                right: Box::new(create_literal_expr(Literal::Float(3.5))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Float(7.0));
    }

    #[test]
    fn test_binary_multiply() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(create_literal_expr(Literal::Int(4))),
                op: BinaryOp::Mul,
                right: Box::new(create_literal_expr(Literal::Int(5))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Int(20));
    }

    #[test]
    fn test_binary_divide() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(create_literal_expr(Literal::Int(10))),
                op: BinaryOp::Div,
                right: Box::new(create_literal_expr(Literal::Int(2))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_binary_divide_by_zero() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(create_literal_expr(Literal::Int(10))),
                op: BinaryOp::Div,
                right: Box::new(create_literal_expr(Literal::Int(0))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_modulo() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(create_literal_expr(Literal::Int(10))),
                op: BinaryOp::Mod,
                right: Box::new(create_literal_expr(Literal::Int(3))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_binary_power() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(create_literal_expr(Literal::Int(2))),
                op: BinaryOp::Pow,
                right: Box::new(create_literal_expr(Literal::Int(3))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Int(8));
    }

    #[test]
    fn test_binary_comparison_equal() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(create_literal_expr(Literal::Int(5))),
                op: BinaryOp::Equal,
                right: Box::new(create_literal_expr(Literal::Int(5))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binary_comparison_less() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(create_literal_expr(Literal::Int(3))),
                op: BinaryOp::Less,
                right: Box::new(create_literal_expr(Literal::Int(5))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binary_logical_and_short_circuit() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(create_literal_expr(Literal::Bool(false))),
                op: BinaryOp::And,
                right: Box::new(Expr {
                    kind: ExprKind::Identifier("undefined".to_string()),
                    span: Default::default(),
                }),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        // Should short-circuit and not evaluate the undefined identifier
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_binary_logical_or_short_circuit() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(create_literal_expr(Literal::Bool(true))),
                op: BinaryOp::Or,
                right: Box::new(Expr {
                    kind: ExprKind::Identifier("undefined".to_string()),
                    span: Default::default(),
                }),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        // Should short-circuit and not evaluate the undefined identifier
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binary_null_coalesce() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(create_literal_expr(Literal::Nil)),
                op: BinaryOp::NullCoalesce,
                right: Box::new(create_literal_expr(Literal::Int(42))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_binary_bitwise_and() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(create_literal_expr(Literal::Int(5))),
                op: BinaryOp::BitwiseAnd,
                right: Box::new(create_literal_expr(Literal::Int(3))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Int(1)); // 0101 & 0011 = 0001
    }

    #[test]
    fn test_binary_bitwise_or() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(create_literal_expr(Literal::Int(5))),
                op: BinaryOp::BitwiseOr,
                right: Box::new(create_literal_expr(Literal::Int(3))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Int(7)); // 0101 | 0011 = 0111
    }

    #[test]
    fn test_binary_left_shift() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(create_literal_expr(Literal::Int(2))),
                op: BinaryOp::LeftShift,
                right: Box::new(create_literal_expr(Literal::Int(3))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Int(16)); // 2 << 3 = 16
    }

    // ==================== Unary Operation Tests ====================

    #[test]
    fn test_unary_not() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Unary {
                op: UnaryOp::Not,
                operand: Box::new(create_literal_expr(Literal::Bool(true))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_unary_negate_int() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Unary {
                op: UnaryOp::Neg,
                operand: Box::new(create_literal_expr(Literal::Int(42))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Int(-42));
    }

    #[test]
    fn test_unary_negate_float() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Unary {
                op: UnaryOp::Neg,
                operand: Box::new(create_literal_expr(Literal::Float(3.14))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Float(-3.14));
    }

    #[test]
    fn test_unary_bitwise_not() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Unary {
                op: UnaryOp::BitwiseNot,
                operand: Box::new(create_literal_expr(Literal::Int(5))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Int(!5));
    }

    // ==================== Control Flow Tests ====================

    #[test]
    fn test_if_expression_true_branch() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::If {
                condition: Box::new(create_literal_expr(Literal::Bool(true))),
                then_branch: Box::new(create_literal_expr(Literal::Int(10))),
                else_branch: Some(Box::new(create_literal_expr(Literal::Int(20)))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Int(10));
    }

    #[test]
    fn test_if_expression_false_branch() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::If {
                condition: Box::new(create_literal_expr(Literal::Bool(false))),
                then_branch: Box::new(create_literal_expr(Literal::Int(10))),
                else_branch: Some(Box::new(create_literal_expr(Literal::Int(20)))),
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Int(20));
    }

    #[test]
    fn test_if_expression_no_else() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::If {
                condition: Box::new(create_literal_expr(Literal::Bool(false))),
                then_branch: Box::new(create_literal_expr(Literal::Int(10))),
                else_branch: None,
            },
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Unit);
    }

    // ==================== Data Structure Tests ====================

    #[test]
    fn test_list_literal() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::List(vec![
                create_literal_expr(Literal::Int(1)),
                create_literal_expr(Literal::Int(2)),
                create_literal_expr(Literal::Int(3)),
            ]),
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ]));
    }

    #[test]
    fn test_tuple_literal() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = Expr {
            kind: ExprKind::Tuple(vec![
                create_literal_expr(Literal::Int(1)),
                create_literal_expr(Literal::String("hello".to_string())),
            ]),
            span: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(1);
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0).unwrap();
        assert_eq!(result, Value::Tuple(vec![
            Value::Int(1),
            Value::String("hello".to_string()),
        ]));
    }

    // ==================== Resource Bounds Tests ====================

    #[test]
    fn test_timeout_exceeded() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = create_literal_expr(Literal::Int(42));
        let deadline = Instant::now() - Duration::from_secs(1); // Already expired
        
        let result = evaluate_expression(&expr, &mut ctx, deadline, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timeout"));
    }

    #[test]
    fn test_max_depth_exceeded() {
        let mut bindings = MockBindings::new();
        let mut ctx = create_test_context(&mut bindings);
        let expr = create_literal_expr(Literal::Int(42));
        let deadline = Instant::now() + Duration::from_secs(1);
        
        // Set depth to exceed limit
        let result = evaluate_expression(&expr, &mut ctx, deadline, 1001);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("recursion depth"));
    }

    // ==================== Helper Function Tests ====================

    #[test]
    fn test_is_truthy() {
        assert!(is_truthy(&Value::Bool(true)));
        assert!(!is_truthy(&Value::Bool(false)));
        assert!(is_truthy(&Value::Int(1)));
        assert!(!is_truthy(&Value::Int(0)));
        assert!(is_truthy(&Value::Float(1.0)));
        assert!(!is_truthy(&Value::Float(0.0)));
        assert!(is_truthy(&Value::String("hello".to_string())));
        assert!(!is_truthy(&Value::String("".to_string())));
        assert!(!is_truthy(&Value::Nil));
        assert!(is_truthy(&Value::List(vec![Value::Int(1)])));
        assert!(!is_truthy(&Value::List(vec![])));
    }

    #[test]
    fn test_extract_iterable_items_list() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let items = extract_iterable_items(list).unwrap();
        assert_eq!(items, vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    }

    #[test]
    fn test_extract_iterable_items_range() {
        let range = Value::Range { start: 1, end: 4, inclusive: false };
        let items = extract_iterable_items(range).unwrap();
        assert_eq!(items, vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    }

    #[test]
    fn test_extract_iterable_items_range_inclusive() {
        let range = Value::Range { start: 1, end: 3, inclusive: true };
        let items = extract_iterable_items(range).unwrap();
        assert_eq!(items, vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]);
    }

    #[test]
    fn test_extract_iterable_items_string() {
        let string = Value::String("abc".to_string());
        let items = extract_iterable_items(string).unwrap();
        assert_eq!(items, vec![Value::Char('a'), Value::Char('b'), Value::Char('c')]);
    }

    #[test]
    fn test_extract_iterable_items_too_large_range() {
        let range = Value::Range { start: 0, end: 20000, inclusive: false };
        let result = extract_iterable_items(range);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    // ==================== Pattern Matching Tests ====================

    #[test]
    fn test_match_pattern_wildcard() {
        let pattern = Pattern::Wildcard;
        let value = Value::Int(42);
        let bindings = match_pattern(&pattern, &value);
        assert!(bindings.is_some());
        assert_eq!(bindings.unwrap(), vec![]);
    }

    #[test]
    fn test_match_pattern_literal() {
        let pattern = Pattern::Literal(Literal::Int(42));
        let value = Value::Int(42);
        let bindings = match_pattern(&pattern, &value);
        assert!(bindings.is_some());
        assert_eq!(bindings.unwrap(), vec![]);
    }

    #[test]
    fn test_match_pattern_literal_mismatch() {
        let pattern = Pattern::Literal(Literal::Int(42));
        let value = Value::Int(43);
        let bindings = match_pattern(&pattern, &value);
        assert!(bindings.is_none());
    }

    #[test]
    fn test_match_pattern_identifier() {
        let pattern = Pattern::Identifier("x".to_string());
        let value = Value::Int(42);
        let bindings = match_pattern(&pattern, &value);
        assert!(bindings.is_some());
        assert_eq!(bindings.unwrap(), vec![("x".to_string(), Value::Int(42))]);
    }

    #[test]
    fn test_match_pattern_list() {
        let pattern = Pattern::List(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
        ]);
        let value = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let bindings = match_pattern(&pattern, &value);
        assert!(bindings.is_some());
        assert_eq!(bindings.unwrap(), vec![
            ("a".to_string(), Value::Int(1)),
            ("b".to_string(), Value::Int(2)),
        ]);
    }

    #[test]
    fn test_match_pattern_tuple() {
        let pattern = Pattern::Tuple(vec![
            Pattern::Identifier("x".to_string()),
            Pattern::Wildcard,
        ]);
        let value = Value::Tuple(vec![Value::Int(42), Value::String("ignored".to_string())]);
        let bindings = match_pattern(&pattern, &value);
        assert!(bindings.is_some());
        assert_eq!(bindings.unwrap(), vec![("x".to_string(), Value::Int(42))]);
    }
}