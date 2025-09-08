//! Unit tests for pattern matching module
//! Ensures comprehensive coverage of all pattern matching paths

#[cfg(test)]
mod tests {
    use crate::runtime::pattern_matching::{match_pattern, match_literal_pattern, values_equal};
    use crate::runtime::Value;
    use crate::frontend::ast::{Pattern, Literal};
    use std::collections::HashMap;

    #[test]
    fn test_wildcard_pattern() {
        let pattern = Pattern::Wildcard;
        let value = Value::Int(42);
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.len(), 0);
    }

    #[test]
    fn test_identifier_pattern() {
        let pattern = Pattern::Identifier("x".to_string());
        let value = Value::Int(42);
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "x");
        assert_eq!(bindings[0].1, Value::Int(42));
    }

    #[test]
    fn test_literal_pattern_integer() {
        let pattern = Pattern::Literal(Literal::Integer(42));
        let value = Value::Int(42);
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.len(), 0);
        
        // Non-matching case
        let value = Value::Int(43);
        assert!(match_pattern(&pattern, &value).is_none());
    }

    #[test]
    fn test_literal_pattern_float() {
        let pattern = Pattern::Literal(Literal::Float(std::f64::consts::PI));
        let value = Value::Float(std::f64::consts::PI);
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.len(), 0);
        
        // Non-matching case
        let value = Value::Float(2.71);
        assert!(match_pattern(&pattern, &value).is_none());
    }

    #[test]
    fn test_literal_pattern_string() {
        let pattern = Pattern::Literal(Literal::String("hello".to_string()));
        let value = Value::String("hello".to_string());
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.len(), 0);
        
        // Non-matching case
        let value = Value::String("world".to_string());
        assert!(match_pattern(&pattern, &value).is_none());
    }

    #[test]
    fn test_literal_pattern_boolean() {
        let pattern = Pattern::Literal(Literal::Bool(true));
        let value = Value::Bool(true);
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.len(), 0);
        
        // Non-matching case
        let value = Value::Bool(false);
        assert!(match_pattern(&pattern, &value).is_none());
    }

    #[test]
    fn test_literal_pattern_unit() {
        let pattern = Pattern::Literal(Literal::Unit);
        let value = Value::Unit;
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.len(), 0);
        
        // Non-matching case
        let value = Value::Int(42);
        assert!(match_pattern(&pattern, &value).is_none());
    }

    #[test]
    fn test_tuple_pattern() {
        let patterns = vec![
            Pattern::Identifier("x".to_string()),
            Pattern::Literal(Literal::Integer(42)),
            Pattern::Wildcard,
        ];
        let pattern = Pattern::Tuple(patterns);
        let value = Value::Tuple(vec![Value::Int(10), Value::Int(42), Value::String("ignored".to_string())]);
        
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "x");
        assert_eq!(bindings[0].1, Value::Int(10));
        
        // Wrong length
        let value = Value::Tuple(vec![Value::Int(10), Value::Int(42)]);
        assert!(match_pattern(&pattern, &value).is_none());
        
        // Wrong literal
        let value = Value::Tuple(vec![Value::Int(10), Value::Int(43), Value::String("ignored".to_string())]);
        assert!(match_pattern(&pattern, &value).is_none());
    }

    #[test]
    fn test_list_pattern() {
        let patterns = vec![
            Pattern::Literal(Literal::Integer(1)),
            Pattern::Identifier("x".to_string()),
            Pattern::Literal(Literal::Integer(3)),
        ];
        let pattern = Pattern::List(patterns);
        let value = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "x");
        assert_eq!(bindings[0].1, Value::Int(2));
        
        // Wrong length
        let value = Value::List(vec![Value::Int(1), Value::Int(2)]);
        assert!(match_pattern(&pattern, &value).is_none());
    }

    #[test]
    fn test_struct_pattern() {
        use crate::frontend::ast::StructPatternField;
        let fields = vec![
            StructPatternField { name: "name".to_string(), pattern: Some(Pattern::Identifier("n".to_string())) },
            StructPatternField { name: "age".to_string(), pattern: Some(Pattern::Literal(Literal::Integer(25))) },
        ];
        let pattern = Pattern::Struct { name: "Person".to_string(), fields, has_rest: false };
        
        let mut object_values = HashMap::new();
        object_values.insert("name".to_string(), Value::String("Alice".to_string()));
        object_values.insert("age".to_string(), Value::Int(25));
        object_values.insert("extra".to_string(), Value::Bool(true)); // Extra field should be ignored
        let value = Value::Object(object_values);
        
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "n");
        assert_eq!(bindings[0].1, Value::String("Alice".to_string()));
        
        // Missing field
        let mut incomplete_object = HashMap::new();
        incomplete_object.insert("name".to_string(), Value::String("Bob".to_string()));
        let value = Value::Object(incomplete_object);
        assert!(match_pattern(&pattern, &value).is_none());
        
        // Wrong field value
        let mut wrong_age_object = HashMap::new();
        wrong_age_object.insert("name".to_string(), Value::String("Charlie".to_string()));
        wrong_age_object.insert("age".to_string(), Value::Int(30));
        let value = Value::Object(wrong_age_object);
        assert!(match_pattern(&pattern, &value).is_none());
    }

    #[test]
    fn test_rest_pattern() {
        let pattern = Pattern::Rest;
        let value = Value::Int(42);
        // Rest pattern is typically used in destructuring contexts
        // For now it acts like wildcard
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.len(), 0);
    }

    #[test]
    fn test_type_mismatch() {
        // List pattern against non-list value
        let pattern = Pattern::List(vec![Pattern::Wildcard]);
        let value = Value::Int(42);
        assert!(match_pattern(&pattern, &value).is_none());
        
        // Tuple pattern against non-tuple value
        let pattern = Pattern::Tuple(vec![Pattern::Wildcard]);
        let value = Value::String("not a tuple".to_string());
        assert!(match_pattern(&pattern, &value).is_none());
        
        // Struct pattern against non-object value
        let pattern = Pattern::Struct { name: "Test".to_string(), fields: vec![], has_rest: false };
        let value = Value::List(vec![]);
        assert!(match_pattern(&pattern, &value).is_none());
    }

    #[test]
    fn test_nested_patterns() {
        // Tuple containing list
        let inner_pattern = Pattern::List(vec![
            Pattern::Literal(Literal::Integer(1)),
            Pattern::Identifier("x".to_string()),
        ]);
        let pattern = Pattern::Tuple(vec![
            Pattern::Identifier("first".to_string()),
            inner_pattern,
        ]);
        
        let value = Value::Tuple(vec![
            Value::String("hello".to_string()),
            Value::List(vec![Value::Int(1), Value::Int(2)]),
        ]);
        
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.len(), 2);
        assert!(bindings.iter().any(|(name, val)| name == "first" && *val == Value::String("hello".to_string())));
        assert!(bindings.iter().any(|(name, val)| name == "x" && *val == Value::Int(2)));
    }

    #[test]
    fn test_match_literal_pattern_direct() {
        // Test the match_literal_pattern function directly
        assert!(match_literal_pattern(&Value::Int(42), &Literal::Integer(42)));
        assert!(!match_literal_pattern(&Value::Int(42), &Literal::Integer(43)));
        
        assert!(match_literal_pattern(&Value::Float(std::f64::consts::PI), &Literal::Float(std::f64::consts::PI)));
        assert!(!match_literal_pattern(&Value::Float(std::f64::consts::PI), &Literal::Float(2.71)));
        
        assert!(match_literal_pattern(&Value::String("hello".to_string()), &Literal::String("hello".to_string())));
        assert!(!match_literal_pattern(&Value::String("hello".to_string()), &Literal::String("world".to_string())));
        
        assert!(match_literal_pattern(&Value::Bool(true), &Literal::Bool(true)));
        assert!(!match_literal_pattern(&Value::Bool(true), &Literal::Bool(false)));
        
        assert!(match_literal_pattern(&Value::Unit, &Literal::Unit));
        assert!(!match_literal_pattern(&Value::Int(0), &Literal::Unit));
    }

    #[test]
    fn test_values_equal_direct() {
        // Test the values_equal function directly
        assert!(values_equal(&Value::Int(42), &Value::Int(42)));
        assert!(!values_equal(&Value::Int(42), &Value::Int(43)));
        
        assert!(values_equal(&Value::Float(std::f64::consts::PI), &Value::Float(std::f64::consts::PI)));
        assert!(!values_equal(&Value::Float(std::f64::consts::PI), &Value::Float(2.71)));
        
        assert!(values_equal(&Value::String("hello".to_string()), &Value::String("hello".to_string())));
        assert!(!values_equal(&Value::String("hello".to_string()), &Value::String("world".to_string())));
        
        assert!(values_equal(&Value::Bool(true), &Value::Bool(true)));
        assert!(!values_equal(&Value::Bool(true), &Value::Bool(false)));
        
        assert!(values_equal(&Value::Unit, &Value::Unit));
        assert!(!values_equal(&Value::Unit, &Value::Int(0)));
        
        // Lists
        let list1 = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let list2 = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let list3 = Value::List(vec![Value::Int(1), Value::Int(3)]);
        assert!(values_equal(&list1, &list2));
        assert!(!values_equal(&list1, &list3));
        
        // Tuples
        let tuple1 = Value::Tuple(vec![Value::Int(1), Value::String("a".to_string())]);
        let tuple2 = Value::Tuple(vec![Value::Int(1), Value::String("a".to_string())]);
        let tuple3 = Value::Tuple(vec![Value::Int(1), Value::String("b".to_string())]);
        assert!(values_equal(&tuple1, &tuple2));
        assert!(!values_equal(&tuple1, &tuple3));
        
        // Objects
        let mut obj1 = HashMap::new();
        obj1.insert("a".to_string(), Value::Int(1));
        let mut obj2 = HashMap::new();
        obj2.insert("a".to_string(), Value::Int(1));
        let mut obj3 = HashMap::new();
        obj3.insert("a".to_string(), Value::Int(2));
        assert!(values_equal(&Value::Object(obj1.clone()), &Value::Object(obj2)));
        assert!(!values_equal(&Value::Object(obj1), &Value::Object(obj3)));
    }

    #[test]
    fn test_complex_nested_pattern() {
        use crate::frontend::ast::StructPatternField;
        // Struct containing tuple containing list
        let fields = vec![
            StructPatternField {
                name: "data".to_string(),
                pattern: Some(Pattern::Tuple(vec![
                    Pattern::Literal(Literal::String("type".to_string())),
                    Pattern::List(vec![
                        Pattern::Identifier("first".to_string()),
                        Pattern::Wildcard,
                        Pattern::Identifier("third".to_string()),
                    ]),
                ]))
            }
        ];
        let pattern = Pattern::Struct { name: "DataStruct".to_string(), fields, has_rest: false };
        
        let mut object_values = HashMap::new();
        object_values.insert("data".to_string(), Value::Tuple(vec![
            Value::String("type".to_string()),
            Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ]),
        ]));
        let value = Value::Object(object_values);
        
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.len(), 2);
        assert!(bindings.iter().any(|(name, val)| name == "first" && *val == Value::Int(1)));
        assert!(bindings.iter().any(|(name, val)| name == "third" && *val == Value::Int(3)));
    }

    #[test]
    fn test_option_pattern() {
        // Some variant
        let pattern = Pattern::Some(Box::new(Pattern::Identifier("x".to_string())));
        let value = Value::EnumVariant { 
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![Value::Int(42)])
        };
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "x");
        assert_eq!(bindings[0].1, Value::Int(42));
        
        // None variant
        let pattern = Pattern::None;
        let value = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "None".to_string(),
            data: None
        };
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.len(), 0);
    }

    #[test]
    fn test_range_value_equality() {
        let range1 = Value::Range { start: 1, end: 5, inclusive: false };
        let range2 = Value::Range { start: 1, end: 5, inclusive: false };
        let range3 = Value::Range { start: 1, end: 6, inclusive: false };
        let range4 = Value::Range { start: 1, end: 5, inclusive: true };
        assert!(values_equal(&range1, &range2));
        assert!(!values_equal(&range1, &range3));
        assert!(!values_equal(&range1, &range4));
    }

    #[test]
    fn test_function_value_non_equality() {
        use crate::frontend::ast::{Expr, ExprKind, Span};
        // Functions are never equal (they don't have structural equality)
        let expr = Expr::new(ExprKind::Literal(Literal::Integer(42)), Span { start: 0, end: 0 });
        let func1 = Value::Function { 
            name: "test".to_string(),
            params: vec![],
            body: Box::new(expr.clone())
        };
        let func2 = Value::Function {
            name: "test".to_string(),
            params: vec![],
            body: Box::new(expr)
        };
        assert!(!values_equal(&func1, &func2));
    }
}