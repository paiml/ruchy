//! STDLIB-009: String .substring() + Array .sort() and .join()
//!
//! ROOT CAUSE: Missing implementations for these convenience methods
//! SOLUTION: Implement substring(), sort(), join()
//!
//! EXTREME TDD: RED → GREEN → REFACTOR

use assert_cmd::Command;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ============================================================================
// RED PHASE: .substring() tests (WILL FAIL)
// ============================================================================

#[test]
fn test_substring_basic() {
    let code = r#"
let s = "hello";
println(s.substring(1, 3))
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("el\nnil\n");
}

#[test]
fn test_substring_full_string() {
    let code = r#"
let s = "hello";
println(s.substring(0, 5))
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("hello\nnil\n");
}

#[test]
fn test_substring_empty() {
    let code = r#"
let s = "hello";
println(s.substring(2, 2))
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("\nnil\n");
}

// ============================================================================
// RED PHASE: .sort() tests (WILL FAIL)
// ============================================================================

#[test]
fn test_sort_integers() {
    let code = r#"
let arr = [3, 1, 4, 1, 5, 9, 2, 6];
println(arr.sort())
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[1, 1, 2, 3, 4, 5, 6, 9]\nnil\n");
}

#[test]
fn test_sort_strings() {
    let code = r#"
let arr = ["zebra", "apple", "banana", "cherry"];
println(arr.sort())
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[\"apple\", \"banana\", \"cherry\", \"zebra\"]\nnil\n");
}

#[test]
fn test_sort_empty() {
    let code = r#"
let arr = [];
println(arr.sort())
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[]\nnil\n");
}

// ============================================================================
// RED PHASE: .join() tests (WILL FAIL)
// ============================================================================

#[test]
fn test_join_basic() {
    let code = r#"
let arr = ["a", "b", "c"];
println(arr.join(","))
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("a,b,c\nnil\n");
}

#[test]
fn test_join_with_space() {
    let code = r#"
let arr = ["hello", "world"];
println(arr.join(" "))
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("hello world\nnil\n");
}

#[test]
fn test_join_empty_separator() {
    let code = r#"
let arr = ["a", "b", "c"];
println(arr.join(""))
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("abc\nnil\n");
}

#[test]
fn test_join_integers() {
    let code = r#"
let arr = [1, 2, 3];
println(arr.join("-"))
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("1-2-3\nnil\n");
}
