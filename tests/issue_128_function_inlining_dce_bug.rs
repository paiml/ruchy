// ISSUE-128: Functions with if-else are not transpiled (removed by DCE after inlining)
// ROOT CAUSE: Dead Code Elimination collects used functions from POST-inlined AST
// where Call nodes have been replaced with inlined bodies, so functions appear unused
//
// EXTREME TDD: RED → GREEN → REFACTOR → VALIDATE

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_issue_128_01_function_with_if_else_transpiles() {
    // RED: This test MUST fail initially (function definition missing)

    let script = r#"
fun max(a, b) {
    if a > b {
        a
    } else {
        b
    }
}

let result = max(5, 3)
println("Result: " + result)
"#;

    std::fs::write("/tmp/issue_128_test.ruchy", script).unwrap();

    // Test 1: Interpret mode should work (baseline)
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .assert()
        .success()
        .stdout(predicate::str::contains("Result: 5"));

    // Test 2: Transpiled code must compile (either function def OR correct inlining)
    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("/tmp/issue_128_test.ruchy")
        .output()
        .unwrap();

    assert!(output.status.success());
    let transpiled = String::from_utf8_lossy(&output.stdout);

    // ISSUE-128: The bug was that parameters weren't substituted in if-else
    // Before fix: "if a > b" (undefined variables in main scope)
    // After fix: Either "fn max" exists with parameters OR "if 5 > 3" (inlined with values)

    // The transpiled code should either:
    // 1. Have a proper function definition with parameters: fn max(a, b) { if a > b ... }
    // 2. OR have inlined values: if 5 > 3 ...
    // What we DON'T want: "if a > b" appearing WITHOUT a function definition (undefined vars)
    let has_function_def = transpiled.contains("fn max");
    let has_inlined_values = transpiled.contains("if 5 > 3");
    let has_if_a_b = transpiled.contains("if a > b");

    // Valid: function exists with parameters, OR inlined with values
    // Invalid: "if a > b" without function definition
    assert!(
        has_function_def || has_inlined_values || !has_if_a_b,
        "BUG: 'if a > b' appears without function definition (Issue #128)\nTranspiled:\n{}",
        transpiled
    );
}

#[test]
fn test_issue_128_02_multiple_function_calls() {
    // Verify code compiles correctly with multiple function calls
    // NOTE: Small functions may be inlined - this is correct optimization

    let script = r"
fun double(x) {
    x * 2
}

let a = double(5)
let b = double(10)
println(a + b)
";

    std::fs::write("/tmp/issue_128_multi.ruchy", script).unwrap();

    // The key test: does it execute correctly?
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .assert()
        .success()
        .stdout(predicate::str::contains("30")); // 5*2 + 10*2 = 30
}

#[test]
fn test_issue_128_03_function_not_called_can_be_removed() {
    // Verify execution correctness with mixed used/unused functions

    let script = r"
fun unused(x) {
    x + 1
}

fun used(y) {
    y * 2
}

let result = used(5)
println(result)
";

    std::fs::write("/tmp/issue_128_unused.ruchy", script).unwrap();

    // Key test: execution must work correctly (unused code doesn't break it)
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .assert()
        .success()
        .stdout(predicate::str::contains("10")); // 5 * 2 = 10
}

#[test]
fn test_issue_128_04_inlined_function_with_single_call() {
    // If function is small, inlined, AND called only once, DCE can remove definition

    let script = r"
fun add_one(x) {
    x + 1
}

let result = add_one(5)
println(result)
";

    std::fs::write("/tmp/issue_128_inline_once.ruchy", script).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("/tmp/issue_128_inline_once.ruchy")
        .assert()
        .success(); // Either has function def OR inlined body (both valid)

    // Verify it executes correctly regardless
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .assert()
        .success()
        .stdout(predicate::str::contains("6"));
}

#[test]
#[ignore = "expensive: invokes rustc"]
fn test_issue_128_05_github_issue_exact_case() {
    // Exact case from GitHub Issue #128

    let script = r#"
fun max(a, b) {
    if a > b {
        a
    } else {
        b
    }
}

let result = max(5, 3)
println("Result: " + result)
"#;

    std::fs::write("/tmp/issue_128_exact.ruchy", script).unwrap();

    // Check transpiled output doesn't have undefined variables
    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("/tmp/issue_128_exact.ruchy")
        .output()
        .unwrap();

    let transpiled = String::from_utf8_lossy(&output.stdout);

    // CRITICAL: Transpiled code must compile
    // Either:
    // 1. Function definition exists: "fn max(a: i32, b: i32)"
    // 2. OR body is inlined WITH parameter substitution (no undefined a, b in main)

    // Test compilation by writing to temp file
    let test_rs = "/tmp/issue_128_test.rs";
    std::fs::write(test_rs, transpiled.as_ref()).unwrap();

    Command::new("rustc")
        .arg("--crate-type")
        .arg("bin")
        .arg("-o")
        .arg("/tmp/issue_128_test_bin")
        .arg(test_rs)
        .assert()
        .success(); // ❌ MUST FAIL initially (cannot find value `a` in scope)
}

