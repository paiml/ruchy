// tests/properties/runtime_properties.rs
// Property-based tests for runtime behavior and semantics

use std::fs;
use std::process::Command;

#[test]
fn test_arithmetic_properties() {
    // Property: Addition is commutative
    let test_cases = vec![(1, 2), (5, 10), (0, 42), (-5, 3), (100, -50)];

    for (a, b) in test_cases {
        let code1 = format!("println({} + {})", a, b);
        let code2 = format!("println({} + {})", b, a);

        let result1 = execute_and_get_output(&code1);
        let result2 = execute_and_get_output(&code2);

        assert_eq!(
            result1, result2,
            "Addition not commutative: {} + {} ≠ {} + {}",
            a, b, b, a
        );
    }
}

#[test]
fn test_string_concatenation_properties() {
    // Property: String concatenation is associative
    let test_cases = vec![("hello", " ", "world"), ("a", "b", "c"), ("", "test", "")];

    for (a, b, c) in test_cases {
        let code1 = format!("println((\"{}\" + \"{}\") + \"{}\")", a, b, c);
        let code2 = format!("println(\"{}\" + (\"{}\" + \"{}\"))", a, b, c);

        let result1 = execute_and_get_output(&code1);
        let result2 = execute_and_get_output(&code2);

        assert_eq!(
            result1, result2,
            "String concatenation not associative: ({} + {}) + {} ≠ {} + ({} + {})",
            a, b, c, a, b, c
        );
    }
}

#[test]
fn test_function_call_determinism() {
    // Property: Pure functions should be deterministic
    let pure_function_code = r#"
fn add(x, y) {
    x + y  
}
println(add(5, 3))
"#;

    // Run the same code multiple times
    let results: Vec<_> = (0..5)
        .map(|_| execute_and_get_output(pure_function_code))
        .collect();

    // All results should be identical
    let first_result = &results[0];
    for (i, result) in results.iter().enumerate() {
        assert_eq!(
            result, first_result,
            "Function call not deterministic: run {} produced different result",
            i
        );
    }
}

#[test]
fn test_variable_scoping_properties() {
    // Property: Local variables shouldn't affect global scope
    let scoping_code = r#"
let x = 10
fn test() {
    let x = 20
    println(x)  // Should print 20
}
test()
println(x)  // Should print 10
"#;

    let output = execute_and_get_output(scoping_code);
    let lines: Vec<&str> = output.trim().split('\n').collect();

    assert_eq!(lines.len(), 2, "Expected exactly 2 output lines");
    assert_eq!(lines[0], "20", "Local variable value incorrect");
    assert_eq!(
        lines[1], "10",
        "Global variable value affected by local scope"
    );
}

fn execute_and_get_output(code: &str) -> String {
    let test_file = "/tmp/property_test.ruchy";
    fs::write(test_file, code).expect("Failed to write test file");

    let output = Command::new("./target/release/ruchy")
        .arg("run")
        .arg(test_file)
        .output()
        .expect("Failed to execute ruchy");

    if !output.status.success() {
        panic!(
            "Code execution failed: {}\nCode: {}",
            String::from_utf8_lossy(&output.stderr),
            code
        );
    }

    String::from_utf8_lossy(&output.stdout).to_string()
}
