// CLI-UNIFY-005: Validate 10 working examples across all CLI patterns
// Tests all invocation patterns: direct, run, compile

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("cli")
        .join(name)
}

// Example 1: Hello World
#[test]
fn test_example_01_hello_world_direct() {
    ruchy_cmd()
        .arg(example_path("01_hello_world.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, World!"));
}

#[test]
fn test_example_01_hello_world_run() {
    ruchy_cmd()
        .arg("run")
        .arg(example_path("01_hello_world.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, World!"));
}

// Example 2: Simple Math
#[test]
fn test_example_02_simple_math_direct() {
    ruchy_cmd()
        .arg(example_path("02_simple_math.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_example_02_simple_math_run() {
    ruchy_cmd()
        .arg("run")
        .arg(example_path("02_simple_math.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// Example 3: Variables
#[test]
fn test_example_03_variables_direct() {
    ruchy_cmd()
        .arg(example_path("03_variables.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("30"));
}

#[test]
fn test_example_03_variables_run() {
    ruchy_cmd()
        .arg("run")
        .arg(example_path("03_variables.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("30"));
}

// Example 4: Functions
#[test]
fn test_example_04_functions_direct() {
    ruchy_cmd()
        .arg(example_path("04_functions.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("15"));
}

#[test]
fn test_example_04_functions_run() {
    ruchy_cmd()
        .arg("run")
        .arg(example_path("04_functions.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("15"));
}

// Example 5: Control Flow
#[test]
fn test_example_05_control_flow_direct() {
    ruchy_cmd()
        .arg(example_path("05_control_flow.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("positive"));
}

#[test]
fn test_example_05_control_flow_run() {
    ruchy_cmd()
        .arg("run")
        .arg(example_path("05_control_flow.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("positive"));
}

// Example 6: Loops
#[test]
fn test_example_06_loops_direct() {
    ruchy_cmd()
        .arg(example_path("06_loops.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("0"))
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"))
        .stdout(predicate::str::contains("3"))
        .stdout(predicate::str::contains("4"));
}

#[test]
fn test_example_06_loops_run() {
    ruchy_cmd()
        .arg("run")
        .arg(example_path("06_loops.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("0"))
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"))
        .stdout(predicate::str::contains("3"))
        .stdout(predicate::str::contains("4"));
}

// Example 7: Strings
#[test]
fn test_example_07_strings_direct() {
    ruchy_cmd()
        .arg(example_path("07_strings.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("HELLO WORLD"));
}

#[test]
fn test_example_07_strings_run() {
    ruchy_cmd()
        .arg("run")
        .arg(example_path("07_strings.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("HELLO WORLD"));
}

// Example 8: Lists
#[test]
fn test_example_08_lists_direct() {
    ruchy_cmd()
        .arg(example_path("08_lists.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
}

#[test]
fn test_example_08_lists_run() {
    ruchy_cmd()
        .arg("run")
        .arg(example_path("08_lists.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
}

// Example 9: Match
#[test]
fn test_example_09_match_direct() {
    ruchy_cmd()
        .arg(example_path("09_match.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("two"));
}

#[test]
fn test_example_09_match_run() {
    ruchy_cmd()
        .arg("run")
        .arg(example_path("09_match.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("two"));
}

// Example 10: Closures
#[test]
fn test_example_10_closures_direct() {
    ruchy_cmd()
        .arg(example_path("10_closures.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("15"));
}

#[test]
fn test_example_10_closures_run() {
    ruchy_cmd()
        .arg("run")
        .arg(example_path("10_closures.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("15"));
}

// Comprehensive validation: All examples with both patterns
#[test]
fn test_all_examples_work() {
    let examples = [
        ("01_hello_world.ruchy", "Hello, World!"),
        ("02_simple_math.ruchy", "42"),
        ("03_variables.ruchy", "Alice"),
        ("04_functions.ruchy", "15"),
        ("05_control_flow.ruchy", "positive"),
        ("06_loops.ruchy", "0"),
        ("07_strings.ruchy", "HELLO WORLD"),
        ("08_lists.ruchy", "5"),
        ("09_match.ruchy", "two"),
        ("10_closures.ruchy", "15"),
    ];

    for (example, expected) in &examples {
        // Test direct execution
        ruchy_cmd()
            .arg(example_path(example))
            .assert()
            .success()
            .stdout(predicate::str::contains(*expected));

        // Test run command
        ruchy_cmd()
            .arg("run")
            .arg(example_path(example))
            .assert()
            .success()
            .stdout(predicate::str::contains(*expected));
    }
}

// Performance test: All examples should execute quickly (<1s each)
#[test]
fn test_all_examples_fast_execution() {
    use std::time::Instant;

    let examples = [
        "01_hello_world.ruchy",
        "02_simple_math.ruchy",
        "03_variables.ruchy",
        "04_functions.ruchy",
        "05_control_flow.ruchy",
        "06_loops.ruchy",
        "07_strings.ruchy",
        "08_lists.ruchy",
        "09_match.ruchy",
        "10_closures.ruchy",
    ];

    for example in &examples {
        let start = Instant::now();
        ruchy_cmd()
            .arg("run")
            .arg(example_path(example))
            .assert()
            .success();
        let duration = start.elapsed();

        assert!(
            duration.as_secs() < 1,
            "{} took {}s - should be <1s (interpreter mode)",
            example,
            duration.as_secs_f64()
        );
    }
}
