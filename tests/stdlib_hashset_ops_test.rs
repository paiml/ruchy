//! STDLIB-007: Array set operations (.union, .intersection, .difference)
//!
//! ROOT CAUSE: Missing set operations for arrays (treating arrays as sets)
//! SOLUTION: Implement .union(), .intersection(), .difference() for arrays
//!
//! EXTREME TDD: RED → GREEN → REFACTOR
//!
//! NOTE: These operate on arrays but provide set semantics (unique elements)

use assert_cmd::Command;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ============================================================================
// RED PHASE: Set operation tests (WILL FAIL)
// ============================================================================

#[test]
fn test_union_basic() {
    let code = r#"
let a = [1, 2, 3];
let b = [3, 4, 5];
println(a.union(b))
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[1, 2, 3, 4, 5]\nnil\n");
}

#[test]
fn test_union_with_duplicates() {
    let code = r#"
let a = [1, 2, 2, 3];
let b = [3, 3, 4, 5];
println(a.union(b))
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[1, 2, 3, 4, 5]\nnil\n");
}

#[test]
fn test_intersection_basic() {
    let code = r#"
let a = [1, 2, 3, 4];
let b = [3, 4, 5, 6];
println(a.intersection(b))
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[3, 4]\nnil\n");
}

#[test]
fn test_intersection_no_common() {
    let code = r#"
let a = [1, 2];
let b = [3, 4];
println(a.intersection(b))
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[]\nnil\n");
}

#[test]
fn test_difference_basic() {
    let code = r#"
let a = [1, 2, 3, 4];
let b = [3, 4, 5, 6];
println(a.difference(b))
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[1, 2]\nnil\n");
}

#[test]
fn test_difference_all_different() {
    let code = r#"
let a = [1, 2];
let b = [3, 4];
println(a.difference(b))
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[1, 2]\nnil\n");
}

#[test]
fn test_difference_all_removed() {
    let code = r#"
let a = [1, 2];
let b = [1, 2, 3];
println(a.difference(b))
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[]\nnil\n");
}

#[test]
fn test_union_strings() {
    let code = r#"
let a = ["a", "b"];
let b = ["b", "c"];
println(a.union(b))
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[\"a\", \"b\", \"c\"]\nnil\n");
}
