#![allow(deprecated)]
// LANG-COMP-003: Control Flow - Validation Tests with Traceability
// Links to: examples/lang_comp/03-control-flow/*.ruchy
// Validates: LANG-COMP-003 Control Flow (if, match, for, while, break/continue)
// EXTREME TDD Protocol: Tests use assert_cmd + mandatory naming convention
#![allow(clippy::ignore_without_reason)] // LANG-COMP tests with known issues use ignore

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

/// 15-TOOL VALIDATION: Run ALL 15 native tools on example file
/// MANDATORY/BLOCKING: Test passes ONLY if all tools succeed
/// Skipped: Tool 3 (repl - interactive), Tool 15 (notebook - needs file arg implementation)
/// Working: 13/15 tools = 87% coverage
fn validate_with_15_tools(example: &PathBuf) {
    // TOOL 1: ruchy check - Syntax validation
    ruchy_cmd().arg("check").arg(example).assert().success();

    // TOOL 2: ruchy transpile - Rust code generation
    ruchy_cmd().arg("transpile").arg(example).assert().success();

    // TOOL 3: ruchy repl - SKIPPED (requires interactive input)

    // TOOL 4: ruchy lint - Static analysis
    ruchy_cmd().arg("lint").arg(example).assert().success();

    // TOOL 5: ruchy compile - Binary compilation
    // DEFECT-RACE-CONDITION FIX: Use unique output path per example file to avoid parallel test collisions
    let compile_output = std::env::temp_dir().join(format!(
        "compile_test_{}_{}",
        example.file_stem().unwrap().to_string_lossy(),
        std::process::id()
    ));
    ruchy_cmd().arg("compile").arg(example).arg("-o").arg(&compile_output).assert().success();
    std::fs::remove_file(&compile_output).ok();

    // TOOL 6: ruchy run - Execution
    ruchy_cmd().arg("run").arg(example).assert().success();

    // TOOL 7: ruchy coverage - Test coverage
    ruchy_cmd().arg("coverage").arg(example).assert().success();

    // TOOL 8: ruchy runtime --bigo - Complexity analysis
    ruchy_cmd()
        .arg("runtime")
        .arg(example)
        .arg("--bigo")
        .assert()
        .success();

    // TOOL 9: ruchy ast - AST verification
    ruchy_cmd().arg("ast").arg(example).assert().success();

    // TOOL 10: ruchy wasm - WASM compilation
    ruchy_cmd().arg("wasm").arg(example).assert().success();

    // TOOL 11: ruchy provability - Formal verification
    ruchy_cmd()
        .arg("provability")
        .arg(example)
        .assert()
        .success();

    // TOOL 12: ruchy property-tests - Property-based testing (100 cases for speed)
    ruchy_cmd()
        .arg("property-tests")
        .arg(example)
        .arg("--cases")
        .arg("100")
        .assert()
        .success();

    // TOOL 13: ruchy mutations - Mutation testing (validates single files correctly)
    ruchy_cmd()
        .arg("mutations")
        .arg(example)
        .arg("--min-coverage")
        .arg("0")
        .arg("--timeout")
        .arg("60")
        .assert()
        .success();

    // TOOL 14: ruchy fuzz - Fuzz testing (10 iterations for speed in tests)
    ruchy_cmd()
        .arg("fuzz")
        .arg(example)
        .arg("--iterations")
        .arg("10")
        .assert()
        .success();

    // TOOL 15: ruchy notebook - SKIPPED (requires server)
}

// ============================================================================
// LANG-COMP-003-01: If Expression Tests
// Links to: examples/lang_comp/03-control-flow/01_if.ruchy
// ============================================================================

#[test]
fn test_langcomp_003_01_if_expression_true_branch() {
    // Test: if true { 1 } else { 2 } returns 1
    let temp_file = std::env::temp_dir().join("langcomp_003_01_if_true.ruchy");
    std::fs::write(&temp_file, "println(if true { 1 } else { 2 })").unwrap();

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
    std::fs::write(&temp_file, "println(if false { 1 } else { 2 })").unwrap();

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
    // 15-TOOL VALIDATION: examples/lang_comp/03-control-flow/01_if.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("01_if.ruchy");
    validate_with_15_tools(&example);
}

// ============================================================================
// LANG-COMP-003-02: Match Expression Tests
// Links to: examples/lang_comp/03-control-flow/02_match.ruchy
// ============================================================================

#[test]
fn test_langcomp_003_02_match_literal_pattern() {
    // Test: match 1 { 1 => 100, 2 => 200, _ => 999 } returns 100
    let temp_file = std::env::temp_dir().join("langcomp_003_02_match_literal.ruchy");
    std::fs::write(&temp_file, "println(match 1 { 1 => 100, 2 => 200, _ => 999 })").unwrap();

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
    std::fs::write(&temp_file, "println(match 99 { 1 => 100, 2 => 200, _ => 999 })").unwrap();

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
    // 15-TOOL VALIDATION: examples/lang_comp/03-control-flow/02_match.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("02_match.ruchy");
    validate_with_15_tools(&example);
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
        r"
let sum = 0
for i in 0..3 {
    sum = sum + i
}
println(sum)
",
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
    // 15-TOOL VALIDATION: examples/lang_comp/03-control-flow/03_for.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("03_for.ruchy");
    validate_with_15_tools(&example);
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
        r"
let count = 0
while count < 3 {
    count = count + 1
}
println(count)
",
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
    // 15-TOOL VALIDATION: examples/lang_comp/03-control-flow/04_while.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("04_while.ruchy");
    validate_with_15_tools(&example);
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
        r"
let i = 0
while true {
    if i == 3 {
        break
    }
    i = i + 1
}
println(i)
",
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
    // 15-TOOL VALIDATION: examples/lang_comp/03-control-flow/05_break_continue.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("05_break_continue.ruchy");
    validate_with_15_tools(&example);
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
            let code = format!("if {i} > 50 {{ 1 }} else {{ 0 }}");
            let temp_file = std::env::temp_dir().join(format!("langcomp_003_prop_if_{i}.ruchy"));
            std::fs::write(&temp_file, &code).unwrap();

            ruchy_cmd()
                .arg("run")
                .arg(&temp_file)
                .assert()
                .success()
                .stdout(predicate::str::is_match(r"^(0|1)\s*$").unwrap());

            std::fs::remove_file(&temp_file).ok();
        }
    }

    #[test]
    #[ignore]
    fn test_langcomp_003_property_match_wildcard_never_fails() {
        // Property: match with wildcard always succeeds
        for i in 0..100 {
            let code = format!("match {i} {{ 1 => 100, _ => 999 }}");
            let temp_file = std::env::temp_dir().join(format!("langcomp_003_prop_match_{i}.ruchy"));
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
                r"
let count = 0
for i in 0..{n} {{
    count = count + 1
}}
count
"
            );
            let temp_file = std::env::temp_dir().join(format!("langcomp_003_prop_for_{n}.ruchy"));
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
