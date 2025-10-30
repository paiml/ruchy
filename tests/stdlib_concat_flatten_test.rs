//! STDLIB-005: Array .`concat()` and .`flatten()` methods
//!
//! ROOT CAUSE: Missing array concatenation and flattening operations
//! SOLUTION: Implement .`concat()` and .`flatten()` methods
//!
//! EXTREME TDD: RED → GREEN → REFACTOR

use assert_cmd::Command;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ============================================================================
// RED PHASE: .concat() tests (WILL FAIL)
// ============================================================================

#[test]
fn test_concat_basic() {
    let code = r"
let a = [1, 2];
let b = [3, 4];
println(a.concat(b))
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[1, 2, 3, 4]\nnil\n");
}

#[test]
fn test_concat_empty_arrays() {
    let code = r"
let a = [1, 2];
let b = [];
println(a.concat(b))
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[1, 2]\nnil\n");
}

#[test]
fn test_concat_multiple() {
    let code = r"
let a = [1];
let b = [2];
let c = [3];
println(a.concat(b).concat(c))
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[1, 2, 3]\nnil\n");
}

#[test]
fn test_concat_strings() {
    let code = r#"
let a = ["hello"];
let b = ["world"];
println(a.concat(b))
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[\"hello\", \"world\"]\nnil\n");
}

// ============================================================================
// RED PHASE: .flatten() tests (WILL FAIL)
// ============================================================================

#[test]
fn test_flatten_basic() {
    let code = r"
let nested = [[1, 2], [3, 4]];
println(nested.flatten())
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[1, 2, 3, 4]\nnil\n");
}

#[test]
fn test_flatten_single_level() {
    let code = r"
let nested = [[1], [2], [3]];
println(nested.flatten())
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[1, 2, 3]\nnil\n");
}

#[test]
fn test_flatten_empty_subarrays() {
    let code = r"
let nested = [[1, 2], [], [3]];
println(nested.flatten())
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[1, 2, 3]\nnil\n");
}

#[test]
fn test_flatten_mixed_types() {
    let code = r#"
let nested = [["a", "b"], ["c", "d"]];
println(nested.flatten())
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[\"a\", \"b\", \"c\", \"d\"]\nnil\n");
}

#[test]
fn test_flatten_already_flat() {
    let code = r"
let flat = [1, 2, 3];
println(flat.flatten())
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("[1, 2, 3]\nnil\n");
}
