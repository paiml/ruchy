// Golden Master Testing - Record expected outputs and detect any changes
// Inspired by SQLite's comprehensive test suite approach

use std::fs;
use std::process::Command;

struct GoldenTest {
    name: &'static str,
    code: &'static str,
    expected_stdout: &'static str,
    expected_stderr: &'static str,
    should_succeed: bool,
}

// Every language feature has a golden master test
const GOLDEN_TESTS: &[GoldenTest] = &[
    // Basic arithmetic - never allow regressions
    GoldenTest {
        name: "basic_math",
        code: "println(2 + 3 * 4)",
        expected_stdout: "14\n",
        expected_stderr: "",
        should_succeed: true,
    },
    
    // While loops - critical after our fix
    GoldenTest {
        name: "while_loop_boundary",
        code: "let i = 0; while i < 3 { println(i); i = i + 1 }",
        expected_stdout: "0\n1\n2\n",
        expected_stderr: "",
        should_succeed: true,
    },
    
    // Object.items() - critical after our fix
    GoldenTest {
        name: "object_items_simple",
        code: r#"let obj = {"a": 1}; for k, v in obj.items() { println(k); println(v) }"#,
        expected_stdout: "a\n1\n",
        expected_stderr: "",
        should_succeed: true,
    },
    
    // String operations
    GoldenTest {
        name: "string_interpolation",
        code: r#"let name = "World"; println(f"Hello {name}!")"#,
        expected_stdout: "Hello World!\n",
        expected_stderr: "",
        should_succeed: true,
    },
    
    // Pattern matching
    GoldenTest {
        name: "match_exhaustive",
        code: r#"let x = 2; println(match x { 1 => "one", 2 => "two", _ => "other" })"#,
        expected_stdout: "two\n",
        expected_stderr: "",
        should_succeed: true,
    },
    
    // Functions
    GoldenTest {
        name: "function_return",
        code: "fn double(x) { x * 2 }; println(double(21))",
        expected_stdout: "42\n", 
        expected_stderr: "",
        should_succeed: true,
    },
    
    // Arrays
    GoldenTest {
        name: "array_methods",
        code: r"let arr = [1, 2, 3]; println(arr.map(|x| x * 2))",
        expected_stdout: "[2, 4, 6]\n",
        expected_stderr: "",
        should_succeed: true,
    },
];

#[test]
fn test_all_golden_masters() {
    let mut failures = Vec::new();
    
    for test in GOLDEN_TESTS {
        let result = run_golden_test(test);
        if !result.passed {
            failures.push((test.name, result.error));
        }
    }
    
    if !failures.is_empty() {
        let mut error_msg = String::from("Golden master tests failed:\n");
        for (name, error) in failures {
            error_msg.push_str(&format!("  - {name}: {error}\n"));
        }
        panic!("{}", error_msg);
    }
}

struct GoldenResult {
    passed: bool,
    error: String,
}

fn run_golden_test(test: &GoldenTest) -> GoldenResult {
    // Write test to file
    let test_file = format!("/tmp/golden_{}.ruchy", test.name);
    fs::write(&test_file, test.code).unwrap();
    
    // Execute
    let output = Command::new("./target/release/ruchy")
        .arg(&test_file)
        .output()
        .expect("Failed to run ruchy");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();
    
    // Check exact match
    if success != test.should_succeed {
        return GoldenResult {
            passed: false,
            error: format!("Expected success={}, got success={}", test.should_succeed, success),
        };
    }
    
    if stdout != test.expected_stdout {
        return GoldenResult {
            passed: false,
            error: format!("STDOUT mismatch.\nExpected: {:?}\nActual: {:?}", 
                test.expected_stdout, stdout.as_ref()),
        };
    }
    
    if stderr != test.expected_stderr {
        return GoldenResult {
            passed: false,
            error: format!("STDERR mismatch.\nExpected: {:?}\nActual: {:?}", 
                test.expected_stderr, stderr.as_ref()),
        };
    }
    
    GoldenResult {
        passed: true,
        error: String::new(),
    }
}