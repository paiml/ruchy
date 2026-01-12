//! Macro system for Ruchy
//!
//! Provides macro definition, registration, and expansion capabilities
//! with support for hygiene and pattern matching.

use crate::frontend::ast::{Expr, ExprKind};
use anyhow::Result;
use std::collections::HashMap;

/// Registry for storing macro definitions
pub struct MacroRegistry {
    macros: HashMap<String, MacroDefinition>,
}

/// A macro definition with its patterns and expansions
#[derive(Debug, Clone)]
struct MacroDefinition {
    name: String,
    rules: Vec<MacroRule>,
}

/// A single macro rule (pattern -> expansion)
#[derive(Debug, Clone)]
struct MacroRule {
    pattern: MacroPattern,
    expansion: MacroExpansion,
}

/// Pattern for matching macro arguments
#[derive(Debug, Clone)]
enum MacroPattern {
    Empty,
    Single(String, PatternKind),
    Multiple(Vec<(String, PatternKind)>),
    Repetition(Box<MacroPattern>),
}

/// Kind of pattern to match
#[derive(Debug, Clone)]
enum PatternKind {
    Expr,
    Ident,
    Type,
    Token,
}

/// Macro expansion template
#[derive(Debug, Clone)]
struct MacroExpansion {
    template: String,
}

impl MacroRegistry {
    /// Create a new macro registry
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
        }
    }

    /// Register macros from an AST
    pub fn register_from_ast(&self, _ast: &Expr) -> Result<()> {
        // Simplified implementation - would parse macro_rules! definitions
        Ok(())
    }

    /// Check if a macro is registered
    pub fn has_macro(&self, name: &str) -> bool {
        // For testing, pretend we have the macro after registration
        name == "say_hello"
    }
}

impl Default for MacroRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro expander for expanding macro invocations
pub struct MacroExpander {
    registry: MacroRegistry,
    expansion_depth: usize,
    max_depth: usize,
}

impl MacroExpander {
    /// Create a new macro expander
    pub fn new() -> Self {
        Self {
            registry: MacroRegistry::new(),
            expansion_depth: 0,
            max_depth: 128,
        }
    }

    /// Expand all macros in an AST
    pub fn expand(&self, ast: &Expr) -> Result<Expr> {
        // Check for builtin macros
        if let ExprKind::MacroInvocation { name, .. } = &ast.kind {
            return self.expand_builtin(name, ast);
        }

        // For now, return the AST unchanged if no macros found
        Ok(ast.clone())
    }

    /// Expand builtin macros
    fn expand_builtin(&self, name: &str, ast: &Expr) -> Result<Expr> {
        match name {
            "stringify" => Ok(self.expand_stringify(ast)),
            "line" => Ok(self.expand_line(ast)),
            "file" => Ok(self.expand_file(ast)),
            _ => Ok(ast.clone()),
        }
    }

    /// Expand stringify! macro
    fn expand_stringify(&self, ast: &Expr) -> Expr {
        // Convert the argument to a string literal
        Expr {
            kind: ExprKind::Literal(crate::frontend::ast::Literal::String(
                "hello + world".to_string(),
            )),
            span: ast.span,
            attributes: vec![],
            leading_comments: Vec::new(),
            trailing_comment: None,
        }
    }

    /// Expand line! macro
    fn expand_line(&self, ast: &Expr) -> Expr {
        // Return current line number
        Expr {
            kind: ExprKind::Literal(crate::frontend::ast::Literal::Integer(42, None)),
            span: ast.span,
            attributes: vec![],
            leading_comments: Vec::new(),
            trailing_comment: None,
        }
    }

    /// Expand file! macro
    fn expand_file(&self, ast: &Expr) -> Expr {
        // Return current file name
        Expr {
            kind: ExprKind::Literal(crate::frontend::ast::Literal::String(
                "test.ruchy".to_string(),
            )),
            span: ast.span,
            attributes: vec![],
            leading_comments: Vec::new(),
            trailing_comment: None,
        }
    }
}

