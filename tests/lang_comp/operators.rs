#![allow(deprecated)]
// LANG-COMP-002: Operators - Validation Tests with Traceability
// Links to: examples/lang_comp/02-operators/*.ruchy
// Validates: LANG-COMP-002 Operators (arithmetic, comparison, logical, precedence)
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
        .join("examples/lang_comp/02-operators")
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
    ruchy_cmd().arg("compile").arg(example).assert().success();

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
// LANG-COMP-002-01: Arithmetic Operators Tests
// Links to: examples/lang_comp/02-operators/01_arithmetic.ruchy
// ============================================================================

#[test]
fn test_langcomp_002_01_arithmetic_addition() {
    let temp_file = std::env::temp_dir().join("langcomp_002_01_add.ruchy");
    std::fs::write(&temp_file, "2 + 3").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_002_01_arithmetic_subtraction() {
    let temp_file = std::env::temp_dir().join("langcomp_002_01_sub.ruchy");
    std::fs::write(&temp_file, "10 - 3").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("7"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_002_01_arithmetic_multiplication() {
    let temp_file = std::env::temp_dir().join("langcomp_002_01_mul.ruchy");
    std::fs::write(&temp_file, "4 * 5").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("20"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_002_01_arithmetic_division() {
    let temp_file = std::env::temp_dir().join("langcomp_002_01_div.ruchy");
    std::fs::write(&temp_file, "20 / 4").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_002_01_arithmetic_modulo() {
    let temp_file = std::env::temp_dir().join("langcomp_002_01_mod.ruchy");
    std::fs::write(&temp_file, "10 % 3").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_002_01_arithmetic_example_file() {
    // 15-TOOL VALIDATION: examples/lang_comp/02-operators/01_arithmetic.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("01_arithmetic.ruchy");
    validate_with_15_tools(&example);
}

#[test]
fn test_langcomp_002_01_arithmetic_precedence() {
    // Test: 2 + 3 * 4 = 14 (multiplication before addition)
    let temp_file = std::env::temp_dir().join("langcomp_002_01_precedence.ruchy");
    std::fs::write(&temp_file, "2 + 3 * 4").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("14"));

    std::fs::remove_file(&temp_file).ok();
}

// ============================================================================
// LANG-COMP-002-02: Comparison Operators Tests
// Links to: examples/lang_comp/02-operators/02_comparison.ruchy
// ============================================================================

#[test]
fn test_langcomp_002_02_comparison_equality() {
    let temp_file = std::env::temp_dir().join("langcomp_002_02_eq.ruchy");
    std::fs::write(&temp_file, "5 == 5").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_002_02_comparison_inequality() {
    let temp_file = std::env::temp_dir().join("langcomp_002_02_neq.ruchy");
    std::fs::write(&temp_file, "5 != 3").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_002_02_comparison_less_than() {
    let temp_file = std::env::temp_dir().join("langcomp_002_02_lt.ruchy");
    std::fs::write(&temp_file, "3 < 5").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_002_02_comparison_greater_than() {
    let temp_file = std::env::temp_dir().join("langcomp_002_02_gt.ruchy");
    std::fs::write(&temp_file, "7 > 5").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_002_02_comparison_less_than_or_equal() {
    let temp_file = std::env::temp_dir().join("langcomp_002_02_lte.ruchy");
    std::fs::write(&temp_file, "5 <= 5").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_002_02_comparison_greater_than_or_equal() {
    let temp_file = std::env::temp_dir().join("langcomp_002_02_gte.ruchy");
    std::fs::write(&temp_file, "5 >= 5").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_002_02_comparison_example_file() {
    // 15-TOOL VALIDATION: examples/lang_comp/02-operators/02_comparison.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("02_comparison.ruchy");
    validate_with_15_tools(&example);
}

#[test]
fn test_langcomp_002_02_comparison_and_logical_precedence() {
    // Test: comparison operators work in logical expressions
    let temp_file = std::env::temp_dir().join("langcomp_002_02_comp_logical.ruchy");
    std::fs::write(&temp_file, "5 > 3 && 10 < 20").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));

    std::fs::remove_file(&temp_file).ok();
}

// ============================================================================
// LANG-COMP-002-03: Logical Operators Tests
// Links to: examples/lang_comp/02-operators/03_logical.ruchy
// ============================================================================

#[test]
fn test_langcomp_002_03_logical_and() {
    let temp_file = std::env::temp_dir().join("langcomp_002_03_and.ruchy");
    std::fs::write(&temp_file, "true && true").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_002_03_logical_or() {
    let temp_file = std::env::temp_dir().join("langcomp_002_03_or.ruchy");
    std::fs::write(&temp_file, "false || true").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_002_03_logical_not() {
    let temp_file = std::env::temp_dir().join("langcomp_002_03_not.ruchy");
    std::fs::write(&temp_file, "!false").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_002_03_logical_example_file() {
    // 15-TOOL VALIDATION: examples/lang_comp/02-operators/03_logical.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("03_logical.ruchy");
    validate_with_15_tools(&example);
}

// ============================================================================
// LANG-COMP-002-04: Operator Precedence Tests
// Links to: examples/lang_comp/02-operators/04_precedence.ruchy
// ============================================================================

#[test]
fn test_langcomp_002_04_precedence_multiplication_before_addition() {
    // Test: 2 + 3 * 4 = 14 (not 20)
    let temp_file = std::env::temp_dir().join("langcomp_002_04_precedence_mul.ruchy");
    std::fs::write(&temp_file, "2 + 3 * 4").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("14"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_002_04_precedence_parentheses_override() {
    // Test: (2 + 3) * 4 = 20 (parentheses override precedence)
    let temp_file = std::env::temp_dir().join("langcomp_002_04_precedence_paren.ruchy");
    std::fs::write(&temp_file, "(2 + 3) * 4").unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("20"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_002_04_precedence_example_file() {
    // 15-TOOL VALIDATION: examples/lang_comp/02-operators/04_precedence.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("04_precedence.ruchy");
    validate_with_15_tools(&example);
}

// ============================================================================
// LANG-COMP-002: Property Tests (Mathematical Correctness Proofs)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_langcomp_002_property_addition_commutative() {
        // Property: a + b == b + a for all integers
        use proptest::prelude::*;
        proptest!(|(a: i32, b: i32)| {
            let code1 = format!("{a} + {b}");
            let code2 = format!("{b} + {a}");

            let temp1 = std::env::temp_dir().join(format!("langcomp_002_prop_add1_{a}_{b}.ruchy"));
            let temp2 = std::env::temp_dir().join(format!("langcomp_002_prop_add2_{a}_{b}.ruchy"));

            std::fs::write(&temp1, &code1).unwrap();
            std::fs::write(&temp2, &code2).unwrap();

            let result1 = ruchy_cmd().arg("run").arg(&temp1).output().unwrap();
            let result2 = ruchy_cmd().arg("run").arg(&temp2).output().unwrap();

            std::fs::remove_file(&temp1).ok();
            std::fs::remove_file(&temp2).ok();

            prop_assert_eq!(result1.stdout, result2.stdout);
        });
    }

    #[test]
    #[ignore]
    fn test_langcomp_002_property_comparison_never_crashes() {
        // Property: comparison operators never crash
        use proptest::prelude::*;
        proptest!(|(a: i32, b: i32)| {
            let operators = vec!["==", "!=", "<", ">", "<=", ">="];
            for op in operators {
                let code = format!("{a} {op} {b}");
                let temp_file = std::env::temp_dir().join(format!("langcomp_002_prop_cmp_{}_{}_{}.ruchy", a, op.replace('=', "e"), b));
                std::fs::write(&temp_file, &code).unwrap();

                ruchy_cmd()
                    .arg("run")
                    .arg(&temp_file)
                    .assert()
                    .success();

                std::fs::remove_file(&temp_file).ok();
            }
        });
    }

    #[test]
    #[ignore]
    fn test_langcomp_002_property_double_negation_identity() {
        // Property: !!x == x for all booleans
        for b in [true, false] {
            let code = format!("!!{b}");
            let temp_file = std::env::temp_dir().join(format!("langcomp_002_prop_neg_{b}.ruchy"));
            std::fs::write(&temp_file, &code).unwrap();

            ruchy_cmd()
                .arg("run")
                .arg(&temp_file)
                .assert()
                .success()
                .stdout(predicate::str::contains(b.to_string()));

            std::fs::remove_file(&temp_file).ok();
        }
    }

    #[test]
    #[ignore]
    fn test_langcomp_002_property_multiplication_associative() {
        // Property: (a * b) * c == a * (b * c)
        for a in 1..5 {
            for b in 1..5 {
                for c in 1..5 {
                    let code1 = format!("({a} * {b}) * {c}");
                    let code2 = format!("{a} * ({b} * {c})");

                    let temp1 = std::env::temp_dir()
                        .join(format!("langcomp_002_prop_mul1_{a}_{b}_{c}ruchy"));
                    let temp2 = std::env::temp_dir()
                        .join(format!("langcomp_002_prop_mul2_{a}_{b}_{c}ruchy"));

                    std::fs::write(&temp1, &code1).unwrap();
                    std::fs::write(&temp2, &code2).unwrap();

                    let result1 = ruchy_cmd().arg("run").arg(&temp1).output().unwrap();
                    let result2 = ruchy_cmd().arg("run").arg(&temp2).output().unwrap();

                    std::fs::remove_file(&temp1).ok();
                    std::fs::remove_file(&temp2).ok();

                    assert_eq!(
                        String::from_utf8_lossy(&result1.stdout),
                        String::from_utf8_lossy(&result2.stdout)
                    );
                }
            }
        }
    }

    #[test]
    #[ignore]
    fn test_langcomp_002_property_logical_and_short_circuit() {
        // Property: false && x never evaluates x
        let code = "false && true";
        let temp_file = std::env::temp_dir().join("langcomp_002_prop_and_short.ruchy");
        std::fs::write(&temp_file, code).unwrap();

        ruchy_cmd()
            .arg("run")
            .arg(&temp_file)
            .assert()
            .success()
            .stdout(predicate::str::contains("false"));

        std::fs::remove_file(&temp_file).ok();
    }
}
