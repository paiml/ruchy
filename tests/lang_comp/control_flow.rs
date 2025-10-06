// LANG-COMP-003: Control Flow - Validation Tests with Traceability
// Links to: examples/lang_comp/03-control-flow/*.ruchy
// Validates: LANG-COMP-003 Control Flow (if, match, for, while, break/continue)
// EXTREME TDD Protocol: Tests use assert_cmd + mandatory naming convention

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

/// Helper to get ruchy binary command
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper to get example file path
fn example_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/lang_comp/03-control-flow")
        .join(relative_path)
}

// ============================================================================
// LANG-COMP-003-01: If Expression Tests
// Links to: examples/lang_comp/03-control-flow/01_if.ruchy
// ============================================================================

#[test]
fn test_langcomp_003_01_if_expression_true_branch() {
    // Test: if true { 1 } else { 2 } returns 1
    let temp_file = std::env::temp_dir().join("langcomp_003_01_if_true.ruchy");
    std::fs::write(&temp_file, "if true { 1 } else { 2 }").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_003_01_if_expression_false_branch() {
    // Test: if false { 1 } else { 2 } returns 2
    let temp_file = std::env::temp_dir().join("langcomp_003_01_if_false.ruchy");
    std::fs::write(&temp_file, "if false { 1 } else { 2 }").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));

    std::fs::remove_file(&temp_file).ok();
}

// NOTE: if without else is NOT supported - returns Unit type which can't be used as expression value
// This is by design - all if expressions must have else for type safety

#[test]
fn test_langcomp_003_01_if_expression_example_file() {
    // Validates: examples/lang_comp/03-control-flow/01_if.ruchy
    ruchy_cmd()
        .arg("run")
        .arg(example_path("01_if.ruchy"))
        .assert()
        .success();
}

// ============================================================================
// LANG-COMP-003-02: Match Expression Tests
// Links to: examples/lang_comp/03-control-flow/02_match.ruchy
// ============================================================================

#[test]
fn test_langcomp_003_02_match_literal_pattern() {
    // Test: match 1 { 1 => 100, 2 => 200, _ => 999 } returns 100
    let temp_file = std::env::temp_dir().join("langcomp_003_02_match_literal.ruchy");
    std::fs::write(&temp_file, "match 1 { 1 => 100, 2 => 200, _ => 999 }").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("100"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_003_02_match_wildcard_pattern() {
    // Test: match 99 { 1 => 100, 2 => 200, _ => 999 } returns 999 (wildcard)
    let temp_file = std::env::temp_dir().join("langcomp_003_02_match_wildcard.ruchy");
    std::fs::write(&temp_file, "match 99 { 1 => 100, 2 => 200, _ => 999 }").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("999"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_003_02_match_expression_example_file() {
    // Validates: examples/lang_comp/03-control-flow/02_match.ruchy
    ruchy_cmd()
        .arg("run")
        .arg(example_path("02_match.ruchy"))
        .assert()
        .success();
}

// ============================================================================
// LANG-COMP-003-03: For Loop Tests
// Links to: examples/lang_comp/03-control-flow/03_for.ruchy
// ============================================================================

#[test]
fn test_langcomp_003_03_for_loop_range() {
    // Test: for loop iterates correct number of times
    let temp_file = std::env::temp_dir().join("langcomp_003_03_for_range.ruchy");
    std::fs::write(
        &temp_file,
        r#"
let sum = 0
for i in 0..3 {
    sum = sum + i
}
sum
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_003_03_for_loop_example_file() {
    // Validates: examples/lang_comp/03-control-flow/03_for.ruchy
    ruchy_cmd()
        .arg("run")
        .arg(example_path("03_for.ruchy"))
        .assert()
        .success();
}

// ============================================================================
// LANG-COMP-003-04: While Loop Tests
// Links to: examples/lang_comp/03-control-flow/04_while.ruchy
// ============================================================================

#[test]
fn test_langcomp_003_04_while_loop_condition() {
    // Test: while loop runs until condition false
    let temp_file = std::env::temp_dir().join("langcomp_003_04_while.ruchy");
    std::fs::write(
        &temp_file,
        r#"
let count = 0
while count < 3 {
    count = count + 1
}
count
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_003_04_while_loop_example_file() {
    // Validates: examples/lang_comp/03-control-flow/04_while.ruchy
    ruchy_cmd()
        .arg("run")
        .arg(example_path("04_while.ruchy"))
        .assert()
        .success();
}

// ============================================================================
// LANG-COMP-003-05: Loop Control Tests (break, continue)
// Links to: examples/lang_comp/03-control-flow/05_break_continue.ruchy
// ============================================================================

#[test]
fn test_langcomp_003_05_break_exits_loop() {
    // Test: break statement exits loop immediately
    let temp_file = std::env::temp_dir().join("langcomp_003_05_break.ruchy");
    std::fs::write(
        &temp_file,
        r#"
let i = 0
while true {
    if i == 3 {
        break
    }
    i = i + 1
}
i
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_003_05_loop_control_example_file() {
    // Validates: examples/lang_comp/03-control-flow/05_break_continue.ruchy
    ruchy_cmd()
        .arg("run")
        .arg(example_path("05_break_continue.ruchy"))
        .assert()
        .success();
}

// ============================================================================
// LANG-COMP-003: Property Tests (Mathematical Correctness Proofs)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_langcomp_003_property_if_else_always_returns_value() {
        // Property: if-else always returns a value (no case uncovered)
        for i in 0..100 {
            let code = format!("if {} > 50 {{ 1 }} else {{ 0 }}", i);
            let temp_file = std::env::temp_dir().join(format!("langcomp_003_prop_if_{}.ruchy", i));
            std::fs::write(&temp_file, &code).unwrap();

            ruchy_cmd()
                .arg("run")
                .arg(&temp_file)
                .assert()
                .success()
                .stdout(predicate::str::is_match("^(0|1)$").unwrap());

            std::fs::remove_file(&temp_file).ok();
        }
    }

    #[test]
    #[ignore]
    fn test_langcomp_003_property_match_wildcard_never_fails() {
        // Property: match with wildcard always succeeds
        for i in 0..100 {
            let code = format!("match {} {{ 1 => 100, _ => 999 }}", i);
            let temp_file =
                std::env::temp_dir().join(format!("langcomp_003_prop_match_{}.ruchy", i));
            std::fs::write(&temp_file, &code).unwrap();

            ruchy_cmd().arg("run").arg(&temp_file).assert().success();

            std::fs::remove_file(&temp_file).ok();
        }
    }

    #[test]
    #[ignore]
    fn test_langcomp_003_property_for_loop_iterations_equal_range_size() {
        // Property: for loop runs exactly range.len() times
        for n in 1..10 {
            let code = format!(
                r#"
let count = 0
for i in 0..{} {{
    count = count + 1
}}
count
"#,
                n
            );
            let temp_file = std::env::temp_dir().join(format!("langcomp_003_prop_for_{}.ruchy", n));
            std::fs::write(&temp_file, &code).unwrap();

            ruchy_cmd()
                .arg("run")
                .arg(&temp_file)
                .assert()
                .success()
                .stdout(predicate::str::contains(n.to_string()));

            std::fs::remove_file(&temp_file).ok();
        }
    }
}