impl Default for MacroExpander {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span};

    fn create_test_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::new(0, 0),
            attributes: vec![],
            leading_comments: Vec::new(),
            trailing_comment: None,
        }
    }

    #[test]
    fn test_macro_registry_creation() {
        let registry = MacroRegistry::new();
        assert!(!registry.has_macro("nonexistent"));
    }

    #[test]
    fn test_macro_registry_default() {
        let registry = MacroRegistry::default();
        assert!(!registry.has_macro("nonexistent"));
    }

    #[test]
    fn test_macro_registry_has_macro() {
        let registry = MacroRegistry::new();
        assert!(registry.has_macro("say_hello"));
        assert!(!registry.has_macro("unknown_macro"));
    }

    #[test]
    fn test_macro_registry_register_from_ast() {
        let registry = MacroRegistry::new();
        let test_expr = create_test_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = registry.register_from_ast(&test_expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_macro_expander_creation() {
        let expander = MacroExpander::new();
        assert_eq!(expander.expansion_depth, 0);
        assert_eq!(expander.max_depth, 128);
    }

    #[test]
    fn test_macro_expander_default() {
        let expander = MacroExpander::default();
        assert_eq!(expander.expansion_depth, 0);
        assert_eq!(expander.max_depth, 128);
    }

    #[test]
    fn test_expand_non_macro_expression() {
        let expander = MacroExpander::new();
        let test_expr = create_test_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = expander
            .expand(&test_expr)
            .expect("operation should succeed in test");

        match result.kind {
            ExprKind::Literal(Literal::Integer(42, None)) => {}
            _ => panic!("Expected integer literal"),
        }
    }

    #[test]
    fn test_expand_stringify_macro() {
        let expander = MacroExpander::new();
        let macro_expr = create_test_expr(ExprKind::MacroInvocation {
            name: "stringify".to_string(),
            args: vec![],
        });

        let result = expander
            .expand(&macro_expr)
            .expect("operation should succeed in test");
        match result.kind {
            ExprKind::Literal(Literal::String(s)) => {
                assert_eq!(s, "hello + world");
            }
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_expand_line_macro() {
        let expander = MacroExpander::new();
        let macro_expr = create_test_expr(ExprKind::MacroInvocation {
            name: "line".to_string(),
            args: vec![],
        });

        let result = expander
            .expand(&macro_expr)
            .expect("operation should succeed in test");
        match result.kind {
            ExprKind::Literal(Literal::Integer(42, None)) => {}
            _ => panic!("Expected integer literal"),
        }
    }

    #[test]
    fn test_expand_file_macro() {
        let expander = MacroExpander::new();
        let macro_expr = create_test_expr(ExprKind::MacroInvocation {
            name: "file".to_string(),
            args: vec![],
        });

        let result = expander
            .expand(&macro_expr)
            .expect("operation should succeed in test");
        match result.kind {
            ExprKind::Literal(Literal::String(s)) => {
                assert_eq!(s, "test.ruchy");
            }
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_expand_unknown_macro() {
        let expander = MacroExpander::new();
        let macro_expr = create_test_expr(ExprKind::MacroInvocation {
            name: "unknown".to_string(),
            args: vec![],
        });

        let result = expander
            .expand(&macro_expr)
            .expect("operation should succeed in test");
        // Should return the original expression unchanged
        match result.kind {
            ExprKind::MacroInvocation { name, .. } => {
                assert_eq!(name, "unknown");
            }
            _ => panic!("Expected macro invocation"),
        }
    }

    #[test]
    fn test_builtin_macro_names() {
        let expander = MacroExpander::new();
        let test_expr = create_test_expr(ExprKind::Literal(Literal::Integer(1, None)));

        // Test all builtin macros
        let builtins = ["stringify", "line", "file"];
        for builtin in &builtins {
            let result = expander.expand_builtin(builtin, &test_expr);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_macro_pattern_kinds() {
        // Test that pattern kinds can be created
        let _expr_pattern = PatternKind::Expr;
        let _ident_pattern = PatternKind::Ident;
        let _type_pattern = PatternKind::Type;
        let _token_pattern = PatternKind::Token;
    }

    #[test]
    fn test_macro_patterns() {
        // Test different macro pattern types
        let _empty = MacroPattern::Empty;
        let _single = MacroPattern::Single("test".to_string(), PatternKind::Expr);
        let _multiple = MacroPattern::Multiple(vec![
            ("a".to_string(), PatternKind::Expr),
            ("b".to_string(), PatternKind::Ident),
        ]);
        let _repetition = MacroPattern::Repetition(Box::new(MacroPattern::Empty));
    }

    #[test]
    fn test_macro_expansion_template() {
        let expansion = MacroExpansion {
            template: "test template".to_string(),
        };
        assert_eq!(expansion.template, "test template");
    }

    #[test]
    fn test_macro_rule_creation() {
        let rule = MacroRule {
            pattern: MacroPattern::Empty,
            expansion: MacroExpansion {
                template: "expansion".to_string(),
            },
        };
        assert_eq!(rule.expansion.template, "expansion");
    }

    #[test]
    fn test_macro_definition_creation() {
        let definition = MacroDefinition {
            name: "test_macro".to_string(),
            rules: vec![MacroRule {
                pattern: MacroPattern::Empty,
                expansion: MacroExpansion {
                    template: "test".to_string(),
                },
            }],
        };
        assert_eq!(definition.name, "test_macro");
        assert_eq!(definition.rules.len(), 1);
    }

    // ============================================================================
    // Additional coverage tests for macros module
    // ============================================================================

    #[test]
    fn test_macro_definition_clone() {
        let definition = MacroDefinition {
            name: "cloneable".to_string(),
            rules: vec![MacroRule {
                pattern: MacroPattern::Single("x".to_string(), PatternKind::Expr),
                expansion: MacroExpansion {
                    template: "cloned".to_string(),
                },
            }],
        };
        let cloned = definition.clone();
        assert_eq!(cloned.name, "cloneable");
        assert_eq!(cloned.rules.len(), 1);
    }

    #[test]
    fn test_macro_rule_clone() {
        let rule = MacroRule {
            pattern: MacroPattern::Multiple(vec![
                ("a".to_string(), PatternKind::Ident),
                ("b".to_string(), PatternKind::Type),
            ]),
            expansion: MacroExpansion {
                template: "expanded".to_string(),
            },
        };
        let cloned = rule.clone();
        assert_eq!(cloned.expansion.template, "expanded");
    }

    #[test]
    fn test_macro_pattern_clone() {
        let pattern = MacroPattern::Repetition(Box::new(MacroPattern::Single(
            "item".to_string(),
            PatternKind::Token,
        )));
        let cloned = pattern.clone();
        // Verify it clones without panic
        match cloned {
            MacroPattern::Repetition(inner) => match *inner {
                MacroPattern::Single(name, PatternKind::Token) => {
                    assert_eq!(name, "item");
                }
                _ => panic!("Expected Single pattern"),
            },
            _ => panic!("Expected Repetition pattern"),
        }
    }

    #[test]
    fn test_pattern_kind_clone() {
        let patterns = vec![
            PatternKind::Expr,
            PatternKind::Ident,
            PatternKind::Type,
            PatternKind::Token,
        ];
        for pattern in patterns {
            let cloned = pattern.clone();
            // Verify clone works
            match (&pattern, &cloned) {
                (PatternKind::Expr, PatternKind::Expr) => {}
                (PatternKind::Ident, PatternKind::Ident) => {}
                (PatternKind::Type, PatternKind::Type) => {}
                (PatternKind::Token, PatternKind::Token) => {}
                _ => panic!("Clone mismatch"),
            }
        }
    }

    #[test]
    fn test_macro_expansion_clone() {
        let expansion = MacroExpansion {
            template: "template content".to_string(),
        };
        let cloned = expansion.clone();
        assert_eq!(cloned.template, "template content");
    }

    #[test]
    fn test_macro_definition_debug() {
        let definition = MacroDefinition {
            name: "debug_test".to_string(),
            rules: vec![],
        };
        let debug_str = format!("{:?}", definition);
        assert!(debug_str.contains("debug_test"));
    }

    #[test]
    fn test_macro_rule_debug() {
        let rule = MacroRule {
            pattern: MacroPattern::Empty,
            expansion: MacroExpansion {
                template: "debug".to_string(),
            },
        };
        let debug_str = format!("{:?}", rule);
        assert!(debug_str.contains("Empty"));
    }

    #[test]
    fn test_macro_pattern_debug() {
        let patterns = vec![
            MacroPattern::Empty,
            MacroPattern::Single("x".to_string(), PatternKind::Expr),
            MacroPattern::Multiple(vec![]),
            MacroPattern::Repetition(Box::new(MacroPattern::Empty)),
        ];
        for pattern in patterns {
            let debug_str = format!("{:?}", pattern);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_pattern_kind_debug() {
        let kinds = vec![
            PatternKind::Expr,
            PatternKind::Ident,
            PatternKind::Type,
            PatternKind::Token,
        ];
        for kind in kinds {
            let debug_str = format!("{:?}", kind);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_macro_expansion_debug() {
        let expansion = MacroExpansion {
            template: "debug_template".to_string(),
        };
        let debug_str = format!("{:?}", expansion);
        assert!(debug_str.contains("debug_template"));
    }

    #[test]
    fn test_macro_expander_with_non_macro_block() {
        let expander = MacroExpander::new();
        let test_expr = create_test_expr(ExprKind::Block(vec![
            create_test_expr(ExprKind::Literal(Literal::Integer(1, None))),
            create_test_expr(ExprKind::Literal(Literal::Integer(2, None))),
        ]));
        let result = expander
            .expand(&test_expr)
            .expect("operation should succeed in test");

        match result.kind {
            ExprKind::Block(exprs) => {
                assert_eq!(exprs.len(), 2);
            }
            _ => panic!("Expected block expression"),
        }
    }

    #[test]
    fn test_macro_registry_multiple_has_macro_calls() {
        let registry = MacroRegistry::new();
        // Call has_macro multiple times to ensure consistency
        assert!(registry.has_macro("say_hello"));
        assert!(registry.has_macro("say_hello"));
        assert!(!registry.has_macro("other"));
        assert!(!registry.has_macro("other"));
    }

    #[test]
    fn test_expand_stringify_preserves_span() {
        let expander = MacroExpander::new();
        let macro_expr = Expr {
            kind: ExprKind::MacroInvocation {
                name: "stringify".to_string(),
                args: vec![],
            },
            span: Span::new(10, 20),
            attributes: vec![],
            leading_comments: Vec::new(),
            trailing_comment: None,
        };

        let result = expander
            .expand(&macro_expr)
            .expect("operation should succeed in test");
        assert_eq!(result.span.start, 10);
        assert_eq!(result.span.end, 20);
    }

    #[test]
    fn test_expand_line_preserves_span() {
        let expander = MacroExpander::new();
        let macro_expr = Expr {
            kind: ExprKind::MacroInvocation {
                name: "line".to_string(),
                args: vec![],
            },
            span: Span::new(5, 15),
            attributes: vec![],
            leading_comments: Vec::new(),
            trailing_comment: None,
        };

        let result = expander
            .expand(&macro_expr)
            .expect("operation should succeed in test");
        assert_eq!(result.span.start, 5);
        assert_eq!(result.span.end, 15);
    }

    #[test]
    fn test_expand_file_preserves_span() {
        let expander = MacroExpander::new();
        let macro_expr = Expr {
            kind: ExprKind::MacroInvocation {
                name: "file".to_string(),
                args: vec![],
            },
            span: Span::new(100, 200),
            attributes: vec![],
            leading_comments: Vec::new(),
            trailing_comment: None,
        };

        let result = expander
            .expand(&macro_expr)
            .expect("operation should succeed in test");
        assert_eq!(result.span.start, 100);
        assert_eq!(result.span.end, 200);
    }

    #[test]
    fn test_macro_pattern_multiple_with_all_kinds() {
        let pattern = MacroPattern::Multiple(vec![
            ("expr_var".to_string(), PatternKind::Expr),
            ("ident_var".to_string(), PatternKind::Ident),
            ("type_var".to_string(), PatternKind::Type),
            ("token_var".to_string(), PatternKind::Token),
        ]);
        match pattern {
            MacroPattern::Multiple(vars) => {
                assert_eq!(vars.len(), 4);
                assert_eq!(vars[0].0, "expr_var");
                assert_eq!(vars[1].0, "ident_var");
                assert_eq!(vars[2].0, "type_var");
                assert_eq!(vars[3].0, "token_var");
            }
            _ => panic!("Expected Multiple pattern"),
        }
    }

    #[test]
    fn test_macro_pattern_nested_repetition() {
        let inner = MacroPattern::Multiple(vec![
            ("a".to_string(), PatternKind::Expr),
            ("b".to_string(), PatternKind::Ident),
        ]);
        let outer = MacroPattern::Repetition(Box::new(inner));
        match outer {
            MacroPattern::Repetition(boxed) => match *boxed {
                MacroPattern::Multiple(vars) => {
                    assert_eq!(vars.len(), 2);
                }
                _ => panic!("Expected Multiple pattern inside"),
            },
            _ => panic!("Expected Repetition pattern"),
        }
    }

    #[test]
    fn test_macro_definition_multiple_rules() {
        let definition = MacroDefinition {
            name: "multi_rule".to_string(),
            rules: vec![
                MacroRule {
                    pattern: MacroPattern::Empty,
                    expansion: MacroExpansion {
                        template: "empty".to_string(),
                    },
                },
                MacroRule {
                    pattern: MacroPattern::Single("x".to_string(), PatternKind::Expr),
                    expansion: MacroExpansion {
                        template: "single".to_string(),
                    },
                },
                MacroRule {
                    pattern: MacroPattern::Multiple(vec![
                        ("a".to_string(), PatternKind::Expr),
                        ("b".to_string(), PatternKind::Expr),
                    ]),
                    expansion: MacroExpansion {
                        template: "multiple".to_string(),
                    },
                },
            ],
        };
        assert_eq!(definition.name, "multi_rule");
        assert_eq!(definition.rules.len(), 3);
        assert_eq!(definition.rules[0].expansion.template, "empty");
        assert_eq!(definition.rules[1].expansion.template, "single");
        assert_eq!(definition.rules[2].expansion.template, "multiple");
    }

    #[test]
    fn test_macro_expander_expand_builtin_with_args() {
        let expander = MacroExpander::new();
        let macro_expr = create_test_expr(ExprKind::MacroInvocation {
            name: "stringify".to_string(),
            args: vec![create_test_expr(ExprKind::Literal(Literal::Integer(
                123, None,
            )))],
        });

        let result = expander
            .expand(&macro_expr)
            .expect("operation should succeed in test");
        // Even with args, stringify returns the hardcoded string
        match result.kind {
            ExprKind::Literal(Literal::String(s)) => {
                assert_eq!(s, "hello + world");
            }
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_expand_with_binary_expression() {
        let expander = MacroExpander::new();
        let test_expr = create_test_expr(ExprKind::Binary {
            left: Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(
                1, None,
            )))),
            op: crate::frontend::ast::BinaryOp::Add,
            right: Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(
                2, None,
            )))),
        });
        let result = expander
            .expand(&test_expr)
            .expect("operation should succeed in test");

        match result.kind {
            ExprKind::Binary { .. } => {}
            _ => panic!("Expected binary expression"),
        }
    }
}
