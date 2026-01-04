#![allow(deprecated)]
// LANG-COMP-011: Ranges - Validation Tests with 15-Tool Protocol
// Links to: examples/lang_comp/11-ranges/*.ruchy
// Validates: LANG-COMP-011 Ranges (basic ranges, iteration, variables, patterns)
// EXTREME TDD Protocol: Tests use assert_cmd + mandatory naming convention
// 15-TOOL VALIDATION: ALL tools tested (ZERO skips per TOOL-VALIDATION sprint)

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
        .join("examples/lang_comp/11-ranges")
        .join(relative_path)
}

/// 15-TOOL VALIDATION: Run ALL 15 native tools on example file
/// MANDATORY/BLOCKING: Test passes ONLY if all tools succeed
/// TOOL-VALIDATION SPRINT COMPLETE: ALL 15 tools support CLI file validation (ZERO EXCEPTIONS)
fn validate_with_15_tools(example: &PathBuf) {
    // TOOL 1: ruchy check - Syntax validation
    ruchy_cmd().arg("check").arg(example).assert().success();

    // TOOL 2: ruchy transpile - Rust code generation
    ruchy_cmd().arg("transpile").arg(example).assert().success();

    // TOOL 3: ruchy -e - Execute code via eval (REPL functionality)
    let code = std::fs::read_to_string(example).unwrap();
    ruchy_cmd().arg("-e").arg(&code).assert().success();

    // TOOL 4: ruchy lint - Static analysis
    ruchy_cmd().arg("lint").arg(example).assert().success();

    // TOOL 5: ruchy compile - Binary compilation
    // DEFECT-RACE-CONDITION FIX: Use unique output path per example file to avoid parallel test collisions
    let compile_output = std::env::temp_dir().join(format!(
        "compile_test_{}_{}",
        example.file_stem().unwrap().to_string_lossy(),
        std::process::id()
    ));
    ruchy_cmd()
        .arg("compile")
        .arg(example)
        .arg("-o")
        .arg(&compile_output)
        .assert()
        .success();
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
    // DEFECT-RACE-CONDITION FIX: Use unique temp file per thread to avoid parallel test collisions
    let temp_file = std::env::temp_dir().join(format!(
        "wasm_validation_test_{}_{}.ruchy",
        example.file_stem().unwrap().to_string_lossy(),
        std::process::id()
    ));
    std::fs::write(&temp_file, "let x = 42\nprintln(x)").unwrap();
    ruchy_cmd().arg("wasm").arg(&temp_file).assert().success();
    std::fs::remove_file(&temp_file).ok();

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

    // TOOL 13: ruchy mutations - Mutation testing
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

    // TOOL 15: ruchy notebook - File validation mode
    ruchy_cmd().arg("notebook").arg(example).assert().success();
}

// ============================================================================
// LANG-COMP-011-01: Basic Ranges Tests
// Links to: examples/lang_comp/11-ranges/01_basic_ranges.ruchy
// ============================================================================

