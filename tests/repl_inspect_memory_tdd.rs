// TDD Tests for :inspect command memory estimation (CH23-MEMORY)
//
// Requirements:
// 1. :inspect should show memory estimation for all value types
// 2. Memory shown in bytes with human-readable format (~X bytes)
// 3. Memory estimates should be reasonable approximations
// 4. Works for integers, strings, arrays, objects

use ruchy::runtime::repl::*;
use std::path::PathBuf;

#[test]
fn test_inspect_integer_shows_memory() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    repl.eval("let x = 42").unwrap();
    let result = repl.eval(":inspect x").unwrap();

    // Should show memory estimation
    assert!(
        result.contains("Memory") || result.contains('~') || result.contains("bytes"),
        "Expected memory information but got: {result}"
    );
}

#[test]
fn test_inspect_string_shows_memory() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    repl.eval("let s = \"Hello World\"").unwrap();
    let result = repl.eval(":inspect s").unwrap();

    // Should show memory estimation
    assert!(
        result.contains("Memory") || result.contains('~') || result.contains("bytes"),
        "Expected memory information but got: {result}"
    );
}

#[test]
fn test_inspect_array_shows_memory() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    repl.eval("let arr = [1, 2, 3, 4, 5]").unwrap();
    let result = repl.eval(":inspect arr").unwrap();

    // Should show memory estimation
    assert!(
        result.contains("Memory") || result.contains('~') || result.contains("bytes"),
        "Expected memory information but got: {result}"
    );
}

#[test]
fn test_inspect_object_shows_memory() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    repl.eval("let obj = {\"name\": \"Alice\", \"age\": 30}")
        .unwrap();
    let result = repl.eval(":inspect obj").unwrap();

    // Should show memory estimation
    assert!(
        result.contains("Memory") || result.contains('~') || result.contains("bytes"),
        "Expected memory information but got: {result}"
    );
}

#[test]
fn test_memory_estimation_reasonable() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Small integer should be ~8 bytes
    repl.eval("let n = 42").unwrap();
    let result = repl.eval(":inspect n").unwrap();

    // Should show a small memory size (integers are 8 bytes)
    assert!(
        result.contains('8') || result.contains("16"), // Allow some overhead
        "Expected reasonable memory size for integer, got: {result}"
    );
}

#[test]
fn test_memory_format_human_readable() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    repl.eval("let x = 42").unwrap();
    let result = repl.eval(":inspect x").unwrap();

    // Should show memory in human-readable format (~X bytes)
    assert!(
        result.contains('~') && result.contains("bytes"),
        "Expected '~X bytes' format, got: {result}"
    );
}
