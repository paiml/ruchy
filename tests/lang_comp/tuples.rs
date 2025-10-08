// LANG-COMP-013: Tuples - Validation Tests with Traceability
// Links to: examples/lang_comp/13-tuples/*.ruchy
// Validates: LANG-COMP-013 Tuples (creation, indexing, destructuring, nested, functions)
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
        .join("examples/lang_comp/13-tuples")
        .join(relative_path)
}

/// 15-TOOL VALIDATION: Run ALL 15 native tools on example file
/// MANDATORY/BLOCKING: Test passes ONLY if all tools succeed
/// Skipped: Tool 3 (repl - interactive), Tool 15 (notebook - needs server)
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

    // TOOL 12: ruchy property-tests - Property-based testing
    ruchy_cmd()
        .arg("property-tests")
        .arg(example)
        .assert()
        .success();

    // TOOL 13: ruchy mutations - Mutation testing
    ruchy_cmd().arg("mutations").arg(example).assert().success();

    // TOOL 14: ruchy fuzz - Fuzz testing
    ruchy_cmd().arg("fuzz").arg(example).assert().success();

    // TOOL 15: ruchy notebook - SKIPPED (requires server)
}

// ==================== LANG-COMP-013-01: Basic Tuples ====================

#[test]
fn test_langcomp_013_01_basic_tuples_example_file() {
    let example = example_path("01_basic_tuples.ruchy");
    validate_with_15_tools(&example);
}

#[test]
fn test_langcomp_013_01_basic_tuple_creation() {
    ruchy_cmd()
        .write_stdin("let pair = (1, 2); println(pair)")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("(1, 2)"));
}

#[test]
fn test_langcomp_013_01_tuple_indexing() {
    ruchy_cmd()
        .write_stdin("let pair = (1, 2); println(pair.0)")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));
}

#[test]
fn test_langcomp_013_01_mixed_type_tuple() {
    ruchy_cmd()
        .write_stdin("let mixed = (42, \"hello\", true); println(mixed.1)")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));
}

#[test]
fn test_langcomp_013_01_unit_tuple() {
    ruchy_cmd()
        .write_stdin("let unit = (); println(unit)")
        .arg("repl")
        .assert()
        .success();
}

// ==================== LANG-COMP-013-02: Tuple Destructuring ====================

#[test]
fn test_langcomp_013_02_tuple_destructuring_example_file() {
    let example = example_path("02_tuple_destructuring.ruchy");
    validate_with_15_tools(&example);
}

#[test]
fn test_langcomp_013_02_basic_destructuring() {
    ruchy_cmd()
        .write_stdin("let (x, y) = (3, 4); println(x)")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_langcomp_013_02_match_pattern() {
    ruchy_cmd()
        .write_stdin("let point = (0, 0); match point { (0, 0) => println(\"Origin\"), _ => println(\"Not origin\") }")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("Origin"));
}

#[test]
fn test_langcomp_013_02_underscore_destructuring() {
    ruchy_cmd()
        .write_stdin("let triple = (1, 2, 3); let (first, _, third) = triple; println(first)")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));
}

// ==================== LANG-COMP-013-03: Tuples in Functions ====================

#[test]
fn test_langcomp_013_03_tuple_functions_example_file() {
    let example = example_path("03_tuple_functions.ruchy");
    validate_with_15_tools(&example);
}

#[test]
fn test_langcomp_013_03_function_returning_tuple() {
    ruchy_cmd()
        .write_stdin("fn get_coords() -> (i32, i32) { (100, 200) }; let coords = get_coords(); println(coords.0)")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("100"));
}

#[test]
fn test_langcomp_013_03_tuple_parameter() {
    ruchy_cmd()
        .write_stdin("fn print_point(point: (i32, i32)) { println(point.0) }; print_point((5, 10))")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
}

// ==================== LANG-COMP-013-04: Nested Tuples ====================

#[test]
fn test_langcomp_013_04_nested_tuples_example_file() {
    let example = example_path("04_nested_tuples.ruchy");
    validate_with_15_tools(&example);
}

#[test]
fn test_langcomp_013_04_basic_nested_tuple() {
    ruchy_cmd()
        .write_stdin("let nested = ((1, 2), (3, 4)); println(nested.0)")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("(1, 2)"));
}

#[test]
fn test_langcomp_013_04_deep_indexing() {
    ruchy_cmd()
        .write_stdin("let nested = ((1, 2), (3, 4)); let value = (nested.0).1; println(value)")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
}

#[test]
fn test_langcomp_013_04_triple_nesting() {
    ruchy_cmd()
        .write_stdin("let deep = (((1, 2), 3), 4); println(deep)")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("(((1, 2), 3), 4)"));
}

#[test]
fn test_langcomp_013_04_destructuring_nested() {
    ruchy_cmd()
        .write_stdin("let nested = ((1, 2), (3, 4)); let (a, b) = nested.0; println(a)")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));
}