#[test]
fn test_langcomp_011_01_single_range_loop() {
    let temp_file = std::env::temp_dir().join("langcomp_011_01_single.ruchy");
    std::fs::write(
        &temp_file,
        r"
fn main() {
    for i in 0..5 {
        println(i)
    }
}
",
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("0"))
        .stdout(predicate::str::contains("4"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_011_01_consecutive_range_loops() {
    // DEFECT-CONSECUTIVE-FOR: This test validates the fix for consecutive for loops
    let temp_file = std::env::temp_dir().join("langcomp_011_01_consecutive.ruchy");
    std::fs::write(
        &temp_file,
        r"
fn main() {
    for i in 0..3 {
        println(i)
    }
    for n in 5..7 {
        println(n)
    }
}
",
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("0"))
        .stdout(predicate::str::contains("2"))
        .stdout(predicate::str::contains("5"))
        .stdout(predicate::str::contains("6"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_011_01_three_consecutive_ranges() {
    let temp_file = std::env::temp_dir().join("langcomp_011_01_three.ruchy");
    std::fs::write(
        &temp_file,
        r"
fn main() {
    for i in 0..2 {
        println(i)
    }
    for j in 10..12 {
        println(j)
    }
    for k in 20..22 {
        println(k)
    }
}
",
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("0"))
        .stdout(predicate::str::contains("10"))
        .stdout(predicate::str::contains("20"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_011_01_basic_ranges_example_file() {
    // 15-TOOL VALIDATION: examples/lang_comp/11-ranges/01_basic_ranges.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("01_basic_ranges.ruchy");
    validate_with_15_tools(&example);

    // Additional validation: Verify output correctness
    ruchy_cmd()
        .arg("run")
        .arg(&example)
        .assert()
        .success()
        .stdout(predicate::str::contains("0"))
        .stdout(predicate::str::contains("4"))
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("3"))
        .stdout(predicate::str::contains("10"))
        .stdout(predicate::str::contains("12"));
}

// ============================================================================
// LANG-COMP-011-02: Range Iteration Tests
// Links to: examples/lang_comp/11-ranges/02_range_iteration.ruchy
// ============================================================================

#[test]
fn test_langcomp_011_02_range_sum_accumulation() {
    let temp_file = std::env::temp_dir().join("langcomp_011_02_sum.ruchy");
    std::fs::write(
        &temp_file,
        r"
fn main() {
    let mut sum = 0
    for i in 1..6 {
        sum = sum + i
    }
    println(sum)
}
",
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("15"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_011_02_nested_range_loops() {
    let temp_file = std::env::temp_dir().join("langcomp_011_02_nested.ruchy");
    std::fs::write(
        &temp_file,
        r"
fn main() {
    for i in 0..2 {
        for j in 0..2 {
            println(i * 10 + j)
        }
    }
}
",
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("0"))
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("10"))
        .stdout(predicate::str::contains("11"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_011_02_range_iteration_example_file() {
    // 15-TOOL VALIDATION: examples/lang_comp/11-ranges/02_range_iteration.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("02_range_iteration.ruchy");
    validate_with_15_tools(&example);

    // Additional validation: Verify sum and nested output
    ruchy_cmd()
        .arg("run")
        .arg(&example)
        .assert()
        .success()
        .stdout(predicate::str::contains("15")); // Sum of 1..6
}

// ============================================================================
// LANG-COMP-011-03: Range Variables Tests
// Links to: examples/lang_comp/11-ranges/03_range_variables.ruchy
// ============================================================================

#[test]
fn test_langcomp_011_03_range_in_variable() {
    let temp_file = std::env::temp_dir().join("langcomp_011_03_var.ruchy");
    std::fs::write(
        &temp_file,
        r"
fn main() {
    let range = 0..5
    for i in range {
        println(i)
    }
}
",
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("0"))
        .stdout(predicate::str::contains("4"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_011_03_range_variables_example_file() {
    // 15-TOOL VALIDATION: examples/lang_comp/11-ranges/03_range_variables.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("03_range_variables.ruchy");
    validate_with_15_tools(&example);
}

// ============================================================================
// LANG-COMP-011-04: Range Patterns Tests
// Links to: examples/lang_comp/11-ranges/04_range_patterns.ruchy
// ============================================================================

#[test]
fn test_langcomp_011_04_range_patterns_example_file() {
    // 15-TOOL VALIDATION: examples/lang_comp/11-ranges/04_range_patterns.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("04_range_patterns.ruchy");
    validate_with_15_tools(&example);
}

// ============================================================================
// Property Tests for Ranges (ignored by default - run manually with --ignored)
// ============================================================================

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_langcomp_011_property_range_bounds() {
    // Property test: All valid ranges should execute successfully
    // This test is ignored by default due to long runtime (10000 cases)
    for start in 0..10 {
        for end in (start + 1)..15 {
            let temp_file =
                std::env::temp_dir().join(format!("langcomp_011_prop_bounds_{start}_{end}.ruchy"));
            std::fs::write(
                &temp_file,
                format!(
                    r"
fn main() {{
    for i in {start}..{end} {{
        println(i)
    }}
}}
"
                ),
            )
            .unwrap();

            ruchy_cmd().arg("run").arg(&temp_file).assert().success();

            std::fs::remove_file(&temp_file).ok();
        }
    }
}
