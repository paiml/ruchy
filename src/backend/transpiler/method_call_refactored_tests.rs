//! Comprehensive unit tests for method_call_refactored module
//! Target: Increase coverage from 15.58% to 80%+

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::frontend::parser::Parser;
    use crate::frontend::ast::{Expr, ExprKind};

    fn create_transpiler() -> Transpiler {
        Transpiler::new()
    }

    fn parse_and_extract_method_call(code: &str) -> (Box<Expr>, String, Vec<Expr>) {
        let mut parser = Parser::new(code);
        let expr = parser.parse_expr().expect("Failed to parse");
        
        if let ExprKind::MethodCall { receiver: object, method, args } = expr.kind {
            (object, method, args)
        } else {
            panic!("Not a method call expression");
        }
    }

    #[test]
    fn test_map_method() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("[1,2,3].map(|x| x * 2)");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("iter"));
        assert!(output.contains("map"));
        assert!(output.contains("collect"));
    }

    #[test]
    fn test_filter_method() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("[1,2,3].filter(|x| x > 1)");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("into_iter"));
        assert!(output.contains("filter"));
        assert!(output.contains("collect"));
    }

    #[test]
    fn test_reduce_method() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("[1,2,3].reduce(|a,b| a + b)");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("reduce") || output.contains("fold"));
    }

    #[test]
    fn test_hashmap_get() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("map.get(\"key\")");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("get"));
        assert!(output.contains("cloned"));
    }

    #[test]
    fn test_hashmap_contains_key() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("map.contains_key(\"key\")");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("contains_key"));
    }

    #[test]
    fn test_hashmap_items() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("map.items()");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("iter"));
        assert!(output.contains("clone"));
    }

    #[test]
    fn test_hashset_contains() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("set.contains(42)");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("contains"));
    }

    #[test]
    fn test_hashset_union() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("set1.union(set2)");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("union"));
        assert!(output.contains("HashSet"));
    }

    #[test]
    fn test_collection_push() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("vec.push(42)");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("push"));
    }

    #[test]
    fn test_collection_pop() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("vec.pop()");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("pop"));
    }

    #[test]
    fn test_collection_len() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("vec.len()");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("len"));
    }

    #[test]
    fn test_collection_is_empty() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("vec.is_empty()");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("is_empty"));
    }

    #[test]
    fn test_string_to_upper() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("\"hello\".to_upper()");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("to_uppercase"));
    }

    #[test]
    fn test_string_trim() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("\"  hello  \".trim()");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("trim"));
    }

    #[test]
    fn test_string_split() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("\"a,b,c\".split(\",\")");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("split"));
    }

    #[test]
    fn test_string_replace() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("\"hello\".replace(\"l\", \"r\")");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("replace"));
    }

    #[test]
    fn test_numeric_round() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("3.14.round()");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("round"));
    }

    #[test]
    fn test_numeric_abs() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("(-5).abs()");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("abs"));
    }

    #[test]
    #[ignore = "Flatten method not fully implemented"]
    fn test_advanced_flatten() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("[[1],[2]].flatten()");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("flatten"));
        assert!(output.contains("collect"));
    }

    #[test]
    #[ignore = "Unique method not fully implemented"]
    fn test_advanced_unique() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("[1,2,2,3].unique()");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("HashSet"));
        assert!(output.contains("collect"));
    }

    #[test]
    fn test_default_method_call() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("obj.custom_method(1, 2)");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("custom_method"));
    }

    #[test]
    fn test_iterator_any() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("[1,2,3].any(|x| x > 2)");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("any"));
    }

    #[test]
    fn test_iterator_all() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("[1,2,3].all(|x| x > 0)");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("all"));
    }

    #[test]
    fn test_iterator_find() {
        let transpiler = create_transpiler();
        let (obj, method, args) = parse_and_extract_method_call("[1,2,3].find(|x| x == 2)");
        
        let result = transpiler.transpile_method_call_refactored(&obj, &method, &args)
            .expect("Failed to transpile");
        
        let output = result.to_string();
        assert!(output.contains("find"));
    }
}