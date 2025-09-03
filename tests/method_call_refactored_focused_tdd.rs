//! Focused TDD test suite for method_call_refactored.rs
//! Target: Transform 0% → 70%+ coverage via public API testing
//! Approach: Test all method categories through transpile_method_call_refactored

use ruchy::backend::Transpiler;
use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span};

/// Helper: Create test transpiler instance
fn create_test_transpiler() -> Transpiler {
    Transpiler::new()
}

/// Helper: Create integer literal expression
fn create_integer_literal(value: i64) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::Integer(value)),
        span: Span::new(0, value.to_string().len()),
        attributes: Vec::new(),
    }
}

/// Helper: Create string literal expression
fn create_string_literal(value: &str) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::String(value.to_string())),
        span: Span::new(0, value.len() + 2),
        attributes: Vec::new(),
    }
}

/// Helper: Create identifier expression
fn create_identifier(name: &str) -> Expr {
    Expr {
        kind: ExprKind::Identifier(name.to_string()),
        span: Span::new(0, name.len()),
        attributes: Vec::new(),
    }
}

// ========== ITERATOR METHODS TESTS ==========

#[test]
fn test_transpile_method_call_map() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("items");
    let args = vec![create_identifier("closure")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "map", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("items"));
    assert!(code.contains("iter"));
    assert!(code.contains("map"));
}

#[test]
fn test_transpile_method_call_filter() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("data");
    let args = vec![create_identifier("predicate")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "filter", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("data"));
    assert!(code.contains("filter"));
}

#[test]
fn test_transpile_method_call_reduce() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("numbers");
    let args = vec![create_identifier("accumulator")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "reduce", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("numbers"));
    assert!(code.contains("reduce"));
}

#[test]
fn test_transpile_method_call_fold() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("values");
    let args = vec![create_integer_literal(0), create_identifier("func")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "fold", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("values"));
    assert!(code.contains("fold"));
}

#[test]
fn test_transpile_method_call_any() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("checks");
    let args = vec![create_identifier("condition")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "any", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("checks"));
    assert!(code.contains("any"));
}

#[test]
fn test_transpile_method_call_all() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("requirements");
    let args = vec![create_identifier("validator")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "all", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("requirements"));
    assert!(code.contains("all"));
}

#[test]
fn test_transpile_method_call_find() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("elements");
    let args = vec![create_identifier("matcher")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "find", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("elements"));
    assert!(code.contains("find"));
}

// ========== HASHMAP METHODS TESTS ==========

#[test]
fn test_transpile_method_call_hashmap_get() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("map");
    let args = vec![create_string_literal("key")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "get", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("map"));
    assert!(code.contains("get"));
}

#[test]
fn test_transpile_method_call_hashmap_contains_key() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("dict");
    let args = vec![create_string_literal("search_key")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "contains_key", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("dict"));
    assert!(code.contains("contains_key"));
}

#[test]
fn test_transpile_method_call_hashmap_keys() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("storage");
    let args = vec![];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "keys", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("storage"));
    assert!(code.contains("keys"));
}

#[test]
fn test_transpile_method_call_hashmap_values() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("cache");
    let args = vec![];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "values", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("cache"));
    assert!(code.contains("values"));
}

#[test]
fn test_transpile_method_call_hashmap_items() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("data");
    let args = vec![];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "items", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("data"));
    // The items method actually uses .iter().map() instead of .items()
    assert!(code.contains("iter"));
}

// ========== HASHSET METHODS TESTS ==========

#[test]
fn test_transpile_method_call_hashset_contains() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("set");
    let args = vec![create_string_literal("item")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "contains", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("set"));
    assert!(code.contains("contains"));
}

#[test]
fn test_transpile_method_call_hashset_union() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("set1");
    let args = vec![create_identifier("set2")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "union", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("set1"));
    assert!(code.contains("union"));
}

#[test]
fn test_transpile_method_call_hashset_intersection() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("base_set");
    let args = vec![create_identifier("other_set")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "intersection", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("base_set"));
    assert!(code.contains("intersection"));
}

#[test]
fn test_transpile_method_call_hashset_difference() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("first");
    let args = vec![create_identifier("second")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "difference", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("first"));
    assert!(code.contains("difference"));
}

// ========== COLLECTION MUTATOR METHODS TESTS ==========

#[test]
fn test_transpile_method_call_push() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("vector");
    let args = vec![create_integer_literal(42)];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "push", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("vector"));
    assert!(code.contains("push"));
}

#[test]
fn test_transpile_method_call_pop() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("stack");
    let args = vec![];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "pop", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("stack"));
    assert!(code.contains("pop"));
}

#[test]
fn test_transpile_method_call_insert() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("list");
    let args = vec![create_integer_literal(0), create_string_literal("new")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "insert", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("list"));
    assert!(code.contains("insert"));
}

#[test]
fn test_transpile_method_call_remove() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("collection");
    let args = vec![create_integer_literal(1)];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "remove", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("collection"));
    assert!(code.contains("remove"));
}

