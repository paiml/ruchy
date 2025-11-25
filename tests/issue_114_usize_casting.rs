// ISSUE-114: usize casting for .len() comparisons
// Traceability: GitHub Issue #114 (usize casting follow-up)
//
// Problem: Vec::len() returns usize, but comparisons with i32 variables fail
// Example: while primes.len() < count { ... } where count is i32
// Expected: Transpiler should cast i32 to usize for .len() comparisons
//
// Test Strategy (EXTREME TDD - RED Phase):
// 1. BENCH-008 pattern: vec.len() < i32_variable
// 2. All comparison operators: <, >, <=, >=, ==, !=
// 3. Both operand orders: len() < count AND count > len()
// 4. End-to-end: Full BENCH-008 compilation

use predicates::prelude::*;
use std::io::Write;

#[cfg(test)]
mod property_tests {

    #[test]
    #[ignore = "Run with: cargo test --test issue_114_usize_casting property_tests -- --ignored --nocapture"]
    fn property_all_comparison_operators_generate_usize_cast() {
        // Property: For ANY comparison operator, .len() comparisons should cast to usize
        let operators = vec!["<", ">", "<=", ">=", "==", "!="];

        for op in &operators {
            let input = format!(
                r"
fun test(n) {{
    let items = []
    items.len() {op} n
}}
"
            );

            let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
                .arg("transpile")
                .arg("-")
                .write_stdin(input.as_bytes())
                .assert()
                .success()
                .get_output()
                .stdout
                .clone();

            let rust_code = String::from_utf8(output).unwrap();
            assert!(
                rust_code.contains(&format!("items.len() {op} n as usize"))
                    || rust_code.contains(&format!("items.len() {op} (n as usize)")),
                "Operator {op} missing usize cast"
            );
        }
    }

