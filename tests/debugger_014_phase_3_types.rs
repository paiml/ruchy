//! Tests for DEBUGGER-014 Phase 3: Type-aware tracing
//! GitHub Issue: <https://github.com/paiml/ruchy/issues/84>
//!
//! Phase 3 enhances Phase 2 (argument/return values) with type information:
//! - Before: TRACE: → square(5)
//! - After:  TRACE: → square(5: integer)
//!
//! Test naming convention: `test_debugger_014_phase_3`_<feature>_<scenario>

use assert_cmd::Command;
use predicates::prelude::*;

/// Test #1: Trace integer argument with type annotation
///
/// RED phase: This test MUST FAIL initially because type annotations aren't implemented
#[test]
fn test_debugger_014_phase_3_integer_argument_type() {
    let code = r#"
fun square(x) {
    return x * x;
}

fun main() {
    let result = square(5);
    println("Result: {}", result);
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("TRACE: → square(5: integer)"))
        .stdout(predicate::str::contains("TRACE: ← square = 25: integer"));
}

/// Test #2: Trace string argument with type annotation
///
/// RED phase: This test MUST FAIL initially
#[test]
fn test_debugger_014_phase_3_string_argument_type() {
    let code = r#"
fun greet(name) {
    return "Hello, " + name;
}

fun main() {
    let result = greet("Alice");
    println(result);
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("TRACE: → greet(\"Alice\": string)"))
        .stdout(predicate::str::contains("TRACE: ← greet = \"Hello, Alice\": string"));
}

/// Test #3: Trace multiple arguments with different types
///
/// RED phase: This test MUST FAIL initially
#[test]
fn test_debugger_014_phase_3_multiple_argument_types() {
    let code = r#"
fun format_message(count, message) {
    return message + " (" + count.to_string() + ")";
}

fun main() {
    let result = format_message(42, "items");
    println(result);
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("TRACE: → format_message(42: integer, \"items\": string)"))
        .stdout(predicate::str::contains("TRACE: ← format_message = \"items (42)\": string"));
}

/// Test #4: Trace float argument with type annotation
///
/// RED phase: This test MUST FAIL initially
#[test]
fn test_debugger_014_phase_3_float_argument_type() {
    let code = r#"
fun area(radius) {
    return 3.14159 * radius * radius;
}

fun main() {
    let result = area(2.5);
    println("Area: {}", result);
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("TRACE: → area(2.5: float)"))
        .stdout(predicate::str::contains("TRACE: ← area = ")); // Float result, don't assert exact value
}

/// Test #5: Trace boolean argument with type annotation
///
/// RED phase: This test MUST FAIL initially
#[test]
fn test_debugger_014_phase_3_boolean_argument_type() {
    let code = r#"
fun negate(flag) {
    return !flag;
}

fun main() {
    let result = negate(true);
    println("Result: {}", result);
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("TRACE: → negate(true: boolean)"))
        .stdout(predicate::str::contains("TRACE: ← negate = false: boolean"));
}

/// Test #6: Trace array argument with type annotation
///
/// RED phase: This test MUST FAIL initially
#[test]
fn test_debugger_014_phase_3_array_argument_type() {
    let code = r#"
fun length(arr) {
    return arr.length();
}

fun main() {
    let items = [1, 2, 3];
    let result = length(items);
    println("Length: {}", result);
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("TRACE: → length([1, 2, 3]: array)"))
        .stdout(predicate::str::contains("TRACE: ← length = 3: integer"));
}

/// Test #7: Backward compatibility - Phase 2 tests still pass
///
/// This verifies that adding type annotations doesn't break existing Phase 2 functionality
#[test]
fn test_debugger_014_phase_3_backward_compatible_with_phase_2() {
    let code = r#"
fun add(a, b) {
    return a + b;
}

fun main() {
    let result = add(10, 20);
    println("Result: {}", result);
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        // Phase 2 format still works (values present)
        .stdout(predicate::str::contains("TRACE: → add(10"))
        .stdout(predicate::str::contains("20"))
        .stdout(predicate::str::contains("TRACE: ← add = 30"))
        // Phase 3 format present (types added)
        .stdout(predicate::str::contains(": integer"));
}

/// Test #8: Recursive calls with type annotations
///
/// RED phase: This test MUST FAIL initially
#[test]
fn test_debugger_014_phase_3_recursive_with_types() {
    let code = r#"
fun factorial(n) {
    if n <= 1 {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}

fun main() {
    let result = factorial(3);
    println("Result: {}", result);
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("TRACE: → factorial(3: integer)"))
        .stdout(predicate::str::contains("TRACE: → factorial(2: integer)"))
        .stdout(predicate::str::contains("TRACE: → factorial(1: integer)"))
        .stdout(predicate::str::contains("TRACE: ← factorial = 1: integer"))
        .stdout(predicate::str::contains("TRACE: ← factorial = 2: integer"))
        .stdout(predicate::str::contains("TRACE: ← factorial = 6: integer"));
}
