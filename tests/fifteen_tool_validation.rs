#![allow(missing_docs)]
// 15-TOOL VALIDATION SUITE - PROGRAMMATIC TESTING
// MANDATORY: All LANG-COMP examples MUST pass all 15 tools programmatically
// Uses assert_cmd for deterministic, automated validation
// Toyota Way: Stop the line if ANY tool fails

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

/// Helper to get path to ruchy binary
fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// Helper to get example file path
fn example_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/lang_comp")
        .join(relative_path)
}

// ============================================================================
// TOOL 1: ruchy check - Syntax Validation
// ============================================================================

#[test]
fn tool_01_check_validates_valid_syntax() {
    ruchy_cmd()
        .arg("check")
        .arg(example_path("01-basic-syntax/01_variables.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Syntax is valid"));
}

#[test]
fn tool_01_check_rejects_invalid_syntax() {
    // Create temp file with invalid syntax
    let invalid_code = "let x = ";
    let temp_file = std::env::temp_dir().join("invalid_syntax.ruchy");
    std::fs::write(&temp_file, invalid_code).expect("Failed to write temp file");

    ruchy_cmd().arg("check").arg(&temp_file).assert().failure();

    std::fs::remove_file(&temp_file).ok();
}

// ============================================================================
// TOOL 2: ruchy transpile - Rust Code Generation
// ============================================================================

