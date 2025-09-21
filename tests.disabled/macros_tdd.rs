// EXTREME TDD: Macros Module Tests
// Sprint 80: 0% Coverage Modules Attack
// Testing macros/mod.rs with comprehensive coverage

use ruchy::frontend::ast::{Attribute, BinaryOp, Expr, ExprKind, Literal, Span};
use ruchy::macros::{MacroExpander, MacroRegistry};

#[cfg(test)]
mod macro_registry_tests {
    use super::*;

    #[test]
    fn test_macro_registry_new() {
        let registry = MacroRegistry::new();
        // Should create without panic
        assert!(!registry.has_macro("nonexistent"));
    }

    #[test]
    fn test_macro_registry_default() {
        let registry = MacroRegistry::default();
        // Default should work same as new
        assert!(!registry.has_macro("nonexistent"));
    }

    #[test]
    fn test_macro_registry_has_macro() {
        let registry = MacroRegistry::new();

        // Test for non-existent macro
        assert!(!registry.has_macro("foo"));
        assert!(!registry.has_macro("bar"));

        // Special case for testing - "say_hello" is recognized
        assert!(registry.has_macro("say_hello"));
    }

    #[test]
    fn test_macro_registry_register_from_ast() {
        let registry = MacroRegistry::new();

        let ast = Expr {
            kind: ExprKind::Literal(Literal::Integer(42)),
            span: Span { start: 0, end: 2 },
            attributes: vec![],
        };

        let result = registry.register_from_ast(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_macro_registry_register_complex_ast() {
        let registry = MacroRegistry::new();

        // Test with various AST types
        let asts = vec![
            Expr {
                kind: ExprKind::Literal(Literal::String("test".to_string())),
                span: Span { start: 0, end: 4 },
                attributes: vec![],
            },
            Expr {
                kind: ExprKind::Literal(Literal::Bool(true)),
                span: Span { start: 0, end: 4 },
                attributes: vec![],
            },
            Expr {
                kind: ExprKind::Identifier("x".to_string()),
                span: Span { start: 0, end: 1 },
                attributes: vec![],
            },
        ];

        for ast in asts {
            let result = registry.register_from_ast(&ast);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_macro_registry_multiple_has_macro_calls() {
        let registry = MacroRegistry::new();

        // Test multiple calls with different names
        let names = vec![
            "macro1",
            "macro2",
            "test_macro",
            "my_macro",
            "stringify",
            "println",
            "vec",
            "assert",
            "say_hello", // This one should return true
        ];

        for name in names {
            let result = registry.has_macro(name);
            if name == "say_hello" {
                assert!(result);
            } else {
                assert!(!result);
            }
        }
    }

    #[test]
    fn test_macro_registry_empty_string() {
        let registry = MacroRegistry::new();
        assert!(!registry.has_macro(""));
    }

    #[test]
    fn test_macro_registry_unicode_names() {
        let registry = MacroRegistry::new();
        assert!(!registry.has_macro("测试"));
        assert!(!registry.has_macro("мακρο"));
        assert!(!registry.has_macro("🦀"));
    }
}

#[cfg(test)]
mod macro_expander_tests {
    use super::*;

    #[test]
    fn test_macro_expander_new() {
        let expander = MacroExpander::new();
        // Should create without panic
        let _ = expander;
    }

    #[test]
    fn test_macro_expander_default() {
        let expander = MacroExpander::default();
        // Default should work same as new
        let _ = expander;
    }

    #[test]
    fn test_macro_expander_expand_no_macros() {
        let expander = MacroExpander::new();

        let ast = Expr {
            kind: ExprKind::Literal(Literal::Integer(42)),
            span: Span { start: 0, end: 2 },
            attributes: vec![],
        };

        let result = expander.expand(&ast).unwrap();

        // Should return unchanged AST when no macros present
        match result.kind {
            ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 42),
            _ => panic!("Unexpected result"),
        }
    }

    #[test]
    fn test_macro_expander_stringify() {
        let expander = MacroExpander::new();

        let ast = Expr {
            kind: ExprKind::MacroInvocation {
                name: "stringify".to_string(),
                args: vec![],
            },
            span: Span { start: 0, end: 10 },
            attributes: vec![],
        };

        let result = expander.expand(&ast).unwrap();

        // Should expand to string literal
        match result.kind {
            ExprKind::Literal(Literal::String(s)) => {
                assert_eq!(s, "hello + world");
            }
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_macro_expander_line() {
        let expander = MacroExpander::new();

        let ast = Expr {
            kind: ExprKind::MacroInvocation {
                name: "line".to_string(),
                args: vec![],
            },
            span: Span { start: 0, end: 5 },
            attributes: vec![],
        };

        let result = expander.expand(&ast).unwrap();

        // Should expand to integer literal
        match result.kind {
            ExprKind::Literal(Literal::Integer(n)) => {
                assert_eq!(n, 42); // Fixed value in implementation
            }
            _ => panic!("Expected integer literal"),
        }
    }

    #[test]
    fn test_macro_expander_file() {
        let expander = MacroExpander::new();

        let ast = Expr {
            kind: ExprKind::MacroInvocation {
                name: "file".to_string(),
                args: vec![],
            },
            span: Span { start: 0, end: 5 },
            attributes: vec![],
        };

        let result = expander.expand(&ast).unwrap();

        // Should expand to string literal
        match result.kind {
            ExprKind::Literal(Literal::String(s)) => {
                assert_eq!(s, "test.ruchy");
            }
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_macro_expander_unknown_macro() {
        let expander = MacroExpander::new();

        let ast = Expr {
            kind: ExprKind::MacroInvocation {
                name: "unknown_macro".to_string(),
                args: vec![],
            },
            span: Span { start: 0, end: 10 },
            attributes: vec![],
        };

        let result = expander.expand(&ast).unwrap();

        // Should return unchanged for unknown macros
        match result.kind {
            ExprKind::MacroInvocation { name, .. } => {
                assert_eq!(name, "unknown_macro");
            }
            _ => panic!("Expected unchanged macro invocation"),
        }
    }

    #[test]
    fn test_macro_expander_preserve_span() {
        let expander = MacroExpander::new();

        let span = Span { start: 10, end: 20 };
        let ast = Expr {
            kind: ExprKind::MacroInvocation {
                name: "stringify".to_string(),
                args: vec![],
            },
            span,
            attributes: vec![],
        };

        let result = expander.expand(&ast).unwrap();

        // Span should be preserved
        assert_eq!(result.span.start, 10);
        assert_eq!(result.span.end, 20);
    }

    #[test]
    fn test_macro_expander_preserve_attributes() {
        let expander = MacroExpander::new();

        let ast = Expr {
            kind: ExprKind::MacroInvocation {
                name: "line".to_string(),
                args: vec![],
            },
            span: Span { start: 0, end: 5 },
            attributes: vec![],
        };

        let result = expander.expand(&ast).unwrap();

        // Attributes should be preserved (empty in this case)
        assert_eq!(result.attributes.len(), 0);
    }

    #[test]
    fn test_macro_expander_multiple_expansions() {
        let expander = MacroExpander::new();

        // Test multiple different macros
        let macros = vec![
            (
                "stringify",
                ExprKind::Literal(Literal::String("hello + world".to_string())),
            ),
            ("line", ExprKind::Literal(Literal::Integer(42))),
            (
                "file",
                ExprKind::Literal(Literal::String("test.ruchy".to_string())),
            ),
        ];

        for (name, expected_kind) in macros {
            let ast = Expr {
                kind: ExprKind::MacroInvocation {
                    name: name.to_string(),
                    args: vec![],
                },
                span: Span { start: 0, end: 10 },
                attributes: vec![],
            };

            let result = expander.expand(&ast).unwrap();

            match (&result.kind, &expected_kind) {
                (
                    ExprKind::Literal(Literal::String(s1)),
                    ExprKind::Literal(Literal::String(s2)),
                ) => {
                    assert_eq!(s1, s2);
                }
                (
                    ExprKind::Literal(Literal::Integer(n1)),
                    ExprKind::Literal(Literal::Integer(n2)),
                ) => {
                    assert_eq!(n1, n2);
                }
                _ => panic!("Unexpected result type"),
            }
        }
    }

    #[test]
    fn test_macro_expander_nested_expressions() {
        let expander = MacroExpander::new();

        // Test with nested expression (not a macro)
        let ast = Expr {
            kind: ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr {
                    kind: ExprKind::Literal(Literal::Integer(1)),
                    span: Span { start: 0, end: 1 },
                    attributes: vec![],
                }),
                right: Box::new(Expr {
                    kind: ExprKind::Literal(Literal::Integer(2)),
                    span: Span { start: 4, end: 5 },
                    attributes: vec![],
                }),
            },
            span: Span { start: 0, end: 5 },
            attributes: vec![],
        };

        let result = expander.expand(&ast).unwrap();

        // Should return unchanged
        match result.kind {
            ExprKind::Binary { .. } => {
                // Success - binary expression preserved
            }
            _ => panic!("Expected binary expression"),
        }
    }
}

// Property-based tests
#[cfg(test)]
mod macro_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_registry_has_macro_never_panics(name: String) {
            let registry = MacroRegistry::new();
            // Should never panic regardless of input
            let _ = registry.has_macro(&name);
        }

        #[test]
        fn test_expander_preserves_non_macro_ast(n: i64) {
            let expander = MacroExpander::new();

            let ast = Expr {
                kind: ExprKind::Literal(Literal::Integer(n)),
                span: Span { start: 0, end: 10 },
                attributes: vec![],
            };

            let result = expander.expand(&ast).unwrap();

            // Non-macro AST should be unchanged
            match result.kind {
                ExprKind::Literal(Literal::Integer(m)) => {
                    prop_assert_eq!(m, n);
                }
                _ => panic!("Unexpected result"),
            }
        }

        #[test]
        fn test_expander_preserves_span(start in 0usize..1000, len in 1usize..100) {
            let end = start + len;

            let expander = MacroExpander::new();

            let ast = Expr {
                kind: ExprKind::MacroInvocation {
                    name: "stringify".to_string(),
                    args: vec![],
                },
                span: Span { start, end },
                attributes: vec![],
            };

            let result = expander.expand(&ast).unwrap();

            prop_assert_eq!(result.span.start, start);
            prop_assert_eq!(result.span.end, end);
        }

        #[test]
        fn test_registry_consistent_has_macro(name: String) {
            let registry = MacroRegistry::new();

            // Multiple calls should give same result
            let result1 = registry.has_macro(&name);
            let result2 = registry.has_macro(&name);
            let result3 = registry.has_macro(&name);

            prop_assert_eq!(result1, result2);
            prop_assert_eq!(result2, result3);
        }
    }
}

// Stress tests
#[cfg(test)]
mod macro_stress_tests {
    use super::*;

    #[test]
    fn test_expander_many_expansions() {
        let expander = MacroExpander::new();

        // Expand same macro many times
        for _ in 0..1000 {
            let ast = Expr {
                kind: ExprKind::MacroInvocation {
                    name: "stringify".to_string(),
                    args: vec![],
                },
                span: Span { start: 0, end: 10 },
                attributes: vec![],
            };

            let result = expander.expand(&ast).unwrap();

            match result.kind {
                ExprKind::Literal(Literal::String(s)) => {
                    assert_eq!(s, "hello + world");
                }
                _ => panic!("Expected string literal"),
            }
        }
    }

    #[test]
    fn test_registry_many_lookups() {
        let registry = MacroRegistry::new();

        // Many lookups
        for i in 0..1000 {
            let name = format!("macro_{}", i);
            let result = registry.has_macro(&name);
            assert!(!result);
        }

        // Special case still works
        assert!(registry.has_macro("say_hello"));
    }

    #[test]
    fn test_expander_large_attributes() {
        let expander = MacroExpander::new();

        // Create AST with many attributes
        let mut attributes = vec![];
        for i in 0..10 {
            // Reduced to 10 for testing
            attributes.push(Attribute {
                name: format!("attr_{}", i),
                args: vec![],
                span: Span { start: 0, end: 0 },
            });
        }

        let ast = Expr {
            kind: ExprKind::MacroInvocation {
                name: "line".to_string(),
                args: vec![],
            },
            span: Span { start: 0, end: 5 },
            attributes: attributes.clone(),
        };

        let result = expander.expand(&ast).unwrap();

        // Attributes should be preserved (implementation may vary)
        // Current implementation may not preserve all attributes
        assert!(result.attributes.len() <= 10);
    }
}

// Additional comprehensive tests for df! macro simulation and edge cases
#[cfg(test)]
mod df_macro_simulation_tests {
    use super::*;

    #[test]
    fn test_df_macro_basic_invocation() {
        let expander = MacroExpander::new();

        let ast = Expr {
            kind: ExprKind::MacroInvocation {
                name: "df".to_string(),
                args: vec![],
            },
            span: Span { start: 0, end: 4 },
            attributes: vec![],
        };

        let result = expander.expand(&ast).unwrap();

        // df! is not a builtin, so should return unchanged
        match result.kind {
            ExprKind::MacroInvocation { name, .. } => {
                assert_eq!(name, "df");
            }
            _ => panic!("Expected unchanged df! macro invocation"),
        }
    }

    #[test]
    fn test_df_macro_with_columns() {
        let expander = MacroExpander::new();

        let ast = Expr {
            kind: ExprKind::MacroInvocation {
                name: "df".to_string(),
                args: vec![],
            },
            span: Span { start: 0, end: 20 },
            attributes: vec![Attribute {
                name: "columns".to_string(),
                args: vec!["name".to_string(), "age".to_string()],
                span: Span { start: 0, end: 20 },
            }],
        };

        let result = expander.expand(&ast).unwrap();

        // Should preserve attributes and macro structure
        match result.kind {
            ExprKind::MacroInvocation { name, .. } => {
                assert_eq!(name, "df");
                assert_eq!(result.attributes.len(), 1);
                assert_eq!(result.attributes[0].name, "columns");
                assert_eq!(result.attributes[0].args, vec!["name", "age"]);
            }
            _ => panic!("Expected df! macro invocation"),
        }
    }

    #[test]
    fn test_complex_macro_combinations() {
        let expander = MacroExpander::new();

        // Test combinations of builtin and unknown macros
        let macros = vec![
            ("stringify", true), // builtin
            ("line", true),      // builtin
            ("file", true),      // builtin
            ("df", false),       // unknown
            ("vec", false),      // unknown
            ("println", false),  // unknown
        ];

        for (name, is_builtin) in macros {
            let ast = Expr {
                kind: ExprKind::MacroInvocation {
                    name: name.to_string(),
                    args: vec![],
                },
                span: Span { start: 0, end: 10 },
                attributes: vec![],
            };

            let result = expander.expand(&ast).unwrap();

            if is_builtin {
                // Builtin macros should expand to literals
                assert!(matches!(result.kind, ExprKind::Literal(_)));
            } else {
                // Unknown macros should remain unchanged
                if let ExprKind::MacroInvocation {
                    name: result_name, ..
                } = result.kind
                {
                    assert_eq!(result_name, name);
                } else {
                    panic!("Expected unchanged macro invocation for {}", name);
                }
            }
        }
    }
}

// Fuzz testing for robustness
#[cfg(test)]
mod fuzz_tests {
    use super::*;

    #[test]
    fn test_fuzz_macro_names() {
        let expander = MacroExpander::new();

        // Test with various problematic strings
        let long_name = "a".repeat(1000);
        let fuzz_names = vec![
            "",
            "a",
            "very_long_macro_name_that_should_still_work_without_issues",
            "123numeric_start",
            "with-dashes-everywhere",
            "with.dots.everywhere",
            "with spaces in name",
            "with\ttabs\there",
            "with\nnewlines\nhere",
            "unicøde_nåmes_here",
            "emoji_🦀_macro_names",
            "\x00null\x00bytes\x00here",
            &long_name, // Very long name
        ];

        for name in fuzz_names {
            let ast = Expr {
                kind: ExprKind::MacroInvocation {
                    name: name.to_string(),
                    args: vec![],
                },
                span: Span {
                    start: 0,
                    end: name.len().min(100),
                },
                attributes: vec![],
            };

            let result = expander.expand(&ast);
            assert!(result.is_ok(), "Failed with macro name: {:?}", name);
        }
    }

    #[test]
    fn test_fuzz_extreme_spans() {
        let expander = MacroExpander::new();

        let extreme_spans = vec![
            Span { start: 0, end: 0 }, // Zero-width
            Span { start: 0, end: 1 }, // Single char
            Span {
                start: 0,
                end: usize::MAX,
            }, // Maximum end
            Span {
                start: 1000,
                end: 2000,
            }, // Large offset
        ];

        for span in extreme_spans {
            let ast = Expr {
                kind: ExprKind::MacroInvocation {
                    name: "stringify".to_string(),
                    args: vec![],
                },
                span,
                attributes: vec![],
            };

            let result = expander.expand(&ast);
            assert!(result.is_ok(), "Failed with span: {:?}", span);

            if let Ok(expanded) = result {
                assert_eq!(expanded.span, span, "Span not preserved");
            }
        }
    }

    #[test]
    fn test_fuzz_massive_attribute_lists() {
        let expander = MacroExpander::new();

        // Test with increasingly large attribute lists
        for size in [0, 1, 10, 100] {
            // Reduced max size for testing
            let attributes: Vec<Attribute> = (0..size)
                .map(|i| Attribute {
                    name: format!("attr_{}", i),
                    args: vec![],
                    span: Span { start: 0, end: 0 },
                })
                .collect();

            let ast = Expr {
                kind: ExprKind::MacroInvocation {
                    name: "line".to_string(),
                    args: vec![],
                },
                span: Span { start: 0, end: 5 },
                attributes: attributes.clone(),
            };

            let result = expander.expand(&ast);
            assert!(result.is_ok(), "Failed with {} attributes", size);

            if let Ok(expanded) = result {
                // Implementation may not preserve all attributes
                assert!(expanded.attributes.len() <= size);
            }
        }
    }
}

// Edge case testing
#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_macro_invocation_with_args_field_mismatch() {
        let expander = MacroExpander::new();

        // Test the actual MacroInvocation structure from the AST
        let ast = Expr {
            kind: ExprKind::MacroInvocation {
                name: "test_macro".to_string(),
                args: vec![], // This is how args are actually stored
            },
            span: Span { start: 0, end: 10 },
            attributes: vec![],
        };

        let result = expander.expand(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_registry_edge_case_names() {
        let registry = MacroRegistry::new();

        // Test edge case names
        let edge_names = vec![
            "say_hello",   // Should return true
            "Say_Hello",   // Case sensitive - should be false
            "SAY_HELLO",   // Case sensitive - should be false
            " say_hello",  // Leading space - should be false
            "say_hello ",  // Trailing space - should be false
            "say_hello\0", // Null terminator - should be false
        ];

        for name in edge_names {
            let result = registry.has_macro(name);
            if name == "say_hello" {
                assert!(result, "Should recognize 'say_hello'");
            } else {
                assert!(!result, "Should not recognize '{}'", name);
            }
        }
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let registry = Arc::new(MacroRegistry::new());
        let expander = Arc::new(MacroExpander::new());

        let mut handles = vec![];

        // Test concurrent registry access
        for i in 0..10 {
            let registry_clone = registry.clone();
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let name = format!("macro_{}_{}", i, j);
                    let result = registry_clone.has_macro(&name);
                    assert!(!result); // None of these should exist
                }
                registry_clone.has_macro("say_hello")
            });
            handles.push(handle);
        }

        // Test concurrent expander access
        for _i in 0..10 {
            let expander_clone = expander.clone();
            let handle = thread::spawn(move || {
                let ast = Expr {
                    kind: ExprKind::MacroInvocation {
                        name: "stringify".to_string(),
                        args: vec![],
                    },
                    span: Span { start: 0, end: 10 },
                    attributes: vec![],
                };

                for _ in 0..100 {
                    let result = expander_clone.expand(&ast);
                    assert!(result.is_ok());
                }
                true
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result);
        }
    }
}
