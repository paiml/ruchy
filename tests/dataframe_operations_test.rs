#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
//! Tests for `DataFrame` operations and method chaining

#![allow(clippy::unwrap_used)]

use ruchy::{compile, is_valid_syntax};

#[test]
fn test_dataframe_filter() {
    let code = "df![age => [25, 30, 35]].head(2)";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("head"));
}

#[test]
fn test_dataframe_select() {
    let code = "df![name => [\"Alice\", \"Bob\"], age => [25, 30]].select(name)";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("select"));
}

#[test]
fn test_dataframe_sort() {
    let code = "df![age => [30, 25, 35]].sort(age)";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("sort"));
}

#[test]
fn test_dataframe_limit() {
    let code = "df![x => [1, 2, 3, 4, 5]].limit(3)";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("head") || result.contains("limit"));
}

#[test]
fn test_dataframe_method_chaining() {
    let code = "df![age => [25, 30, 35, 20]].sort(age).head(2)";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("sort"));
    assert!(result.contains("head"));
}

#[test]
fn test_dataframe_groupby() {
    let code = "df![city => [\"NYC\", \"LA\", \"NYC\"], sales => [100, 200, 150]].groupby(city)";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("groupby"));
}
