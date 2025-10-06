// LANG-COMP-005: String Interpolation - Validation Tests with Traceability
// Links to: examples/lang_comp/05-string-interpolation/*.ruchy
// Validates: LANG-COMP-005 String Interpolation (basic, expressions, functions, nested)
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
        .join("examples/lang_comp/05-string-interpolation")
        .join(relative_path)
}

// ============================================================================
// LANG-COMP-005-01: Basic String Interpolation Tests
// Links to: examples/lang_comp/05-string-interpolation/01_basic_interpolation.ruchy
// ============================================================================

#[test]
fn test_langcomp_005_01_basic_variable_interpolation() {
    // Test: f"Hello, {name}" works with string variables - must use println()
    let temp_file = std::env::temp_dir().join("langcomp_005_01_basic_var.ruchy");
    std::fs::write(
        &temp_file,
        r#"
let name = "World"
println(f"Hello, {name}!")
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, World!"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_005_01_basic_integer_interpolation() {
    // Test: f"Number: {x}" works with integers - must use println()
    let temp_file = std::env::temp_dir().join("langcomp_005_01_basic_int.ruchy");
    std::fs::write(
        &temp_file,
        r#"
let x = 42
println(f"The answer is {x}")
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("The answer is 42"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_005_01_multiple_interpolations() {
    // Test: f"{a} and {b}" works with multiple variables - must use println()
    let temp_file = std::env::temp_dir().join("langcomp_005_01_multiple.ruchy");
    std::fs::write(
        &temp_file,
        r#"
let name = "Alice"
let age = 30
println(f"Hello, {name}! You are {age} years old.")
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Hello, Alice! You are 30 years old.",
        ));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_005_01_basic_interpolation_example_file() {
    // Validates: examples/lang_comp/05-string-interpolation/01_basic_interpolation.ruchy
    ruchy_cmd()
        .arg("run")
        .arg(example_path("01_basic_interpolation.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Name: Alice"))
        .stdout(predicate::str::contains("Age: 30"))
        .stdout(predicate::str::contains(
            "Hello, Alice! You are 30 years old.",
        ));
}

// ============================================================================
// LANG-COMP-005-02: Expression Interpolation Tests
// Links to: examples/lang_comp/05-string-interpolation/02_expressions.ruchy
// ============================================================================

#[test]
fn test_langcomp_005_02_arithmetic_expression_interpolation() {
    // Test: f"{x + y}" evaluates arithmetic expressions - must use println()
    let temp_file = std::env::temp_dir().join("langcomp_005_02_arithmetic.ruchy");
    std::fs::write(
        &temp_file,
        r#"
let x = 10
let y = 20
println(f"x + y = {x + y}")
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("x + y = 30"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_005_02_comparison_expression_interpolation() {
    // Test: f"{x > y}" evaluates comparison expressions - must use println()
    let temp_file = std::env::temp_dir().join("langcomp_005_02_comparison.ruchy");
    std::fs::write(
        &temp_file,
        r#"
let x = 10
let y = 20
println(f"x > y is {x > y}")
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("x > y is false"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_005_02_complex_expression_interpolation() {
    // Test: f"{x + y * 2}" follows operator precedence - must use println()
    let temp_file = std::env::temp_dir().join("langcomp_005_02_complex.ruchy");
    std::fs::write(
        &temp_file,
        r#"
let x = 10
let y = 20
println(f"Result: {x + y * 2}")
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: 50"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_005_02_expression_interpolation_example_file() {
    // Validates: examples/lang_comp/05-string-interpolation/02_expressions.ruchy
    ruchy_cmd()
        .arg("run")
        .arg(example_path("02_expressions.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("x + y = 30"))
        .stdout(predicate::str::contains("x * y = 200"))
        .stdout(predicate::str::contains("x > y is false"))
        .stdout(predicate::str::contains("Result: 50"));
}

// ============================================================================
// LANG-COMP-005-03: Function Call Interpolation Tests
// Links to: examples/lang_comp/05-string-interpolation/03_function_calls.ruchy
// ============================================================================

#[test]
fn test_langcomp_005_03_function_call_interpolation() {
    // Test: f"{func(x)}" calls function and interpolates result - must use println()
    let temp_file = std::env::temp_dir().join("langcomp_005_03_func_call.ruchy");
    std::fs::write(
        &temp_file,
        r#"
fn double(x) {
    x * 2
}
let num = 21
println(f"double({num}) = {double(num)}")
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("double(21) = 42"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_005_03_function_with_interpolated_result() {
    // Test: Interpolating function results works correctly - must use println()
    // NOTE: Functions returning strings need explicit type annotation (LANG-COMP future work)
    let temp_file = std::env::temp_dir().join("langcomp_005_03_func_result.ruchy");
    std::fs::write(
        &temp_file,
        r#"
fn add(a, b) {
    a + b
}
let result = add(10, 20)
println(f"Result: {result}")
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: 30"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_005_03_function_call_interpolation_example_file() {
    // Validates: examples/lang_comp/05-string-interpolation/03_function_calls.ruchy
    ruchy_cmd()
        .arg("run")
        .arg(example_path("03_function_calls.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("double(21) = 42"))
        .stdout(predicate::str::contains("add(10, 20) = 30"));
}

// ============================================================================
// LANG-COMP-005-04: Nested Interpolation Tests
// Links to: examples/lang_comp/05-string-interpolation/04_nested_interpolation.ruchy
// ============================================================================

#[test]
fn test_langcomp_005_04_nested_variable_interpolation() {
    // Test: f-strings can reference other f-string variables - must use println()
    let temp_file = std::env::temp_dir().join("langcomp_005_04_nested_var.ruchy");
    std::fs::write(
        &temp_file,
        r#"
let first = "John"
let last = "Doe"
let full_name = f"{first} {last}"
println(f"Full name: {full_name}")
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Full name: John Doe"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_005_04_interpolated_fstring_variable() {
    // Test: f-string variables can be interpolated into other f-strings - must use println()
    // NOTE: Direct f-string nesting not yet supported (parser limitation)
    let temp_file = std::env::temp_dir().join("langcomp_005_04_fstring_var.ruchy");
    std::fs::write(
        &temp_file,
        r#"
let name = "Alice"
let greeting = f"Hello, {name}!"
println(f"Message: {greeting}")
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Message: Hello, Alice!"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_005_04_nested_interpolation_example_file() {
    // Validates: examples/lang_comp/05-string-interpolation/04_nested_interpolation.ruchy
    ruchy_cmd()
        .arg("run")
        .arg(example_path("04_nested_interpolation.ruchy"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Full name: John Doe"))
        .stdout(predicate::str::contains("Greeting: Hello, John Doe!"));
}

// ============================================================================
// LANG-COMP-005: Property Tests (Mathematical Correctness Proofs)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_langcomp_005_property_interpolation_is_deterministic() {
        // Property: Same input always produces same output - must use println()
        for i in 1..20 {
            let code = format!(
                r#"
let x = {}
println(f"Value: {{x}}")
"#,
                i
            );
            let temp_file =
                std::env::temp_dir().join(format!("langcomp_005_prop_deterministic_{}.ruchy", i));
            std::fs::write(&temp_file, &code).unwrap();

            // Run twice and compare
            let result1 = ruchy_cmd().arg("run").arg(&temp_file).output().unwrap();
            let result2 = ruchy_cmd().arg("run").arg(&temp_file).output().unwrap();

            assert_eq!(result1.stdout, result2.stdout);
            std::fs::remove_file(&temp_file).ok();
        }
    }

    #[test]
    #[ignore]
    fn test_langcomp_005_property_expression_evaluation_in_interpolation() {
        // Property: f"{a + b}" equals println(a + b) for all a, b - must use println()
        for i in 1..10 {
            for j in 1..10 {
                let code = format!(
                    r#"
let a = {}
let b = {}
println(f"Result: {{a + b}}")
"#,
                    i, j
                );
                let temp_file = std::env::temp_dir()
                    .join(format!("langcomp_005_prop_expr_eval_{}_{}.ruchy", i, j));
                std::fs::write(&temp_file, &code).unwrap();

                let expected = format!("Result: {}", i + j);
                ruchy_cmd()
                    .arg("run")
                    .arg(&temp_file)
                    .assert()
                    .success()
                    .stdout(predicate::str::contains(expected));

                std::fs::remove_file(&temp_file).ok();
            }
        }
    }

    #[test]
    #[ignore]
    fn test_langcomp_005_property_multiple_interpolations_independent() {
        // Property: f"{a} {b}" equals concatenation of individual interpolations
        for i in 1..10 {
            let code = format!(
                r#"
let a = {}
let b = {}
println(f"{{a}} {{b}}")
"#,
                i,
                i * 2
            );
            let temp_file =
                std::env::temp_dir().join(format!("langcomp_005_prop_multi_interp_{}.ruchy", i));
            std::fs::write(&temp_file, &code).unwrap();

            let expected = format!("{} {}", i, i * 2);
            ruchy_cmd()
                .arg("run")
                .arg(&temp_file)
                .assert()
                .success()
                .stdout(predicate::str::contains(expected));

            std::fs::remove_file(&temp_file).ok();
        }
    }
}
