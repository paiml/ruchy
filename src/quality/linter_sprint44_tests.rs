    use super::*;
    use crate::frontend::ast::*;

    // Helper function for creating comprehensive test scenarios
    fn create_complex_nested_expr() -> Expr {
        let inner_let = Expr::new(
            ExprKind::Let {
                name: "inner".to_string(),
                type_annotation: None,
                value: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(1, None)),
                    Span { start: 0, end: 1 },
                )),
                body: Box::new(Expr::new(
                    ExprKind::Identifier("inner".to_string()),
                    Span { start: 0, end: 1 },
                )),
                is_mutable: false,
                else_block: None,
            },
            Span { start: 0, end: 1 },
        );

        Expr::new(
            ExprKind::Let {
                name: "outer".to_string(),
                type_annotation: None,
                value: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(2, None)),
                    Span { start: 0, end: 1 },
                )),
                body: Box::new(inner_let),
                is_mutable: false,
                else_block: None,
            },
            Span { start: 0, end: 1 },
        )
    }

    // ========== SPRINT 44: Advanced Linter Tests (20 tests) ==========

    #[test]
    fn test_sprint_44_01_deeply_nested_scopes() {
        let linter = Linter::new();
        let complex_expr = create_complex_nested_expr();
        let result = linter.lint(&complex_expr, "nested code");
        assert!(result.is_ok());
        let _issues = result.expect("result should be Ok in test");
        // Should handle deeply nested scopes without panicking
        // Issues length is always >= 0 for usize
    }

    #[test]
    fn test_sprint_44_02_recursive_pattern_extraction() {
        let _linter = Linter::new();
        let mut scope = Scope::new();

        // Test deeply nested tuple patterns
        let nested_tuple = Pattern::Tuple(vec![
            Pattern::Tuple(vec![
                Pattern::Identifier("a".to_string()),
                Pattern::Identifier("b".to_string()),
            ]),
            Pattern::Identifier("c".to_string()),
        ]);

        Linter::extract_pattern_bindings(&nested_tuple, &mut scope);
        assert!(scope.is_defined("a"));
        assert!(scope.is_defined("b"));
        assert!(scope.is_defined("c"));
    }

    #[test]
    fn test_sprint_44_03_malformed_rule_strings() {
        let mut linter = Linter::new();

        // Test edge case rule strings
        let edge_cases = vec![
            "",
            ",,,,",
            "unknown,,,unused",
            "   ,  ,  ",
            "UPPERCASE",
            "mix3d_c4s3s",
            "\n\t\r",
        ];

        for rule_str in edge_cases {
            linter.set_rules(rule_str);
            // Should not panic and should have some reasonable state
            // Rules length is always >= 0 for usize, no need to check
        }
    }

    #[test]
    fn test_sprint_44_04_complexity_edge_cases() {
        let _linter = Linter::new();

        // Test empty block
        let empty_block = Expr::new(ExprKind::Block(vec![]), Span { start: 0, end: 1 });
        assert_eq!(Linter::calculate_complexity(&empty_block), 0);

        // Test single expression block
        let single_block = Expr::new(
            ExprKind::Block(vec![Expr::new(
                ExprKind::Literal(Literal::Integer(42, None)),
                Span { start: 0, end: 1 },
            )]),
            Span { start: 0, end: 1 },
        );
        assert_eq!(Linter::calculate_complexity(&single_block), 0);
    }

    #[test]
    fn test_sprint_44_05_variable_shadowing_multiple_levels() {
        let mut linter = Linter::new();
        linter.set_rules("shadowing");

        // Create 3-level nested shadowing
        let level3 = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(3, None)),
                    Span { start: 0, end: 1 },
                )),
                body: Box::new(Expr::new(
                    ExprKind::Identifier("x".to_string()),
                    Span { start: 0, end: 1 },
                )),
                is_mutable: false,
                else_block: None,
            },
            Span { start: 0, end: 1 },
        );

        let level2 = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(2, None)),
                    Span { start: 0, end: 1 },
                )),
                body: Box::new(level3),
                is_mutable: false,
                else_block: None,
            },
            Span { start: 0, end: 1 },
        );

        let level1 = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(1, None)),
                    Span { start: 0, end: 1 },
                )),
                body: Box::new(level2),
                is_mutable: false,
                else_block: None,
            },
            Span { start: 0, end: 1 },
        );

        let issues = linter
            .lint(&level1, "triple shadow")
            .expect("operation should succeed in test");
        // Should detect multiple shadowing instances
        let shadowing_count = issues.iter().filter(|i| i.rule == "shadowing").count();
        assert!(shadowing_count >= 1);
    }

    #[test]
    fn test_sprint_44_06_match_guard_variable_usage() {
        let linter = Linter::new();

        let guard_expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Gt,
                left: Box::new(Expr::new(
                    ExprKind::Identifier("bound_var".to_string()),
                    Span { start: 0, end: 1 },
                )),
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    Span { start: 0, end: 1 },
                )),
            },
            Span { start: 0, end: 1 },
        );

        let match_arm = MatchArm {
            pattern: Pattern::Identifier("bound_var".to_string()),
            guard: Some(Box::new(guard_expr)),
            body: Box::new(Expr::new(
                ExprKind::Identifier("bound_var".to_string()),
                Span { start: 0, end: 1 },
            )),
            span: Span { start: 0, end: 1 },
        };

        let match_expr = Expr::new(
            ExprKind::Match {
                expr: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(42, None)),
                    Span { start: 0, end: 1 },
                )),
                arms: vec![match_arm],
            },
            Span { start: 0, end: 1 },
        );

        let issues = linter
            .lint(&match_expr, "match with guard")
            .expect("operation should succeed in test");
        // Variable should be properly tracked through guard and body
        assert!(!issues
            .iter()
            .any(|i| i.name == "bound_var" && i.rule.contains("unused")));
    }

    #[test]
    fn test_sprint_44_07_lambda_parameter_patterns() {
        let linter = Linter::new();

        // Lambda with tuple destructuring parameter
        let tuple_param = Param {
            pattern: Pattern::Tuple(vec![
                Pattern::Identifier("x".to_string()),
                Pattern::Identifier("y".to_string()),
            ]),
            ty: Type {
                kind: TypeKind::Named("Tuple".to_string()),
                span: Span { start: 0, end: 1 },
            },
            span: Span { start: 0, end: 1 },
            is_mutable: false,
            default_value: None,
        };

        let lambda_body = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::new(
                    ExprKind::Identifier("x".to_string()),
                    Span { start: 0, end: 1 },
                )),
                right: Box::new(Expr::new(
                    ExprKind::Identifier("y".to_string()),
                    Span { start: 0, end: 1 },
                )),
            },
            Span { start: 0, end: 1 },
        );

        let lambda = Expr::new(
            ExprKind::Lambda {
                params: vec![tuple_param],
                body: Box::new(lambda_body),
            },
            Span { start: 0, end: 1 },
        );

        let issues = linter
            .lint(&lambda, "|(x, y)| x + y")
            .expect("operation should succeed in test");
        // Both x and y should be marked as used
        assert!(!issues
            .iter()
            .any(|i| (i.name == "x" || i.name == "y") && i.rule.contains("unused")));
    }

    #[test]
    fn test_sprint_44_08_string_interpolation_complex() {
        let mut linter = Linter::new();
        linter.set_rules("undefined");

        let complex_interpolation = Expr::new(
            ExprKind::StringInterpolation {
                parts: vec![
                    StringPart::Text("Value: ".to_string()),
                    StringPart::Expr(Box::new(Expr::new(
                        ExprKind::Binary {
                            op: BinaryOp::Add,
                            left: Box::new(Expr::new(
                                ExprKind::Identifier("undefined_a".to_string()),
                                Span { start: 0, end: 1 },
                            )),
                            right: Box::new(Expr::new(
                                ExprKind::Identifier("undefined_b".to_string()),
                                Span { start: 0, end: 1 },
                            )),
                        },
                        Span { start: 0, end: 1 },
                    ))),
                    StringPart::ExprWithFormat {
                        expr: Box::new(Expr::new(
                            ExprKind::Call {
                                func: Box::new(Expr::new(
                                    ExprKind::Identifier("undefined_func".to_string()),
                                    Span { start: 0, end: 1 },
                                )),
                                args: vec![],
                            },
                            Span { start: 0, end: 1 },
                        )),
                        format_spec: ":.2f".to_string(),
                    },
                ],
            },
            Span { start: 0, end: 1 },
        );

        let issues = linter
            .lint(&complex_interpolation, "complex f-string")
            .expect("operation should succeed in test");
        assert!(issues.iter().any(|i| i.name == "undefined_a"));
        assert!(issues.iter().any(|i| i.name == "undefined_b"));
        assert!(issues.iter().any(|i| i.name == "undefined_func"));
    }

    #[test]
    fn test_sprint_44_09_loop_pattern_destructuring() {
        let mut linter = Linter::new();
        linter.set_rules("unused");

        // For loop with struct pattern destructuring
        let struct_pattern = Pattern::Struct {
            name: "Point".to_string(),
            fields: vec![
                StructPatternField {
                    name: "x".to_string(),
                    pattern: None, // Shorthand: { x } means { x: x }
                },
                StructPatternField {
                    name: "y".to_string(),
                    pattern: Some(Pattern::Identifier("y_coord".to_string())),
                },
            ],
            has_rest: false,
        };

        let for_loop = Expr::new(
            ExprKind::For {
                label: None,
                var: "item".to_string(),
                pattern: Some(struct_pattern),
                iter: Box::new(Expr::new(
                    ExprKind::Identifier("points".to_string()),
                    Span { start: 0, end: 1 },
                )),
                body: Box::new(Expr::new(
                    ExprKind::Identifier("x".to_string()),
                    Span { start: 0, end: 1 },
                )), // Only use x
            },
            Span { start: 0, end: 1 },
        );

        let issues = linter
            .lint(&for_loop, "for {x, y: y_coord} in points { x }")
            .expect("operation should succeed in test");
        // x should be used, y_coord should be unused
        assert!(!issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("unused")));
        assert!(issues
            .iter()
            .any(|i| i.name == "y_coord" && i.rule.contains("unused")));
    }

    #[test]
    fn test_sprint_44_10_result_pattern_matching() {
        let linter = Linter::new();

        let ok_arm = MatchArm {
            pattern: Pattern::Ok(Box::new(Pattern::Identifier("success".to_string()))),
            guard: None,
            body: Box::new(Expr::new(
                ExprKind::Identifier("success".to_string()),
                Span { start: 0, end: 1 },
            )),
            span: Span { start: 0, end: 1 },
        };

        let err_arm = MatchArm {
            pattern: Pattern::Err(Box::new(Pattern::Identifier("error".to_string()))),
            guard: None,
            body: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(0, None)),
                Span { start: 0, end: 1 },
            )),
            span: Span { start: 0, end: 1 },
        };

        let result_match = Expr::new(
            ExprKind::Match {
                expr: Box::new(Expr::new(
                    ExprKind::Identifier("result".to_string()),
                    Span { start: 0, end: 1 },
                )),
                arms: vec![ok_arm, err_arm],
            },
            Span { start: 0, end: 1 },
        );

        let issues = linter
            .lint(
                &result_match,
                "match result { Ok(success) => success, Err(error) => 0 }",
            )
            .expect("operation should succeed in test");
        // success is used, error is unused
        assert!(!issues
            .iter()
            .any(|i| i.name == "success" && i.rule.contains("unused")));
        assert!(issues
            .iter()
            .any(|i| i.name == "error" && i.rule.contains("unused")));
    }

    #[test]
    fn test_sprint_44_11_auto_fix_preserves_semantics() {
        let linter = Linter::new();

        let test_cases = vec![
            ("let x = 42", "let x = 42"),       // No change for non-style issues
            ("let  x  =  42", "let x = 42"),    // Style fix
            ("fn test() { }", "fn test() { }"), // Preserve function structure
        ];

        for (input, _expected_pattern) in test_cases {
            let style_issue = LintIssue {
                line: 1,
                column: 1,
                severity: "warning".to_string(),
                rule: "style".to_string(),
                message: "spacing".to_string(),
                suggestion: "fix".to_string(),
                issue_type: "style".to_string(),
                name: "spacing".to_string(),
            };

            let fixed = linter
                .auto_fix(input, &[style_issue])
                .expect("operation should succeed in test");
            if input.contains("  ") {
                assert!(!fixed.contains("  "), "Double spaces should be fixed");
            }
            assert!(
                fixed.len() <= input.len(),
                "Fix should not increase length significantly"
            );
        }
    }

    #[test]
    fn test_sprint_44_12_concurrent_scope_modification() {
        let _linter = Linter::new();
        let mut scope = Scope::new();

        // Test multiple rapid modifications to scope
        for i in 0..100 {
            let var_name = format!("var_{i}");
            scope.define(var_name.clone(), i, i, VarType::Local);
            scope.mark_used(&var_name);
            assert!(scope.is_defined(&var_name));
        }

        assert_eq!(scope.variables.len(), 100);

        // All variables should be marked as used
        for info in scope.variables.values() {
            assert!(info.used);
        }
    }

    #[test]
    fn test_sprint_44_13_lint_issue_field_completeness() {
        let issue = LintIssue {
            line: 42,
            column: 13,
            severity: "critical".to_string(),
            rule: "custom_rule".to_string(),
            message: "Custom message with unicode: 🚀".to_string(),
            suggestion: "Suggestion with newlines\nand tabs\t".to_string(),
            issue_type: "custom_type".to_string(),
            name: "unicode_var_名前".to_string(),
        };

        // Test serialization handles all fields and special characters
        let json = serde_json::to_string(&issue).expect("operation should succeed in test");
        assert!(json.contains("42"));
        assert!(json.contains("13"));
        assert!(json.contains("critical"));
        assert!(json.contains("custom_rule"));
        assert!(json.contains("🚀"));
        assert!(json.contains("unicode_var_名前"));

        // Test round-trip
        let deserialized: LintIssue =
            serde_json::from_str(&json).expect("operation should succeed in test");
        assert_eq!(deserialized.line, 42);
        assert_eq!(deserialized.name, "unicode_var_名前");
    }

    #[test]
    fn test_sprint_44_14_scope_hierarchy_lookup() {
        let mut grandparent = Scope::new();
        grandparent.define("global".to_string(), 1, 1, VarType::Local);

        let mut parent = Scope::with_parent(grandparent);
        parent.define("parent_var".to_string(), 2, 1, VarType::Local);

        let mut child = Scope::with_parent(parent);
        child.define("child_var".to_string(), 3, 1, VarType::Local);

        // Test lookup through hierarchy
        assert!(child.is_defined("child_var"));
        assert!(child.is_defined("parent_var"));
        assert!(child.is_defined("global"));
        assert!(!child.is_defined("nonexistent"));

        // Test marking used propagates up
        assert!(child.mark_used("global"));
        assert!(child.mark_used("parent_var"));
        assert!(!child.mark_used("nonexistent"));
    }

    #[test]
    fn test_sprint_44_15_complexity_calculation_nested() {
        let _linter = Linter::new();

        // Create deeply nested if-else chain
        let mut nested_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            Span { start: 0, end: 1 },
        );

        for i in 0..5 {
            nested_expr = Expr::new(
                ExprKind::If {
                    condition: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Bool(true)),
                        Span { start: 0, end: 1 },
                    )),
                    then_branch: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(i, None)),
                        Span { start: 0, end: 1 },
                    )),
                    else_branch: Some(Box::new(nested_expr)),
                },
                Span { start: 0, end: 1 },
            );
        }

        let complexity = Linter::calculate_complexity(&nested_expr);
        assert_eq!(complexity, 5); // Each if adds 1 complexity
    }

    #[test]
    fn test_sprint_44_16_rule_filtering_comprehensive() {
        let mut linter = Linter::new();

        // Test all individual rules
        let rule_tests = vec![
            ("unused", 4), // UnusedVariable, Parameter, LoopVariable, MatchBinding
            ("undefined", 1),
            ("shadowing", 1),
            ("complexity", 1),
            ("style", 1),
            ("security", 1),
            ("performance", 1),
        ];

        for (rule_name, expected_count) in rule_tests {
            linter.set_rules(rule_name);
            assert_eq!(
                linter.rules.len(),
                expected_count,
                "Rule '{rule_name}' should add {expected_count} rules"
            );
        }

        // Test combination
        linter.set_rules("unused,undefined,complexity");
        assert_eq!(linter.rules.len(), 6); // 4 + 1 + 1
    }

    #[test]
    fn test_sprint_44_17_builtin_function_comprehensive() {
        let mut linter = Linter::new();
        linter.set_rules("undefined");

        let builtins = vec!["println", "print", "eprintln"];

        for builtin in builtins {
            let expr = Expr::new(
                ExprKind::Identifier(builtin.to_string()),
                Span { start: 0, end: 1 },
            );
            let issues = linter
                .lint(&expr, builtin)
                .expect("operation should succeed in test");
            assert_eq!(
                issues.len(),
                0,
                "Builtin '{builtin}' should not be flagged as undefined"
            );
        }

        // Test that non-builtins are still caught
        let non_builtin = Expr::new(
            ExprKind::Identifier("definitely_undefined".to_string()),
            Span { start: 0, end: 1 },
        );
        let issues = linter
            .lint(&non_builtin, "definitely_undefined")
            .expect("operation should succeed in test");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule, "undefined");
    }

    #[test]
    fn test_sprint_44_18_variable_type_usage_patterns() {
        let _linter = Linter::new();

        // Test all VarType variants have correct behavior
        let mut scope = Scope::new();

        scope.define("local".to_string(), 1, 1, VarType::Local);
        scope.define("param".to_string(), 1, 1, VarType::Parameter);
        scope.define("loop_var".to_string(), 1, 1, VarType::LoopVariable);
        scope.define("match_bind".to_string(), 1, 1, VarType::MatchBinding);

        // Initially all should be unused
        for info in scope.variables.values() {
            assert!(!info.used);
        }

        // Mark all as used
        assert!(scope.mark_used("local"));
        assert!(scope.mark_used("param"));
        assert!(scope.mark_used("loop_var"));
        assert!(scope.mark_used("match_bind"));

        // Now all should be used
        for info in scope.variables.values() {
            assert!(info.used);
        }
    }

    #[test]
    fn test_sprint_44_19_error_recovery_malformed_ast() {
        let linter = Linter::new();

        // Test with potentially problematic AST structures
        let empty_call = Expr::new(
            ExprKind::Call {
                func: Box::new(Expr::new(
                    ExprKind::Identifier("func".to_string()),
                    Span { start: 0, end: 1 },
                )),
                args: vec![],
            },
            Span { start: 0, end: 1 },
        );

        let result = linter.lint(&empty_call, "func()");
        assert!(result.is_ok()); // Should handle gracefully

        // Test with empty method call
        let empty_method = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(Expr::new(
                    ExprKind::Identifier("obj".to_string()),
                    Span { start: 0, end: 1 },
                )),
                method: String::new(), // Empty method name
                args: vec![],
            },
            Span { start: 0, end: 1 },
        );

        let result = linter.lint(&empty_method, "obj.()");
        assert!(result.is_ok()); // Should handle gracefully
    }

    #[test]
    fn test_sprint_44_20_performance_characteristics() {
        let linter = Linter::new();

        // Test that linter scales reasonably with input size
        let start_time = std::time::Instant::now();

        // Create a moderately complex expression tree
        let mut complex_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span { start: 0, end: 1 },
        );

        for i in 1..50 {
            complex_expr = Expr::new(
                ExprKind::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(complex_expr),
                    right: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(i, None)),
                        Span { start: 0, end: 1 },
                    )),
                },
                Span { start: 0, end: 1 },
            );
        }

        let result = linter.lint(&complex_expr, "large expression");
        let elapsed = start_time.elapsed();

        assert!(result.is_ok());
        assert!(
            elapsed.as_millis() < 1000,
            "Linting should complete quickly even for complex expressions"
        );

        // Test complexity calculation performance
        let complexity = Linter::calculate_complexity(&complex_expr);
        assert_eq!(complexity, 0); // Binary operations don't add complexity in current implementation
    }

    // ========== LINTER BUG FIX: Block Scope Tracking ==========

    /// RED phase: Test that reproduces block scope bug
    /// Bug: Linter incorrectly reports "unused variable" and "undefined variable"
    /// when variable is defined in one statement and used in next statement
    #[test]
    fn test_block_scope_variable_usage_across_statements() {
        let linter = Linter::new();

        // Create AST for: let x = 42\nx
        // This should parse as Block([Let { name: "x", value: 42, body: Unit }, Identifier("x")])
        let let_expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(42, None)),
                    Span { start: 88, end: 98 },
                )),
                body: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Unit),
                    Span { start: 96, end: 98 },
                )),
                is_mutable: false,
                else_block: None,
            },
            Span { start: 88, end: 98 },
        );

        let identifier_expr = Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span {
                start: 99,
                end: 100,
            },
        );

        let block = Expr::new(
            ExprKind::Block(vec![let_expr, identifier_expr]),
            Span { start: 0, end: 100 },
        );

        let result = linter.lint(&block, "let x = 42\nx");
        assert!(result.is_ok(), "Linting should succeed");

        let issues = result.expect("result should be Ok in test");

        // CRITICAL: Variable 'x' should NOT be reported as unused (it's used in next statement)
        let unused_x = issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("unused"));
        assert!(
            !unused_x,
            "Variable 'x' should NOT be reported as unused - it's used in the next statement. Issues: {issues:?}"
        );

        // CRITICAL: Variable 'x' should NOT be reported as undefined (it's defined in previous statement)
        let undefined_x = issues
            .iter()
            .any(|i| i.name == "x" && i.rule.contains("undefined"));
        assert!(
            !undefined_x,
            "Variable 'x' should NOT be reported as undefined - it's defined in previous statement. Issues: {issues:?}"
        );

        // The code should have ZERO issues
        assert_eq!(
            issues.len(),
            0,
            "Code should have zero linting issues, got: {issues:?}"
        );
    }

    /// Property test: Block scope should maintain variables across statements
    #[test]
    fn test_block_scope_multiple_variables() {
        let linter = Linter::new();

        // Create AST for: let x = 1\nlet y = 2\nx + y
        let let_x = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(1, None)),
                    Span { start: 0, end: 1 },
                )),
                body: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Unit),
                    Span { start: 0, end: 1 },
                )),
                is_mutable: false,
                else_block: None,
            },
            Span { start: 0, end: 1 },
        );

        let let_y = Expr::new(
            ExprKind::Let {
                name: "y".to_string(),
                type_annotation: None,
                value: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(2, None)),
                    Span { start: 0, end: 1 },
                )),
                body: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Unit),
                    Span { start: 0, end: 1 },
                )),
                is_mutable: false,
                else_block: None,
            },
            Span { start: 0, end: 1 },
        );

        let usage = Expr::new(
            ExprKind::Binary {
                op: crate::frontend::ast::BinaryOp::Add,
                left: Box::new(Expr::new(
                    ExprKind::Identifier("x".to_string()),
                    Span { start: 0, end: 1 },
                )),
                right: Box::new(Expr::new(
                    ExprKind::Identifier("y".to_string()),
                    Span { start: 0, end: 1 },
                )),
            },
            Span { start: 0, end: 1 },
        );

        let block = Expr::new(
            ExprKind::Block(vec![let_x, let_y, usage]),
            Span { start: 0, end: 10 },
        );

        let result = linter.lint(&block, "let x = 1\nlet y = 2\nx + y");
        assert!(result.is_ok());

        let issues = result.expect("result should be Ok in test");

        // Both variables should be used, no undefined/unused errors
        assert!(
            !issues.iter().any(|i| i.name == "x"),
            "Variable 'x' should have no issues"
        );
        assert!(
            !issues.iter().any(|i| i.name == "y"),
            "Variable 'y' should have no issues"
        );
    }
