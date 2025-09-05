//! Comprehensive unit tests for patterns module
//! Target: Increase coverage from 33.33% to 80%+

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::frontend::ast::{Pattern, Literal};
    use proc_macro2::TokenStream;
    use quote::quote;

    fn create_transpiler() -> Transpiler {
        Transpiler::new()
    }

    #[test]
    fn test_wildcard_pattern() {
        let transpiler = create_transpiler();
        let pattern = Pattern::Wildcard;
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert_eq!(output, "_");
    }

    #[test]
    fn test_identifier_pattern() {
        let transpiler = create_transpiler();
        let pattern = Pattern::Identifier("x".to_string());
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert_eq!(output, "x");
    }

    #[test]
    fn test_literal_pattern_integer() {
        let transpiler = create_transpiler();
        let pattern = Pattern::Literal(Literal::Integer(42));
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("42"));
    }

    #[test]
    fn test_literal_pattern_string() {
        let transpiler = create_transpiler();
        let pattern = Pattern::Literal(Literal::String("hello".to_string()));
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("hello"));
    }

    #[test]
    fn test_literal_pattern_bool() {
        let transpiler = create_transpiler();
        let pattern = Pattern::Literal(Literal::Bool(true));
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert_eq!(output, "true");
    }

    #[test]
    fn test_tuple_pattern_simple() {
        let transpiler = create_transpiler();
        let pattern = Pattern::Tuple(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
        ]);
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("("));
        assert!(output.contains("a"));
        assert!(output.contains("b"));
        assert!(output.contains(")"));
    }

    #[test]
    fn test_tuple_pattern_nested() {
        let transpiler = create_transpiler();
        let pattern = Pattern::Tuple(vec![
            Pattern::Tuple(vec![
                Pattern::Identifier("a".to_string()),
                Pattern::Identifier("b".to_string()),
            ]),
            Pattern::Identifier("c".to_string()),
        ]);
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("(("));
        assert!(output.contains("a"));
        assert!(output.contains("b"));
        assert!(output.contains("c"));
    }

    #[test]
    fn test_list_pattern_empty() {
        let transpiler = create_transpiler();
        let pattern = Pattern::List(vec![]);
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert_eq!(output, "[]");
    }

    #[test]
    fn test_list_pattern_simple() {
        let transpiler = create_transpiler();
        let pattern = Pattern::List(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
            Pattern::Identifier("c".to_string()),
        ]);
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("["));
        assert!(output.contains("a"));
        assert!(output.contains("b"));
        assert!(output.contains("c"));
        assert!(output.contains("]"));
    }

    #[test]
    fn test_list_pattern_with_rest() {
        let transpiler = create_transpiler();
        let pattern = Pattern::List(vec![
            Pattern::Identifier("head".to_string()),
            Pattern::Rest,
        ]);
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("["));
        assert!(output.contains("head"));
        assert!(output.contains(".."));
        assert!(output.contains("]"));
    }

    #[test]
    fn test_list_pattern_with_named_rest() {
        let transpiler = create_transpiler();
        let pattern = Pattern::List(vec![
            Pattern::Identifier("head".to_string()),
            Pattern::RestNamed("tail".to_string()),
        ]);
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("["));
        assert!(output.contains("head"));
        assert!(output.contains(".."));
        assert!(output.contains("tail"));
        assert!(output.contains("]"));
    }

    #[test]
    fn test_struct_pattern_simple() {
        let transpiler = create_transpiler();
        let pattern = Pattern::Struct {
            name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), Pattern::Identifier("x_val".to_string())),
                ("y".to_string(), Pattern::Identifier("y_val".to_string())),
            ],
            has_rest: false,
        };
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("Point"));
        assert!(output.contains("{"));
        assert!(output.contains("x"));
        assert!(output.contains("x_val"));
        assert!(output.contains("y"));
        assert!(output.contains("y_val"));
        assert!(output.contains("}"));
    }

    #[test]
    fn test_struct_pattern_with_rest() {
        let transpiler = create_transpiler();
        let pattern = Pattern::Struct {
            name: "Config".to_string(),
            fields: vec![
                ("debug".to_string(), Pattern::Literal(Literal::Bool(true))),
            ],
            has_rest: true,
        };
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("Config"));
        assert!(output.contains("debug"));
        assert!(output.contains("true"));
        assert!(output.contains(".."));
    }

    #[test]
    fn test_enum_pattern() {
        let transpiler = create_transpiler();
        let pattern = Pattern::Enum {
            name: "Option".to_string(),
            variant: "Some".to_string(),
            fields: Some(vec![Pattern::Identifier("val".to_string())]),
        };
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("Option"));
        assert!(output.contains("Some"));
        assert!(output.contains("val"));
    }

    #[test]
    fn test_enum_pattern_no_fields() {
        let transpiler = create_transpiler();
        let pattern = Pattern::Enum {
            name: "Option".to_string(),
            variant: "None".to_string(),
            fields: None,
        };
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("Option"));
        assert!(output.contains("None"));
    }

    #[test]
    fn test_qualified_name_pattern() {
        let transpiler = create_transpiler();
        let pattern = Pattern::QualifiedName(vec![
            "std".to_string(),
            "cmp".to_string(),
            "Ordering".to_string(),
            "Less".to_string(),
        ]);
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("std"));
        assert!(output.contains("cmp"));
        assert!(output.contains("Ordering"));
        assert!(output.contains("Less"));
    }

    #[test]
    fn test_range_pattern() {
        let transpiler = create_transpiler();
        let pattern = Pattern::Range {
            start: Box::new(Pattern::Literal(Literal::Integer(0))),
            end: Box::new(Pattern::Literal(Literal::Integer(10))),
            inclusive: false,
        };
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("0"));
        assert!(output.contains("10"));
    }

    #[test]
    fn test_range_pattern_inclusive() {
        let transpiler = create_transpiler();
        let pattern = Pattern::Range {
            start: Box::new(Pattern::Literal(Literal::Integer(1))),
            end: Box::new(Pattern::Literal(Literal::Integer(5))),
            inclusive: true,
        };
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("1"));
        assert!(output.contains("5"));
        assert!(output.contains("="));
    }

    #[test]
    fn test_or_pattern() {
        let transpiler = create_transpiler();
        let pattern = Pattern::Or(vec![
            Pattern::Literal(Literal::Integer(1)),
            Pattern::Literal(Literal::Integer(2)),
            Pattern::Literal(Literal::Integer(3)),
        ]);
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("1"));
        assert!(output.contains("|"));
        assert!(output.contains("2"));
        assert!(output.contains("|"));
        assert!(output.contains("3"));
    }

    #[test]
    fn test_complex_nested_pattern() {
        let transpiler = create_transpiler();
        let pattern = Pattern::Tuple(vec![
            Pattern::List(vec![
                Pattern::Identifier("first".to_string()),
                Pattern::Rest,
            ]),
            Pattern::Struct {
                name: "Config".to_string(),
                fields: vec![
                    ("enabled".to_string(), Pattern::Literal(Literal::Bool(true))),
                ],
                has_rest: true,
            },
        ]);
        
        let result = transpiler.transpile_pattern(&pattern)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("("));
        assert!(output.contains("["));
        assert!(output.contains("first"));
        assert!(output.contains("Config"));
        assert!(output.contains("enabled"));
    }
}