#[test]
fn test_transpile_method_call_clear() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("data");
    let args = vec![];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "clear", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("data"));
    assert!(code.contains("clear"));
}

// ========== COLLECTION ACCESSOR METHODS TESTS ==========

#[test]
fn test_transpile_method_call_len() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("items");
    let args = vec![];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "len", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("items"));
    assert!(code.contains("len"));
}

#[test]
fn test_transpile_method_call_is_empty() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("collection");
    let args = vec![];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "is_empty", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("collection"));
    assert!(code.contains("is_empty"));
}

#[test]
fn test_transpile_method_call_slice() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("array");
    let args = vec![create_integer_literal(1), create_integer_literal(5)];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "slice", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("array"));
    assert!(code.contains("1"));
    assert!(code.contains("5"));
}

#[test]
fn test_transpile_method_call_first() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("sequence");
    let args = vec![];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "first", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("sequence"));
    assert!(code.contains("first"));
}

#[test]
fn test_transpile_method_call_last() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("elements");
    let args = vec![];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "last", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("elements"));
    assert!(code.contains("last"));
}

// ========== STRING METHODS TESTS ==========

#[test]
fn test_transpile_method_call_string_to_upper() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("text");
    let args = vec![];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "to_upper", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("text"));
    assert!(code.contains("to_uppercase"));
}

#[test]
fn test_transpile_method_call_string_to_lower() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("message");
    let args = vec![];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "to_lower", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("message"));
    assert!(code.contains("to_lowercase"));
}

#[test]
fn test_transpile_method_call_string_length() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("content");
    let args = vec![];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "length", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("content"));
    assert!(code.contains("len"));
}

#[test]
fn test_transpile_method_call_string_trim() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("raw_text");
    let args = vec![];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "trim", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("raw_text"));
    assert!(code.contains("trim"));
}

#[test]
fn test_transpile_method_call_string_split() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("sentence");
    let args = vec![create_string_literal(",")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "split", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("sentence"));
    assert!(code.contains("split"));
}

#[test]
fn test_transpile_method_call_string_replace() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("original");
    let args = vec![create_string_literal("old"), create_string_literal("new")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "replace", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("original"));
    assert!(code.contains("replace"));
}

#[test]
fn test_transpile_method_call_string_starts_with() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("filename");
    let args = vec![create_string_literal("prefix")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "starts_with", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("filename"));
    assert!(code.contains("starts_with"));
}

#[test]
fn test_transpile_method_call_string_ends_with() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("path");
    let args = vec![create_string_literal(".rs")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "ends_with", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("path"));
    assert!(code.contains("ends_with"));
}

// ========== DATAFRAME METHODS TESTS ==========

#[test]
fn test_transpile_method_call_dataframe_select() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("df");
    let args = vec![create_identifier("columns")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "select", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("df"));
    assert!(code.contains("select"));
}

#[test]
fn test_transpile_method_call_dataframe_groupby() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("data");
    let args = vec![create_string_literal("category")];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "groupby", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("data"));
    assert!(code.contains("groupby"));
}

#[test]
fn test_transpile_method_call_dataframe_statistical() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("numbers_df");
    let args = vec![];
    
    for method in &["mean", "std", "min", "max", "sum", "count"] {
        let result = transpiler.transpile_method_call_refactored(&obj, method, &args);
        assert!(result.is_ok(), "Failed for method: {}", method);
        let tokens = result.unwrap();
        let code = tokens.to_string();
        assert!(code.contains("numbers_df"));
        assert!(code.contains(method));
    }
}

// ========== UNKNOWN METHOD FALLBACK TEST ==========

#[test]
fn test_transpile_method_call_unknown_method_fallback() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("custom_obj");
    let args = vec![create_integer_literal(123)];
    
    let result = transpiler.transpile_method_call_refactored(&obj, "unknown_method", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("custom_obj"));
    assert!(code.contains("unknown_method"));
}

// ========== COMPREHENSIVE METHOD CATEGORIES TEST ==========

#[test]
fn test_all_method_categories_comprehensive() {
    let transpiler = create_test_transpiler();
    let obj = create_identifier("test_obj");
    let single_arg = vec![create_integer_literal(1)];
    
    // Test one method from each category to ensure dispatcher works
    let test_cases = vec![
        ("map", &single_arg[..]),      // iterator
        ("get", &single_arg[..]),      // hashmap
        ("contains", &single_arg[..]), // hashset
        ("push", &single_arg[..]),     // collection_mutator
        ("len", &[][..]),              // collection_accessor
        ("trim", &[][..]),             // string
        ("select", &single_arg[..]),   // dataframe
        ("custom", &single_arg[..]),   // fallback
    ];
    
    for (method, args) in test_cases {
        let result = transpiler.transpile_method_call_refactored(&obj, method, args);
        assert!(result.is_ok(), "Method '{}' failed", method);
        let tokens = result.unwrap();
        let code = tokens.to_string();
        assert!(code.contains("test_obj"), "Method '{}' should contain object", method);
        assert!(code.contains(method), "Method '{}' should contain method name", method);
    }
}