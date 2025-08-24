#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
//! REPL `DataFrame` evaluation tests

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::doc_markdown)]

use ruchy::runtime::Repl;

#[test]
fn test_repl_dataframe_empty() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    let result = repl.eval("df![]");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("DataFrame"));
}

#[test]
fn test_repl_dataframe_single_column() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    let result = repl.eval(r#"df![age => [25, 30, 35]]"#);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("DataFrame"));
    assert!(output.contains("age"));
    assert!(output.contains("25"));
    assert!(output.contains("30"));
    assert!(output.contains("35"));
}

#[test]
fn test_repl_dataframe_multiple_columns() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    let result = repl.eval(
        r#"df![
        name => ["Alice", "Bob", "Charlie"],
        age => [25, 30, 35],
        score => [95.5, 87.3, 92.1]
    ]"#,
    );
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("DataFrame"));
    assert!(output.contains("name"));
    assert!(output.contains("Alice"));
    assert!(output.contains("Bob"));
    assert!(output.contains("Charlie"));
    assert!(output.contains("age"));
    assert!(output.contains("score"));
}

#[test]
fn test_repl_dataframe_with_variables() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Define some variables
    assert!(repl.eval("let x = 10").is_ok());
    assert!(repl.eval("let y = 20").is_ok());

    // Use them in DataFrame
    let result = repl.eval(
        r#"df![
        values => [x, y, x + y]
    ]"#,
    );
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("10"));
    assert!(output.contains("20"));
    assert!(output.contains("30"));
}

#[test]
fn test_repl_dataframe_assignment() {
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Assign DataFrame to variable
    let result = repl.eval(
        r#"let data = df![
        id => [1, 2, 3],
        value => [100, 200, 300]
    ]"#,
    );
    assert!(result.is_ok());

    // Should be able to reference it
    let result = repl.eval("data");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("DataFrame"));
    assert!(output.contains("id"));
    assert!(output.contains("value"));
}
