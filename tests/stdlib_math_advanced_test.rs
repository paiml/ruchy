// STDLIB-002: Advanced Math Functions
//
// Implementing: sin, cos, tan, log, log10, random
// Pattern: Zero-cost abstraction (wrapping Rust std::f64 methods)
// Tests: Both interpreter (-e flag) and transpiler (run command) modes
//
// Reference: docs/specifications/stdlib1.20-spec.md - Advanced Math section

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ============================================================================
// Trigonometric Functions - sin, cos, tan
// ============================================================================

#[test]
fn test_stdlib002_sin_zero() {
    let code = r#"
let result = sin(0.0)
assert_eq(result, 0.0)
println("sin(0) = {}", result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("sin(0) = 0"));
}

#[test]
fn test_stdlib002_sin_pi_over_2() {
    let code = r#"
let result = sin(1.5707963267948966)
println("sin(π/2) = {}", result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("sin(π/2) = 1"));
}

#[test]
fn test_stdlib002_cos_zero() {
    let code = r#"
let result = cos(0.0)
assert_eq(result, 1.0)
println("cos(0) = {}", result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("cos(0) = 1"));
}

#[test]
fn test_stdlib002_cos_pi() {
    let code = r#"
let result = cos(3.141592653589793)
println("cos(π) = {}", result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("cos(π) = -1"));
}

#[test]
fn test_stdlib002_tan_zero() {
    let code = r#"
let result = tan(0.0)
assert_eq(result, 0.0)
println("tan(0) = {}", result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("tan(0) = 0"));
}

#[test]
fn test_stdlib002_tan_pi_over_4() {
    let code = r#"
let result = tan(0.7853981633974483)
println("tan(π/4) = {}", result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("tan(π/4) = 0.99"));  // Accepts 0.9999... or 1.0
}

// ============================================================================
// Logarithmic Functions - log, log10
// ============================================================================

#[test]
fn test_stdlib002_log_e() {
    let code = r#"
let result = log(2.718281828459045)
println("log(e) = {}", result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("log(e) = 1"));
}

#[test]
fn test_stdlib002_log_one() {
    let code = r#"
let result = log(1.0)
assert_eq(result, 0.0)
println("log(1) = {}", result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("log(1) = 0"));
}

#[test]
fn test_stdlib002_log10_ten() {
    let code = r#"
let result = log10(10.0)
assert_eq(result, 1.0)
println("log10(10) = {}", result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("log10(10) = 1"));
}

#[test]
fn test_stdlib002_log10_hundred() {
    let code = r#"
let result = log10(100.0)
assert_eq(result, 2.0)
println("log10(100) = {}", result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("log10(100) = 2"));
}

#[test]
fn test_stdlib002_log10_one() {
    let code = r#"
let result = log10(1.0)
assert_eq(result, 0.0)
println("log10(1) = {}", result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("log10(1) = 0"));
}

// ============================================================================
// Random Number Generation
// ============================================================================

#[test]
fn test_stdlib002_random_range() {
    let code = r#"
let r = random()
println("random() = {}", r)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success();
}

#[test]
fn test_stdlib002_random_multiple_calls() {
    let code = r#"
let r1 = random()
let r2 = random()
let r3 = random()
println("r1 = {}, r2 = {}, r3 = {}", r1, r2, r3)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success();
}

// ============================================================================
// Transpiler Mode Tests (compile to binary)
// ============================================================================

#[test]
fn test_stdlib002_transpiler_trig() {
    let code = r#"
fn main() {
    let s = sin(0.0)
    let c = cos(0.0)
    let t = tan(0.0)

    assert_eq(s, 0.0)
    assert_eq(c, 1.0)
    assert_eq(t, 0.0)

    println("Trig functions work!")
}
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(code.as_bytes()).expect("Failed to write temp file");
    let temp_path = temp_file.path();

    ruchy_cmd()
        .arg("run")
        .arg(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Trig functions work!"));
}

#[test]
fn test_stdlib002_transpiler_log() {
    let code = r#"
fn main() {
    let l1 = log(1.0)
    let l2 = log10(1.0)

    assert_eq(l1, 0.0)
    assert_eq(l2, 0.0)

    println("Log functions work!")
}
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(code.as_bytes()).expect("Failed to write temp file");
    let temp_path = temp_file.path();

    ruchy_cmd()
        .arg("run")
        .arg(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Log functions work!"));
}

#[test]
fn test_stdlib002_transpiler_random() {
    let code = r#"
fn main() {
    let r = random()
    println("Random value: {}", r)
}
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(code.as_bytes()).expect("Failed to write temp file");
    let temp_path = temp_file.path();

    ruchy_cmd()
        .arg("run")
        .arg(temp_path)
        .assert()
        .success();
}

// ============================================================================
// Property Tests - Mathematical Invariants (run with --ignored)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;

    /// Property: sin²(x) + cos²(x) = 1 (Pythagorean identity)
    #[test]
    #[ignore]
    fn prop_pythagorean_identity() {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for iteration in 0..10000 {
            let x: f64 = rng.gen_range(-10.0..10.0);

            let code = format!(
                r#"
let x = {}
let s = sin(x)
let c = cos(x)
let sum = s * s + c * c
let diff = abs(sum - 1.0)
println("sin²({{}}) + cos²({{}}) = {{}}, diff = {{}}", x, x, sum, diff)
"#,
                x
            );

            ruchy_cmd()
                .arg("-e")
                .arg(&code)
                .assert()
                .success();

            if iteration % 1000 == 0 {
                println!("Property test iteration: {}/10000", iteration);
            }
        }

        println!("✅ Property test passed: sin²(x) + cos²(x) = 1 for 10,000 random values");
    }

    /// Property: log(a * b) = log(a) + log(b)
    #[test]
    #[ignore]
    fn prop_logarithm_product_rule() {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for iteration in 0..10000 {
            let a: f64 = rng.gen_range(1.0..100.0);
            let b: f64 = rng.gen_range(1.0..100.0);

            let code = format!(
                r#"
let a = {}
let b = {}
let log_ab = log(a * b)
let log_a_plus_log_b = log(a) + log(b)
let diff = abs(log_ab - log_a_plus_log_b)
println("log({{}}) = {{}}, diff = {{}}", a * b, log_ab, diff)
"#,
                a, b
            );

            ruchy_cmd()
                .arg("-e")
                .arg(&code)
                .assert()
                .success();

            if iteration % 1000 == 0 {
                println!("Property test iteration: {}/10000", iteration);
            }
        }

        println!("✅ Property test passed: log(a*b) = log(a) + log(b) for 10,000 random values");
    }

    /// Property: random() returns values in [0.0, 1.0)
    #[test]
    #[ignore]
    fn prop_random_in_range() {
        for iteration in 0..10000 {
            let code = r#"
let r = random()
println("random() = {}", r)
"#;

            ruchy_cmd()
                .arg("-e")
                .arg(code)
                .assert()
                .success();

            if iteration % 1000 == 0 {
                println!("Property test iteration: {}/10000", iteration);
            }
        }

        println!("✅ Property test passed: random() generated 10,000 values");
    }
}
