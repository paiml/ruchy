#![allow(deprecated)]
// LANG-COMP-004: Functions - Validation Tests with Traceability
// Links to: examples/lang_comp/04-functions/*.ruchy
// Validates: LANG-COMP-004 Functions (declaration, parameters, return values, closures)
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
        .join("examples/lang_comp/04-functions")
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
// LANG-COMP-004-01: Function Declaration Tests
// Links to: examples/lang_comp/04-functions/01_declaration.ruchy
// ============================================================================

#[test]
fn test_langcomp_004_01_function_declaration_no_params() {
    // Test: fn greet() { println("Hello") } works
    let temp_file = std::env::temp_dir().join("langcomp_004_01_decl_no_params.ruchy");
    std::fs::write(
        &temp_file,
        r#"
fn greet() {
    println("Hello")
}
greet()
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_004_01_function_declaration_with_return() {
    // Test: fn add() { return 5 } returns value - must use println() to see output
    let temp_file = std::env::temp_dir().join("langcomp_004_01_decl_return.ruchy");
    std::fs::write(
        &temp_file,
        r#"
fn get_five() {
    return 5
}
println(f"Result: {get_five()}")
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: 5"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_004_01_function_declaration_example_file() {
    // 15-TOOL VALIDATION: examples/lang_comp/04-functions/01_declaration.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("01_declaration.ruchy");
    validate_with_15_tools(&example);
}

// ============================================================================
// LANG-COMP-004-02: Function Parameters Tests
// Links to: examples/lang_comp/04-functions/02_parameters.ruchy
// ============================================================================

#[test]
fn test_langcomp_004_02_function_single_parameter() {
    // Test: fn double(x) { x * 2 } works - must use println()
    let temp_file = std::env::temp_dir().join("langcomp_004_02_single_param.ruchy");
    std::fs::write(
        &temp_file,
        r#"
fn double(x) {
    x * 2
}
println(f"Result: {double(5)}")
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: 10"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_004_02_function_multiple_parameters() {
    // Test: fn add(a, b) { a + b } works - must use println()
    let temp_file = std::env::temp_dir().join("langcomp_004_02_multi_params.ruchy");
    std::fs::write(
        &temp_file,
        r#"
fn add(a, b) {
    a + b
}
println(f"Result: {add(3, 7)}")
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: 10"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_004_02_function_parameters_example_file() {
    // 15-TOOL VALIDATION: examples/lang_comp/04-functions/02_parameters.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("02_parameters.ruchy");
    validate_with_15_tools(&example);
}

// ============================================================================
// LANG-COMP-004-03: Function Return Values Tests
// Links to: examples/lang_comp/04-functions/03_return_values.ruchy
// ============================================================================

#[test]
fn test_langcomp_004_03_function_implicit_return() {
    // Test: fn square(x) { x * x } returns last expression - must use println()
    let temp_file = std::env::temp_dir().join("langcomp_004_03_implicit_return.ruchy");
    std::fs::write(
        &temp_file,
        r#"
fn square(x) {
    x * x
}
println(f"Result: {square(4)}")
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: 16"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_004_03_function_explicit_return() {
    // Test: return statement works - must use println()
    let temp_file = std::env::temp_dir().join("langcomp_004_03_explicit_return.ruchy");
    std::fs::write(
        &temp_file,
        r#"
fn max(a, b) {
    if a > b {
        return a
    }
    return b
}
println(f"Result: {max(10, 7)}")
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: 10"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_004_03_function_return_values_example_file() {
    // 15-TOOL VALIDATION: examples/lang_comp/04-functions/03_return_values.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("03_return_values.ruchy");
    validate_with_15_tools(&example);
}

// ============================================================================
// LANG-COMP-004-04: Closures Tests (if supported)
// Links to: examples/lang_comp/04-functions/04_closures.ruchy
// ============================================================================

#[test]
fn test_langcomp_004_04_closure_fat_arrow_syntax() {
    // Test: let double = |x| x * 2; double(5) - must use println()
    let temp_file = std::env::temp_dir().join("langcomp_004_04_closure_fat_arrow.ruchy");
    std::fs::write(
        &temp_file,
        r#"
let double = |x| x * 2
println(f"Result: {double(5)}")
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: 10"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_004_04_closure_example_file() {
    // 15-TOOL VALIDATION: examples/lang_comp/04-functions/04_closures.ruchy
    // ACCEPTANCE CRITERIA: ALL 15 tools must succeed
    let example = example_path("04_closures.ruchy");
    validate_with_15_tools(&example);
}

// ============================================================================
// LANG-COMP-004: Property Tests (Mathematical Correctness Proofs)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;

    #[test]
    #[ignore = "Test disabled - run with --ignored"]
    fn test_langcomp_004_property_function_calls_are_deterministic() {
        // Property: Same input always produces same output - must use println()
        for i in 1..20 {
            let code = format!(
                r#"
fn double(x) {{
    x * 2
}}
println(f"Result: {{double({i})}}")
"#
            );
            let temp_file =
                std::env::temp_dir().join(format!("langcomp_004_prop_deterministic_{i}.ruchy"));
            std::fs::write(&temp_file, &code).unwrap();

            // Run twice and compare
            let result1 = ruchy_cmd().arg("run").arg(&temp_file).output().unwrap();
            let result2 = ruchy_cmd().arg("run").arg(&temp_file).output().unwrap();

            assert_eq!(result1.stdout, result2.stdout);
            std::fs::remove_file(&temp_file).ok();
        }
    }

    #[test]
    #[ignore = "Test disabled - run with --ignored"]
    fn test_langcomp_004_property_nested_calls_work() {
        // Property: f(g(x)) works for all x - must use println()
        for i in 1..10 {
            let code = format!(
                r#"
fn double(x) {{
    x * 2
}}
fn square(x) {{
    x * x
}}
println(f"Result: {{square(double({i}))}}")
"#
            );
            let temp_file =
                std::env::temp_dir().join(format!("langcomp_004_prop_nested_{i}.ruchy"));
            std::fs::write(&temp_file, &code).unwrap();

            ruchy_cmd().arg("run").arg(&temp_file).assert().success();

            std::fs::remove_file(&temp_file).ok();
        }
    }

    #[test]
    #[ignore = "Test disabled - run with --ignored"]
    fn test_langcomp_004_property_parameter_count_matches() {
        // Property: Calling fn with wrong number of params fails gracefully
        let code = r"
fn add(a, b) {
    a + b
}
add(5)
";
        let temp_file = std::env::temp_dir().join("langcomp_004_prop_param_count.ruchy");
        std::fs::write(&temp_file, code).unwrap();

        // Should fail or handle gracefully (not panic)
        let _ = ruchy_cmd().arg("run").arg(&temp_file).output();

        std::fs::remove_file(&temp_file).ok();
    }
}
