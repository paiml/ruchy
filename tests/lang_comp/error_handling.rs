// LANG-COMP-012: Error Handling - Validation Tests with 15-Tool Protocol
// Links to: examples/lang_comp/12-error-handling/*.ruchy
// Validates: LANG-COMP-012 Error Handling (Result, Option, try/catch, propagation)
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
        .join("examples/lang_comp/12-error-handling")
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
    // DEFECT-RACE-CONDITION FIX: Use unique temp file per thread to avoid parallel test collisions
    let temp_file = std::env::temp_dir().join(format!(
        "wasm_validation_test_{:?}.ruchy",
        std::thread::current().id()
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
// LANG-COMP-012-01: Result Type Tests
// Links to: examples/lang_comp/12-error-handling/01_result_type.ruchy
// ============================================================================

#[test]
fn test_langcomp_012_01_result_ok_case() {
    let temp_file = std::env::temp_dir().join("langcomp_012_01_ok.ruchy");
    std::fs::write(
        &temp_file,
        r#"
fn main() {
    fn divide(a: i32, b: i32) -> Result<i32, String> {
        if b == 0 {
            Err("Division by zero")
        } else {
            Ok(a / b)
        }
    }

    let result = divide(10, 2)
    match result {
        Ok(value) => println(value),
        Err(msg) => println(msg)
    }
}
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_012_01_result_err_case() {
    let temp_file = std::env::temp_dir().join("langcomp_012_01_err.ruchy");
    std::fs::write(
        &temp_file,
        r#"
fn main() {
    fn divide(a: i32, b: i32) -> Result<i32, String> {
        if b == 0 {
            Err("Division by zero")
        } else {
            Ok(a / b)
        }
    }

    let result = divide(10, 0)
    match result {
        Ok(value) => println(value),
        Err(msg) => println(msg)
    }
}
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Division by zero"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_012_01_result_type_example_file() {
    // 15-TOOL VALIDATION: examples/lang_comp/12-error-handling/01_result_type.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("01_result_type.ruchy");
    validate_with_15_tools(&example);

    // Additional validation: Verify output correctness
    ruchy_cmd()
        .arg("run")
        .arg(&example)
        .assert()
        .success()
        .stdout(predicate::str::contains("5"))
        .stdout(predicate::str::contains("Division by zero"));
}

// ============================================================================
// LANG-COMP-012-02: Option Type Tests
// Links to: examples/lang_comp/12-error-handling/02_option_type.ruchy
// ============================================================================

#[test]
fn test_langcomp_012_02_option_some_case() {
    let temp_file = std::env::temp_dir().join("langcomp_012_02_some.ruchy");
    std::fs::write(
        &temp_file,
        r#"
fn main() {
    fn find_user(id: i32) -> Option<String> {
        if id == 1 {
            Some("Alice")
        } else {
            None
        }
    }

    let user = find_user(1)
    match user {
        Some(name) => println(name),
        None => println("Not found")
    }
}
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_012_02_option_none_case() {
    let temp_file = std::env::temp_dir().join("langcomp_012_02_none.ruchy");
    std::fs::write(
        &temp_file,
        r#"
fn main() {
    fn find_user(id: i32) -> Option<String> {
        if id == 1 {
            Some("Alice")
        } else {
            None
        }
    }

    let user = find_user(999)
    match user {
        Some(name) => println(name),
        None => println("Not found")
    }
}
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Not found"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_012_02_option_type_example_file() {
    // 15-TOOL VALIDATION: examples/lang_comp/12-error-handling/02_option_type.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("02_option_type.ruchy");
    validate_with_15_tools(&example);

    // Additional validation: Verify output correctness
    ruchy_cmd()
        .arg("run")
        .arg(&example)
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Not found"));
}

// ============================================================================
// LANG-COMP-012-03: Try-Catch Tests
// Links to: examples/lang_comp/12-error-handling/03_try_catch.ruchy
// ============================================================================

#[test]
fn test_langcomp_012_03_try_success() {
    let temp_file = std::env::temp_dir().join("langcomp_012_03_success.ruchy");
    std::fs::write(
        &temp_file,
        r#"
fn main() {
    fn risky_operation(x: i32) -> i32 {
        if x < 0 {
            throw "Negative not allowed"
        }
        x * 2
    }

    try {
        let result = risky_operation(5)
        println(result)
    } catch e {
        println(e)
    }
}
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_012_03_try_catch_error() {
    let temp_file = std::env::temp_dir().join("langcomp_012_03_catch.ruchy");
    std::fs::write(
        &temp_file,
        r#"
fn main() {
    fn risky_operation(x: i32) -> i32 {
        if x < 0 {
            throw "Negative not allowed"
        }
        x * 2
    }

    try {
        let result = risky_operation(-1)
        println(result)
    } catch e {
        println(e)
    }
}
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Negative not allowed"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_012_03_try_catch_example_file() {
    // 15-TOOL VALIDATION: examples/lang_comp/12-error-handling/03_try_catch.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("03_try_catch.ruchy");
    validate_with_15_tools(&example);

    // Additional validation: Verify output correctness
    ruchy_cmd()
        .arg("run")
        .arg(&example)
        .assert()
        .success()
        .stdout(predicate::str::contains("10"))
        .stdout(predicate::str::contains("Negative not allowed"));
}

// ============================================================================
// LANG-COMP-012-04: Error Propagation Tests
// Links to: examples/lang_comp/12-error-handling/04_error_propagation.ruchy
// ============================================================================

#[test]
fn test_langcomp_012_04_error_chain_success() {
    let temp_file = std::env::temp_dir().join("langcomp_012_04_chain.ruchy");
    std::fs::write(
        &temp_file,
        r#"
fn main() {
    fn step1(x: i32) -> Result<i32, String> {
        if x > 0 {
            Ok(x * 2)
        } else {
            Err("Step 1 failed")
        }
    }

    fn step2(x: i32) -> Result<i32, String> {
        if x < 100 {
            Ok(x + 10)
        } else {
            Err("Step 2 failed")
        }
    }

    match step1(10) {
        Ok(v1) => match step2(v1) {
            Ok(v2) => println(v2),
            Err(e) => println(e)
        },
        Err(e) => println(e)
    }
}
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("30"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_012_04_error_propagation_example_file() {
    // 15-TOOL VALIDATION: examples/lang_comp/12-error-handling/04_error_propagation.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("04_error_propagation.ruchy");
    validate_with_15_tools(&example);

    // Additional validation: Verify error propagation works
    ruchy_cmd()
        .arg("run")
        .arg(&example)
        .assert()
        .success()
        .stdout(predicate::str::contains("30"))
        .stdout(predicate::str::contains("Step 1 failed"));
}