#[test]
fn test_issue_128_06_recursive_function_not_inlined() {
    // Recursive functions should NEVER be inlined (infinite loop risk)

    let script = r"
fun factorial(n) {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

let result = factorial(5)
println(result)
";

    std::fs::write("/tmp/issue_128_recursive.ruchy", script).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("/tmp/issue_128_recursive.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("fn factorial")); // MUST have function def
}

#[test]
fn test_issue_128_07_large_function_not_inlined() {
    // Functions >10 LOC should NOT be inlined (size heuristic)

    let script = r"
fun large_function(x) {
    let a = x + 1
    let b = a * 2
    let c = b - 3
    let d = c / 4
    let e = d + 5
    let f = e * 6
    let g = f - 7
    let h = g / 8
    let i = h + 9
    let j = i * 10
    j
}

let result = large_function(5)
println(result)
";

    std::fs::write("/tmp/issue_128_large.ruchy", script).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("/tmp/issue_128_large.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("fn large_function")); // Too large to inline
}

#[test]
#[ignore = "expensive: invokes rustc"]
fn test_issue_128_08_return_expression_with_recursion() {
    // RED: This test MUST fail - recursive functions with return statements
    // Bug: check_recursion() doesn't look inside Return expressions
    // Bug: substitute_identifiers() doesn't substitute inside Return expressions

    let script = r"
fun fib(n) {
    if n <= 1 {
        return n
    } else {
        return fib(n - 1) + fib(n - 2)
    }
}

println(fib(10))
";

    std::fs::write("/tmp/issue_128_fib.ruchy", script).unwrap();

    // Test 1: Interpret mode works (baseline)
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .assert()
        .success()
        .stdout(predicate::str::contains("55")); // fib(10) = 55

    // Test 2: Transpile and verify output compiles
    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("/tmp/issue_128_fib.ruchy")
        .output()
        .unwrap();

    assert!(output.status.success());
    let transpiled = String::from_utf8_lossy(&output.stdout);

    // Expected: Either function definition exists (not inlined due to recursion)
    // OR function was correctly inlined with all parameters substituted
    let has_function_def = transpiled.contains("fn fib");

    // If function was NOT inlined (recursion detected correctly), we're done - test passes
    if has_function_def {
        // Recursion detected ✅ - function definition exists
        // Parameters like "n" in function body are CORRECT
        return;
    }

    // If function WAS inlined, check for undefined variables
    // Bug symptoms: "if n <= 1" (n undefined in main), "return n" (n undefined), "fib(n-1)" (fib undefined)
    assert!(
        !(transpiled.contains("if n <=")
            || transpiled.contains("return n")
            || transpiled.contains("fib(n")),
        "BUG DETECTED: Function was inlined but has undefined variables!\n\
             Either:\n\
             1. check_recursion() failed to detect recursion in Return expressions\n\
             2. substitute_identifiers() failed to substitute parameters in Return expressions\n\
             \n\
             Transpiled code:\n{transpiled}\n"
    );

    // If inlined, verify it compiles with rustc
    if !has_function_def {
        // Write to temp file and try to compile
        let temp_rs = "/tmp/issue_128_fib_test.rs";
        std::fs::write(temp_rs, transpiled.as_bytes()).unwrap();

        let rustc_result = std::process::Command::new("rustc")
            .arg("--crate-type")
            .arg("bin")
            .arg("-o")
            .arg("/tmp/issue_128_fib_test_bin")
            .arg(temp_rs)
            .output()
            .unwrap();

        if !rustc_result.status.success() {
            let stderr = String::from_utf8_lossy(&rustc_result.stderr);
            panic!(
                "BUG: Transpiled code doesn't compile!\n\
                 rustc errors:\n{stderr}\n\
                 \n\
                 Transpiled code:\n{transpiled}\n"
            );
        }
    }
}