    #[test]
    #[ignore = "Pending implementation of collection.len() type inference"]
    fn property_all_collection_types_get_usize_cast() {
        // Property: Vec, String, and any .len() call should get usize casting
        let collection_types = vec![("[]", "vec![]"), (r#""""#, r#"String::from("")"#)];

        for (ruchy_init, _rust_init) in collection_types {
            let input = format!(
                r"
fun test(max) {{
    let collection = {ruchy_init}
    collection.len() < max
}}
"
            );

            let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
                .arg("transpile")
                .arg("-")
                .write_stdin(input.as_bytes())
                .assert()
                .success()
                .get_output()
                .stdout
                .clone();

            let rust_code = String::from_utf8(output).unwrap();
            assert!(
                rust_code.contains("collection.len() < max as usize")
                    || rust_code.contains("collection.len() < (max as usize)"),
                "Collection type {ruchy_init:?} missing usize cast"
            );
        }
    }
}

#[test]
fn test_issue_114_usize_bench_008_pattern() {
    // BENCH-008 pattern: while primes.len() < count
    let input = r"
fun generate_primes(count) {
    let mut primes = []
    let mut candidate = 2

    while primes.len() < count {
        primes.push(candidate)
        candidate = candidate + 1
    }

    primes
}
";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        // Should cast count to usize for comparison with len()
        .stdout(
            predicate::str::contains("primes.len() < count as usize")
                .or(predicate::str::contains("primes.len() < (count as usize)")),
        );
}

#[test]
fn test_issue_114_usize_all_comparison_operators() {
    // Test all comparison operators with .len()
    let input = r#"
fun test_comparisons(target) {
    let mut items = []

    // All comparison operators should work with len()
    if items.len() < target {
        println("less than")
    }
    if items.len() > target {
        println("greater than")
    }
    if items.len() <= target {
        println("less or equal")
    }
    if items.len() >= target {
        println("greater or equal")
    }
    if items.len() == target {
        println("equal")
    }
    if items.len() != target {
        println("not equal")
    }
}
"#;

    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let rust_code = String::from_utf8(output).unwrap();

    // All comparison operators should have usize casting
    assert!(
        rust_code.contains("items.len() < target as usize")
            || rust_code.contains("items.len() < (target as usize)")
    );
    assert!(
        rust_code.contains("items.len() > target as usize")
            || rust_code.contains("items.len() > (target as usize)")
    );
    assert!(
        rust_code.contains("items.len() <= target as usize")
            || rust_code.contains("items.len() <= (target as usize)")
    );
    assert!(
        rust_code.contains("items.len() >= target as usize")
            || rust_code.contains("items.len() >= (target as usize)")
    );
    assert!(
        rust_code.contains("items.len() == target as usize")
            || rust_code.contains("items.len() == (target as usize)")
    );
    assert!(
        rust_code.contains("items.len() != target as usize")
            || rust_code.contains("items.len() != (target as usize)")
    );
}

#[test]
fn test_issue_114_usize_reversed_operand_order() {
    // Test when i32 variable is on the left side
    let input = r#"
fun test_reversed(limit) {
    let mut items = []

    // Should handle both operand orders
    if limit > items.len() {
        println("limit greater than length")
    }
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("limit as usize > items.len()")
                .or(predicate::str::contains("(limit as usize) > items.len()")),
        );
}

#[test]
#[ignore = "expensive: invokes rustc"]
fn test_issue_114_usize_bench_008_end_to_end() {
    // Full BENCH-008: Transpile → Compile → Execute
    let input = r"
fun is_prime(n) {
    if n < 2 {
        return false
    }
    if n == 2 {
        return true
    }
    if n % 2 == 0 {
        return false
    }

    let mut i = 3
    while i * i <= n {
        if n % i == 0 {
            return false
        }
        i = i + 2
    }
    true
}

fun generate_primes(count) {
    let mut primes = []
    let mut candidate = 2

    while primes.len() < count {
        if is_prime(candidate) {
            primes.push(candidate)
        }
        candidate = candidate + 1
    }

    primes
}

fun main() {
    let primes = generate_primes(100)
    println(primes.len())
}
";

    // Transpile to Rust
    let transpile_output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let rust_code = String::from_utf8(transpile_output).unwrap();

    // Verify usize casting is present
    assert!(
        rust_code.contains("primes.len() < count as usize")
            || rust_code.contains("primes.len() < (count as usize)"),
        "Missing usize cast in transpiled code"
    );

    // Write to temp file and compile with rustc
    let temp_file = std::env::temp_dir().join("bench_008_usize_test.rs");
    let mut file = std::fs::File::create(&temp_file).unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();
    drop(file);

    // Compile with rustc
    let compile_result = std::process::Command::new("rustc")
        .arg("--crate-type")
        .arg("bin")
        .arg("-o")
        .arg(temp_file.with_extension(""))
        .arg(&temp_file)
        .output()
        .expect("Failed to run rustc");

    assert!(
        compile_result.status.success(),
        "Compilation failed:\nSTDOUT:\n{}\nSTDERR:\n{}",
        String::from_utf8_lossy(&compile_result.stdout),
        String::from_utf8_lossy(&compile_result.stderr)
    );

    // Execute the binary
    let exec_result = std::process::Command::new(temp_file.with_extension(""))
        .output()
        .expect("Failed to execute binary");

    assert!(exec_result.status.success(), "Execution failed");

    let output = String::from_utf8(exec_result.stdout).unwrap();
    assert!(output.contains("100"), "Expected 100 primes, got: {output}");
}

#[test]
fn test_issue_114_usize_simple_len_comparison() {
    // Simplest possible test case
    let input = r"
fun test_len(n) {
    let items = [1, 2, 3]
    items.len() < n
}
";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("items.len() < n as usize")
                .or(predicate::str::contains("items.len() < (n as usize)")),
        );
}

#[test]
fn test_issue_114_usize_nested_in_while_loop() {
    // BENCH-008 pattern: .len() in while condition
    let input = r"
fun fill_to_size(target_size) {
    let mut collection = []
    let mut i = 0

    while collection.len() < target_size {
        collection.push(i)
        i = i + 1
    }

    collection
}
";

    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let rust_code = String::from_utf8(output).unwrap();

    // Verify usize casting in while condition
    assert!(
        rust_code.contains("collection.len() < target_size as usize")
            || rust_code.contains("collection.len() < (target_size as usize)"),
        "Missing usize cast in while condition"
    );
}

#[test]
fn test_issue_114_usize_multiple_len_calls() {
    // Test with multiple .len() calls in same function
    let input = r#"
fun compare_sizes(target) {
    let mut a = [1, 2]
    let mut b = [3, 4, 5]

    if a.len() < target {
        println("a too small")
    }
    if b.len() >= target {
        println("b big enough")
    }
}
"#;

    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let rust_code = String::from_utf8(output).unwrap();

    // Both .len() calls should have usize casting
    assert!(
        rust_code.contains("a.len() < target as usize")
            || rust_code.contains("a.len() < (target as usize)")
    );
    assert!(
        rust_code.contains("b.len() >= target as usize")
            || rust_code.contains("b.len() >= (target as usize)")
    );
}

#[test]
fn test_issue_114_usize_string_len() {
    // Test with String.len() as well (also returns usize)
    let input = r#"
fun check_string_length(max_len) {
    let text = "hello"
    text.len() < max_len
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("text.len() < max_len as usize")
                .or(predicate::str::contains("text.len() < (max_len as usize)")),
        );
}
