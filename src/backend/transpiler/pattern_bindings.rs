//! Pattern binding extraction for transpiler
//!
//! This module provides functions to extract variable bindings from patterns,
//! used for destructuring assignments and pattern matching.

use crate::frontend::ast::Pattern;

/// Extracts all variable bindings from a pattern
///
/// This function recursively traverses a pattern and collects all variable names
/// that will be bound when the pattern matches.
///
/// # Examples
/// ```ignore
/// use ruchy::backend::transpiler::pattern_bindings::extract_pattern_bindings;
/// use ruchy::frontend::ast::Pattern;
///
/// let pattern = Pattern::Identifier("x".to_string());
/// let bindings = extract_pattern_bindings(&pattern);
/// assert_eq!(bindings, vec!["x".to_string()]);
/// ```
pub fn extract_pattern_bindings(pattern: &Pattern) -> Vec<String> {
    match pattern {
        Pattern::Identifier(name) => vec![name.clone()],
        Pattern::Tuple(patterns) | Pattern::List(patterns) => {
            patterns.iter().flat_map(extract_pattern_bindings).collect()
        }
        Pattern::TupleVariant { patterns, .. } => {
            patterns.iter().flat_map(extract_pattern_bindings).collect()
        }
        Pattern::Struct { fields, .. } => fields
            .iter()
            .flat_map(|field| {
                field
                    .pattern
                    .as_ref()
                    .map_or_else(|| vec![field.name.clone()], extract_pattern_bindings)
            })
            .collect(),
        Pattern::RestNamed(name) => vec![name.clone()],
        Pattern::Or(patterns) => {
            // For Or patterns, all branches must bind the same variables
            // Just extract from first pattern
            patterns
                .first()
                .map(extract_pattern_bindings)
                .unwrap_or_default()
        }
        Pattern::AtBinding { name, pattern } => {
            let mut bindings = vec![name.clone()];
            bindings.extend(extract_pattern_bindings(pattern));
            bindings
        }
        Pattern::WithDefault { pattern, .. } => extract_pattern_bindings(pattern),
        Pattern::Mut(pattern)
        | Pattern::Ok(pattern)
        | Pattern::Err(pattern)
        | Pattern::Some(pattern) => extract_pattern_bindings(pattern),
        Pattern::None => vec![],
        Pattern::Wildcard
        | Pattern::Literal(_)
        | Pattern::QualifiedName(_)
        | Pattern::Rest
        | Pattern::Range { .. } => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Literal, StructPatternField};

    // ==================== Identifier Pattern Tests ====================

    #[test]
    fn test_identifier_pattern() {
        let pattern = Pattern::Identifier("x".to_string());
        assert_eq!(extract_pattern_bindings(&pattern), vec!["x"]);
    }

    #[test]
    fn test_identifier_pattern_long_name() {
        let pattern = Pattern::Identifier("very_long_variable_name".to_string());
        assert_eq!(
            extract_pattern_bindings(&pattern),
            vec!["very_long_variable_name"]
        );
    }

    // ==================== Tuple Pattern Tests ====================

    #[test]
    fn test_tuple_pattern_single() {
        let pattern = Pattern::Tuple(vec![Pattern::Identifier("a".to_string())]);
        assert_eq!(extract_pattern_bindings(&pattern), vec!["a"]);
    }

    #[test]
    fn test_tuple_pattern_multiple() {
        let pattern = Pattern::Tuple(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
            Pattern::Identifier("c".to_string()),
        ]);
        assert_eq!(extract_pattern_bindings(&pattern), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_tuple_pattern_empty() {
        let pattern = Pattern::Tuple(vec![]);
        assert!(extract_pattern_bindings(&pattern).is_empty());
    }

    #[test]
    fn test_tuple_pattern_nested() {
        let pattern = Pattern::Tuple(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Tuple(vec![
                Pattern::Identifier("b".to_string()),
                Pattern::Identifier("c".to_string()),
            ]),
        ]);
        assert_eq!(extract_pattern_bindings(&pattern), vec!["a", "b", "c"]);
    }

    // ==================== List Pattern Tests ====================

    #[test]
    fn test_list_pattern_single() {
        let pattern = Pattern::List(vec![Pattern::Identifier("x".to_string())]);
        assert_eq!(extract_pattern_bindings(&pattern), vec!["x"]);
    }

    #[test]
    fn test_list_pattern_multiple() {
        let pattern = Pattern::List(vec![
            Pattern::Identifier("head".to_string()),
            Pattern::Identifier("tail".to_string()),
        ]);
        assert_eq!(extract_pattern_bindings(&pattern), vec!["head", "tail"]);
    }

    #[test]
    fn test_list_pattern_empty() {
        let pattern = Pattern::List(vec![]);
        assert!(extract_pattern_bindings(&pattern).is_empty());
    }

    // ==================== TupleVariant Pattern Tests ====================

    #[test]
    fn test_tuple_variant_pattern() {
        let pattern = Pattern::TupleVariant {
            path: vec!["Some".to_string()],
            patterns: vec![Pattern::Identifier("value".to_string())],
        };
        assert_eq!(extract_pattern_bindings(&pattern), vec!["value"]);
    }

    #[test]
    fn test_tuple_variant_pattern_multiple() {
        let pattern = Pattern::TupleVariant {
            path: vec!["Point".to_string()],
            patterns: vec![
                Pattern::Identifier("x".to_string()),
                Pattern::Identifier("y".to_string()),
            ],
        };
        assert_eq!(extract_pattern_bindings(&pattern), vec!["x", "y"]);
    }

    #[test]
    fn test_tuple_variant_pattern_empty() {
        let pattern = Pattern::TupleVariant {
            path: vec!["None".to_string()],
            patterns: vec![],
        };
        assert!(extract_pattern_bindings(&pattern).is_empty());
    }

    // ==================== Struct Pattern Tests ====================

    #[test]
    fn test_struct_pattern_simple_field() {
        let pattern = Pattern::Struct {
            name: "Point".to_string(),
            fields: vec![StructPatternField {
                name: "x".to_string(),
                pattern: None, // Shorthand: `{ x }` means bind to `x`
            }],
            has_rest: false,
        };
        assert_eq!(extract_pattern_bindings(&pattern), vec!["x"]);
    }

    #[test]
    fn test_struct_pattern_with_pattern() {
        let pattern = Pattern::Struct {
            name: "Point".to_string(),
            fields: vec![StructPatternField {
                name: "coord".to_string(),
                pattern: Some(Pattern::Identifier("my_x".to_string())),
            }],
            has_rest: false,
        };
        assert_eq!(extract_pattern_bindings(&pattern), vec!["my_x"]);
    }

    #[test]
    fn test_struct_pattern_multiple_fields() {
        let pattern = Pattern::Struct {
            name: "Point".to_string(),
            fields: vec![
                StructPatternField {
                    name: "x".to_string(),
                    pattern: None,
                },
                StructPatternField {
                    name: "y".to_string(),
                    pattern: None,
                },
            ],
            has_rest: false,
        };
        assert_eq!(extract_pattern_bindings(&pattern), vec!["x", "y"]);
    }

    #[test]
    fn test_struct_pattern_empty() {
        let pattern = Pattern::Struct {
            name: "Empty".to_string(),
            fields: vec![],
            has_rest: false,
        };
        assert!(extract_pattern_bindings(&pattern).is_empty());
    }

    // ==================== RestNamed Pattern Tests ====================

    #[test]
    fn test_rest_named_pattern() {
        let pattern = Pattern::RestNamed("rest".to_string());
        assert_eq!(extract_pattern_bindings(&pattern), vec!["rest"]);
    }

    // ==================== Or Pattern Tests ====================

    #[test]
    fn test_or_pattern_single() {
        let pattern = Pattern::Or(vec![Pattern::Identifier("x".to_string())]);
        assert_eq!(extract_pattern_bindings(&pattern), vec!["x"]);
    }

    #[test]
    fn test_or_pattern_multiple_same_bindings() {
        // In Or patterns, all branches must bind same variables
        // We only extract from first pattern
        let pattern = Pattern::Or(vec![
            Pattern::Identifier("x".to_string()),
            Pattern::Identifier("x".to_string()),
        ]);
        assert_eq!(extract_pattern_bindings(&pattern), vec!["x"]);
    }

    #[test]
    fn test_or_pattern_empty() {
        let pattern = Pattern::Or(vec![]);
        assert!(extract_pattern_bindings(&pattern).is_empty());
    }

    // ==================== AtBinding Pattern Tests ====================

    #[test]
    fn test_at_binding_simple() {
        let pattern = Pattern::AtBinding {
            name: "whole".to_string(),
            pattern: Box::new(Pattern::Identifier("inner".to_string())),
        };
        assert_eq!(extract_pattern_bindings(&pattern), vec!["whole", "inner"]);
    }

    #[test]
    fn test_at_binding_with_tuple() {
        let pattern = Pattern::AtBinding {
            name: "all".to_string(),
            pattern: Box::new(Pattern::Tuple(vec![
                Pattern::Identifier("a".to_string()),
                Pattern::Identifier("b".to_string()),
            ])),
        };
        assert_eq!(extract_pattern_bindings(&pattern), vec!["all", "a", "b"]);
    }

    // ==================== WithDefault Pattern Tests ====================

    #[test]
    fn test_with_default_pattern() {
        use crate::frontend::ast::{Expr, ExprKind, Literal, Span};
        let default_expr = Expr {
            kind: ExprKind::Literal(Literal::Integer(42, None)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let pattern = Pattern::WithDefault {
            pattern: Box::new(Pattern::Identifier("x".to_string())),
            default: Box::new(default_expr),
        };
        assert_eq!(extract_pattern_bindings(&pattern), vec!["x"]);
    }

    // ==================== Mut Pattern Tests ====================

    #[test]
    fn test_mut_pattern() {
        let pattern = Pattern::Mut(Box::new(Pattern::Identifier("x".to_string())));
        assert_eq!(extract_pattern_bindings(&pattern), vec!["x"]);
    }

    // ==================== Ok/Err/Some/None Pattern Tests ====================

    #[test]
    fn test_ok_pattern() {
        let pattern = Pattern::Ok(Box::new(Pattern::Identifier("value".to_string())));
        assert_eq!(extract_pattern_bindings(&pattern), vec!["value"]);
    }

    #[test]
    fn test_err_pattern() {
        let pattern = Pattern::Err(Box::new(Pattern::Identifier("error".to_string())));
        assert_eq!(extract_pattern_bindings(&pattern), vec!["error"]);
    }

    #[test]
    fn test_some_pattern() {
        let pattern = Pattern::Some(Box::new(Pattern::Identifier("val".to_string())));
        assert_eq!(extract_pattern_bindings(&pattern), vec!["val"]);
    }

    #[test]
    fn test_none_pattern() {
        let pattern = Pattern::None;
        assert!(extract_pattern_bindings(&pattern).is_empty());
    }

    // ==================== No Binding Patterns Tests ====================

    #[test]
    fn test_wildcard_pattern() {
        let pattern = Pattern::Wildcard;
        assert!(extract_pattern_bindings(&pattern).is_empty());
    }

    #[test]
    fn test_literal_pattern() {
        let pattern = Pattern::Literal(Literal::Integer(42, None));
        assert!(extract_pattern_bindings(&pattern).is_empty());
    }

    #[test]
    fn test_qualified_name_pattern() {
        let pattern = Pattern::QualifiedName(vec![
            "std".to_string(),
            "option".to_string(),
            "None".to_string(),
        ]);
        assert!(extract_pattern_bindings(&pattern).is_empty());
    }

    #[test]
    fn test_rest_pattern() {
        let pattern = Pattern::Rest;
        assert!(extract_pattern_bindings(&pattern).is_empty());
    }

    #[test]
    fn test_range_pattern() {
        use crate::frontend::ast::Literal;
        let pattern = Pattern::Range {
            start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
            end: Box::new(Pattern::Literal(Literal::Integer(10, None))),
            inclusive: true,
        };
        assert!(extract_pattern_bindings(&pattern).is_empty());
    }

    // ==================== Complex Nested Pattern Tests ====================

    #[test]
    fn test_deeply_nested_pattern() {
        // Pattern: (a, [b, c], Point { x, y })
        let pattern = Pattern::Tuple(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::List(vec![
                Pattern::Identifier("b".to_string()),
                Pattern::Identifier("c".to_string()),
            ]),
            Pattern::Struct {
                name: "Point".to_string(),
                fields: vec![
                    StructPatternField {
                        name: "x".to_string(),
                        pattern: None,
                    },
                    StructPatternField {
                        name: "y".to_string(),
                        pattern: None,
                    },
                ],
                has_rest: false,
            },
        ]);
        assert_eq!(
            extract_pattern_bindings(&pattern),
            vec!["a", "b", "c", "x", "y"]
        );
    }

    #[test]
    fn test_mixed_wildcards_and_bindings() {
        let pattern = Pattern::Tuple(vec![
            Pattern::Identifier("first".to_string()),
            Pattern::Wildcard,
            Pattern::Identifier("last".to_string()),
        ]);
        assert_eq!(extract_pattern_bindings(&pattern), vec!["first", "last"]);
    }

    #[test]
    fn test_ok_with_tuple() {
        let pattern = Pattern::Ok(Box::new(Pattern::Tuple(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
        ])));
        assert_eq!(extract_pattern_bindings(&pattern), vec!["a", "b"]);
    }
}
