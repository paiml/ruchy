//! TDD Test Suite for method_call_refactored.rs
//! Target: 15.58% → 80%+ coverage
//! PMAT: Keep complexity <10 per test

#![cfg(test)]

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

/// Test helper: Parse and transpile code
fn transpile_method_call(code: &str) -> anyhow::Result<String> {
    let mut parser = Parser::new(code);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let tokens = transpiler.transpile(&ast)?;
    Ok(tokens.to_string())
}

// Iterator Methods Tests
#[test]
fn test_map_method() {
    let code = "[1, 2, 3].map(|x| x * 2)";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("iter()") && result.contains("map"));
}

#[test]
fn test_filter_method() {
    let code = "[1, 2, 3, 4].filter(|x| x % 2 == 0)";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("filter") && result.contains("collect"));
}

#[test]
fn test_reduce_method() {
    let code = "[1, 2, 3].reduce(|a, b| a + b)";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("reduce"));
}

#[test]
fn test_fold_method() {
    let code = "[1, 2, 3].fold(0, |acc, x| acc + x)";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("fold"));
}

#[test]
fn test_any_method() {
    let code = "[1, 2, 3].any(|x| x > 2)";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("any"));
}

#[test]
fn test_all_method() {
    let code = "[1, 2, 3].all(|x| x > 0)";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("all"));
}

#[test]
fn test_find_method() {
    let code = "[1, 2, 3].find(|x| x % 2 == 0)";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("find"));
}

// HashMap/Dict Methods Tests
#[test]
fn test_hashmap_get() {
    let code = r#"{"a": 1}.get("a")"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("get"));
}

#[test]
fn test_hashmap_contains_key() {
    let code = r#"{"a": 1}.contains_key("a")"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("contains_key"));
}

#[test]
fn test_hashmap_keys() {
    let code = r#"{"a": 1, "b": 2}.keys()"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("keys"));
}

#[test]
fn test_hashmap_values() {
    let code = r#"{"a": 1, "b": 2}.values()"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("values"));
}

#[test]
fn test_hashmap_items() {
    let code = r#"{"a": 1}.items()"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("iter") || result.contains("items"));
}

// HashSet Methods Tests
#[test]
fn test_hashset_contains() {
    let code = "{1, 2, 3}.contains(2)";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("contains"));
}

#[test]
fn test_hashset_union() {
    let code = "{1, 2}.union({2, 3})";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("union"));
}

#[test]
fn test_hashset_intersection() {
    let code = "{1, 2, 3}.intersection({2, 3, 4})";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("intersection"));
}

// Collection Mutator Tests
#[test]
fn test_vec_push() {
    let code = r#"
let mut v = [1, 2, 3]
v.push(4)
"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("push"));
}

#[test]
fn test_vec_pop() {
    let code = r#"
let mut v = [1, 2, 3]
v.pop()
"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("pop"));
}

#[test]
fn test_vec_insert() {
    let code = r#"
let mut v = [1, 3]
v.insert(1, 2)
"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("insert"));
}

#[test]
fn test_vec_remove() {
    let code = r#"
let mut v = [1, 2, 3]
v.remove(1)
"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("remove"));
}

#[test]
fn test_vec_clear() {
    let code = r#"
let mut v = [1, 2, 3]
v.clear()
"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("clear"));
}

#[test]
fn test_vec_extend() {
    let code = r#"
let mut v = [1, 2]
v.extend([3, 4])
"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("extend"));
}

// Collection Accessor Tests
#[test]
fn test_vec_len() {
    let code = "[1, 2, 3].len()";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("len"));
}

#[test]
fn test_vec_is_empty() {
    let code = "[].is_empty()";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("is_empty"));
}

#[test]
fn test_vec_first() {
    let code = "[1, 2, 3].first()";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("first"));
}

#[test]
fn test_vec_last() {
    let code = "[1, 2, 3].last()";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("last"));
}

// String Methods Tests
#[test]
fn test_string_to_upper() {
    let code = r#""hello".to_upper()"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("to_uppercase") || result.contains("to_upper"));
}

#[test]
fn test_string_to_lower() {
    let code = r#""HELLO".to_lower()"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("to_lowercase") || result.contains("to_lower"));
}

#[test]
fn test_string_trim() {
    let code = r#""  hello  ".trim()"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("trim"));
}

#[test]
fn test_string_split() {
    let code = r#""a,b,c".split(",")"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("split"));
}

#[test]
fn test_string_replace() {
    let code = r#""hello world".replace("world", "rust")"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("replace"));
}

#[test]
fn test_string_starts_with() {
    let code = r#""hello".starts_with("he")"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("starts_with"));
}

#[test]
fn test_string_ends_with() {
    let code = r#""hello".ends_with("lo")"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("ends_with"));
}

#[test]
fn test_string_length() {
    let code = r#""hello".length()"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("len") || result.contains("length"));
}

// DataFrame Methods Tests (basic)
#[test]
fn test_dataframe_select() {
    let code = r#"df.select(["col1", "col2"])"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("select"));
}

#[test]
fn test_dataframe_groupby() {
    let code = r#"df.groupby("category")"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("group") || result.contains("groupby"));
}

#[test]
fn test_dataframe_mean() {
    let code = "df.mean()";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("mean"));
}

#[test]
fn test_dataframe_sum() {
    let code = "df.sum()";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("sum"));
}

// Default Method Pass-through Test
#[test]
fn test_unknown_method_passthrough() {
    let code = "obj.custom_method(1, 2, 3)";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("custom_method"));
}

// Edge Cases
#[test]
fn test_chained_methods() {
    let code = "[1, 2, 3].map(|x| x * 2).filter(|x| x > 2)";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("map") && result.contains("filter"));
}

#[test]
fn test_nested_method_calls() {
    let code = r#""hello".replace("l", "r").to_upper()"#;
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("replace"));
}

// PMAT Complexity Check - Each test should be <10 complexity
#[test]
fn test_complex_iterator_chain() {
    let code = "[1, 2, 3, 4, 5].map(|x| x * 2).filter(|x| x > 5).fold(0, |a, b| a + b)";
    let result = transpile_method_call(code).unwrap();
    assert!(result.contains("map") || result.contains("filter") || result.contains("fold"));
}