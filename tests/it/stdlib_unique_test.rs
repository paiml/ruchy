#![allow(missing_docs)]
//! STDLIB-006: Array `.unique()` method
//!
//! Tests for array deduplication operation
//! IMPLEMENTATION: Uses IndexSet-based deduplication preserving order

use assert_cmd::Command;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// ============================================================================
// GREEN PHASE: .unique() tests (NOW WORKING)
// ============================================================================

#[test]
fn test_unique_basic() {
    let code = r"
let arr = [1, 2, 1, 3, 2];
println(arr.unique())
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[1, 2, 3]\n");
}

#[test]
fn test_unique_empty() {
    let code = r"
let arr = [];
println(arr.unique())
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[]\n");
}

#[test]
fn test_unique_already_unique() {
    let code = r"
let arr = [1, 2, 3];
println(arr.unique())
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[1, 2, 3]\n");
}

#[test]
fn test_unique_all_duplicates() {
    let code = r"
let arr = [5, 5, 5, 5];
println(arr.unique())
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[5]\n");
}

#[test]
fn test_unique_strings() {
    let code = r#"
let arr = ["a", "b", "a", "c", "b"];
println(arr.unique())
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[\"a\", \"b\", \"c\"]\n");
}

#[test]
fn test_unique_preserves_order() {
    let code = r"
let arr = [3, 1, 4, 1, 5, 9, 2, 6, 5];
println(arr.unique())
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[3, 1, 4, 5, 9, 2, 6]\n");
}