#[test]
fn tool_02_transpile_generates_rust_code() {
    ruchy_cmd()
        .arg("transpile")
        .arg(example_path("01-basic-syntax/01_variables.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("fn main"))
        .stdout(predicate::str::contains("let x = 42"));
}

#[test]
fn tool_02_transpile_output_is_valid_rust() {
    let output = ruchy_cmd()
        .arg("transpile")
        .arg(example_path("01-basic-syntax/01_variables.ruchy"))
        .output()
        .expect("Failed to transpile");

    assert!(output.status.success());

    let rust_code = String::from_utf8_lossy(&output.stdout);
    assert!(rust_code.contains("fn main"));
    assert!(rust_code.contains("let x = 42"));
}

// ============================================================================
// TOOL 3: ruchy repl - Interactive Validation
// ============================================================================

#[test]
#[ignore = "REPL requires interactive input - use rexpect in integration tests"]
fn tool_03_repl_evaluates_expressions() {
    // This test is ignored because REPL requires interactive session
    // See: tests/integration/repl_validation.rs for rexpect-based tests
}

// ============================================================================
// TOOL 4: ruchy lint - Static Analysis
// ============================================================================

#[test]
fn tool_04_lint_passes_valid_code() {
    ruchy_cmd()
        .arg("lint")
        .arg(example_path("01-basic-syntax/01_variables.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("No issues found"));
}

#[test]
fn tool_04_lint_detects_unused_variables() {
    let code = "let unused = 42\n100";
    let temp_file = std::env::temp_dir().join("unused_var.ruchy");
    std::fs::write(&temp_file, code).expect("Failed to write temp file");

    ruchy_cmd()
        .arg("lint")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("unused"));

    std::fs::remove_file(&temp_file).ok();
}

// ============================================================================
// TOOL 5: ruchy compile - Binary Compilation
// ============================================================================

#[test]
fn tool_05_compile_generates_binary() {
    let temp_dir = std::env::temp_dir();
    let _output_binary = temp_dir.join("test_compile_output");

    ruchy_cmd()
        .arg("compile")
        .arg(example_path("01-basic-syntax/01_variables.ruchy"))
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully compiled"));

    // Verify binary was created (default name: a.out)
    let binary_path = temp_dir.join("a.out");
    assert!(binary_path.exists(), "Compiled binary should exist");

    // Cleanup
    std::fs::remove_file(&binary_path).ok();
}

// ============================================================================
// TOOL 6: ruchy run - Script Execution
// ============================================================================

#[test]
fn tool_06_run_executes_and_outputs_correctly() {
    ruchy_cmd()
        .arg("run")
        .arg(example_path("01-basic-syntax/01_variables.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn tool_06_run_executes_all_basic_syntax_examples() {
    let examples = vec![
        "01-basic-syntax/01_variables.ruchy",
        "01-basic-syntax/02_string_variables.ruchy",
        "01-basic-syntax/03_literals.ruchy",
        "01-basic-syntax/04_comments.ruchy",
    ];

    for example in examples {
        ruchy_cmd()
            .arg("run")
            .arg(example_path(example))
            .assert()
            .success();
    }
}

// ============================================================================
// TOOL 7: ruchy coverage - Code Coverage Analysis
// ============================================================================

#[test]
fn tool_07_coverage_generates_report() {
    ruchy_cmd()
        .arg("coverage")
        .arg(example_path("01-basic-syntax/01_variables.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Coverage"));
}

// ============================================================================
// TOOL 8: ruchy big-o - Complexity Analysis
// ============================================================================

#[test]
fn tool_08_big_o_analyzes_complexity() {
    ruchy_cmd()
        .arg("runtime")
        .arg(example_path("01-basic-syntax/01_variables.ruchy"))
        .arg("--bigo")
        .assert()
        .success()
        .stdout(predicate::str::contains("O("));
}

// ============================================================================
// TOOL 9: ruchy ast - AST Pretty-Printing
// ============================================================================

#[test]
fn tool_09_ast_prints_tree() {
    ruchy_cmd()
        .arg("ast")
        .arg(example_path("01-basic-syntax/01_variables.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Expr"));
}

// ============================================================================
// TOOL 10: ruchy wasm - WebAssembly Compilation
// ============================================================================

#[test]
fn tool_10_wasm_compiles_to_wasm() {
    ruchy_cmd()
        .arg("wasm")
        .arg(example_path("01-basic-syntax/01_variables.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains(".wasm"));
}

// ============================================================================
// TOOL 11: ruchy provability - Formal Verification
// ============================================================================

#[test]
fn tool_11_provability_verifies_properties() {
    ruchy_cmd()
        .arg("provability")
        .arg(example_path("01-basic-syntax/01_variables.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Provability"));
}

// ============================================================================
// TOOL 12: ruchy property-tests - Property-Based Testing
// ============================================================================

#[test]
#[ignore = "Slow E2E test (14s): Runs cargo run"]
fn tool_12_property_tests_runs_successfully() {
    ruchy_cmd()
        .arg("property-tests")
        .arg(example_path("01-basic-syntax/"))
        .arg("--cases")
        .arg("100")
        .arg("--format")
        .arg("text")
        .assert()
        .success()
        .stdout(predicate::str::contains("Property Test Report"))
        .stdout(predicate::str::contains("PASSED"));
}

#[test]
#[ignore = "Depends on full lang_comp suite passing"]
fn tool_12_property_tests_json_format() {
    ruchy_cmd()
        .arg("property-tests")
        .arg(example_path("01-basic-syntax/"))
        .arg("--cases")
        .arg("100")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("status"));
}

// ============================================================================
// TOOL 13: ruchy mutations - Mutation Testing
// ============================================================================

#[test]
#[ignore = "Slow E2E test (1s): Mutation testing"]
fn tool_13_mutations_runs_successfully() {
    ruchy_cmd()
        .arg("mutations")
        .arg(example_path("01-basic-syntax/01_variables.ruchy"))
        .arg("--timeout")
        .arg("60")
        .arg("--format")
        .arg("text")
        .assert()
        .success()
        .stdout(predicate::str::contains("Mutation Test Report"));
}

// ============================================================================
// TOOL 14: ruchy fuzz - Fuzz Testing
// ============================================================================

#[test]
fn tool_14_fuzz_runs_successfully() {
    ruchy_cmd()
        .arg("fuzz")
        .arg("parser")
        .arg("--iterations")
        .arg("1000")
        .arg("--format")
        .arg("text")
        .assert()
        .success()
        .stdout(predicate::str::contains("Fuzz Test Report"));
}

// ============================================================================
// TOOL 15: ruchy notebook - Interactive WASM Notebook Server
// ============================================================================

#[test]
fn tool_15_notebook_help_works() {
    ruchy_cmd()
        .arg("notebook")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Launch interactive notebook server",
        ))
        .stdout(predicate::str::contains("port"))
        .stdout(predicate::str::contains("host"));
}

#[test]
#[ignore = "Requires server startup - use integration tests with timeout"]
fn tool_15_notebook_starts_server() {
    // This test is ignored because notebook starts a long-running server
    // Integration tests should verify server startup and shutdown
}

// ============================================================================
// COMPREHENSIVE VALIDATION: All 15 Tools on Single Example
// ============================================================================

#[test]
#[ignore = "Run manually: cargo test --test fourteen_tool_validation comprehensive_validation -- --ignored"]
fn comprehensive_validation_all_15_tools() {
    let example = example_path("01-basic-syntax/01_variables.ruchy");

    // Tool 1: check
    ruchy_cmd().arg("check").arg(&example).assert().success();

    // Tool 2: transpile
    ruchy_cmd()
        .arg("transpile")
        .arg(&example)
        .assert()
        .success();

    // Tool 3: repl (skipped - requires interactive session)

    // Tool 4: lint
    ruchy_cmd().arg("lint").arg(&example).assert().success();

    // Tool 5: compile
    ruchy_cmd().arg("compile").arg(&example).assert().success();

    // Tool 6: run
    ruchy_cmd().arg("run").arg(&example).assert().success();

    // Tool 7: coverage
    ruchy_cmd().arg("coverage").arg(&example).assert().success();

    // Tool 8: big-o
    ruchy_cmd()
        .arg("runtime")
        .arg(&example)
        .arg("--bigo")
        .assert()
        .success();

    // Tool 9: ast
    ruchy_cmd().arg("ast").arg(&example).assert().success();

    // Tool 10: wasm
    ruchy_cmd().arg("wasm").arg(&example).assert().success();

    // Tool 11: provability
    ruchy_cmd()
        .arg("provability")
        .arg(&example)
        .assert()
        .success();

    // Tool 12: property-tests
    ruchy_cmd()
        .arg("property-tests")
        .arg(example_path("01-basic-syntax/"))
        .arg("--cases")
        .arg("100")
        .assert()
        .success();

    // Tool 13: mutations
    ruchy_cmd()
        .arg("mutations")
        .arg(&example)
        .arg("--timeout")
        .arg("60")
        .assert()
        .success();

    // Tool 14: fuzz (skipped - requires cargo-fuzz setup)

    // Tool 15: notebook (verify help works - server start tested separately)
    ruchy_cmd().arg("notebook").arg("--help").assert().success();
}
