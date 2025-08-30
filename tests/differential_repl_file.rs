// Differential testing: REPL vs File execution must produce same results
// This would have caught our obj.items() transpilation bug immediately

use std::process::Command;
use std::fs;

struct DifferentialTest {
    name: &'static str,
    code: &'static str,
}

const DIFFERENTIAL_TESTS: &[DifferentialTest] = &[
    DifferentialTest {
        name: "while_loop_output",
        code: "let i = 0; while i < 3 { println(i); i = i + 1 }",
    },
    DifferentialTest {
        name: "object_items_iteration", 
        code: r#"let obj = {"key": 42}; for k, v in obj.items() { println(k) }"#,
    },
    DifferentialTest {
        name: "arithmetic_expression",
        code: "println(2 + 3 * 4 - 1)",
    },
    DifferentialTest {
        name: "string_interpolation",
        code: r#"let x = 42; println(f"Answer: {x}")"#,
    },
    DifferentialTest {
        name: "array_operations",
        code: "let arr = [1, 2, 3]; println(arr.map(|x| x * 2))",
    },
    DifferentialTest {
        name: "function_calls",
        code: "fn double(x) { x * 2 }; println(double(21))",
    },
    DifferentialTest {
        name: "pattern_matching",
        code: r#"let x = 2; println(match x { 1 => "one", _ => "other" })"#,
    },
];

#[test]
fn test_repl_file_consistency() {
    let mut failures = Vec::new();
    
    for test in DIFFERENTIAL_TESTS {
        match run_differential_test(test) {
            Ok(()) => continue,
            Err(error) => failures.push((test.name, error)),
        }
    }
    
    if !failures.is_empty() {
        let mut error_msg = String::from("REPL vs File execution differences detected:\n");
        for (name, error) in failures {
            error_msg.push_str(&format!("  - {name}: {error}\n"));
        }
        panic!("{}", error_msg);
    }
}

fn run_differential_test(test: &DifferentialTest) -> Result<(), String> {
    // Test via REPL
    let repl_output = Command::new("bash")
        .arg("-c")
        .arg(format!("echo '{}' | timeout 10 ./target/release/ruchy repl", test.code))
        .output()
        .map_err(|e| format!("Failed to run REPL: {e}"))?;
    
    // Test via file
    let test_file = format!("/tmp/diff_test_{}.ruchy", test.name);
    fs::write(&test_file, test.code)
        .map_err(|e| format!("Failed to write test file: {e}"))?;
    
    let file_output = Command::new("timeout")
        .arg("10")
        .arg("./target/release/ruchy")
        .arg(&test_file)
        .output()
        .map_err(|e| format!("Failed to run file: {e}"))?;
    
    // Compare results
    let repl_stdout = extract_repl_output(&String::from_utf8_lossy(&repl_output.stdout));
    let file_stdout = String::from_utf8_lossy(&file_output.stdout);
    
    let repl_success = repl_output.status.success();
    let file_success = file_output.status.success();
    
    // Check success status
    if repl_success != file_success {
        return Err(format!(
            "Success status differs: REPL={}, File={}\nREPL stderr: {}\nFile stderr: {}",
            repl_success, file_success,
            String::from_utf8_lossy(&repl_output.stderr),
            String::from_utf8_lossy(&file_output.stderr)
        ));
    }
    
    // Check output content (normalize whitespace)
    let repl_normalized = normalize_output(&repl_stdout);
    let file_normalized = normalize_output(&file_stdout);
    
    if repl_normalized != file_normalized {
        return Err(format!(
            "Output differs:\nREPL: {repl_normalized:?}\nFile: {file_normalized:?}"
        ));
    }
    
    Ok(())
}

fn extract_repl_output(repl_full_output: &str) -> String {
    // Remove REPL prompts and system messages, and trailing "()" unit values
    repl_full_output
        .lines()
        .filter(|line| {
            !line.contains("Welcome to Ruchy REPL") &&
            !line.contains("Type :help") &&
            !line.contains("Goodbye!") &&
            !line.is_empty() &&
            line.trim() != "()"  // Filter out unit return values
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize_output(output: &str) -> String {
    output.trim()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_error_consistency() {
    // Errors should produce similar error messages, but exit codes differ by design
    // REPL continues running (exit 0), files terminate (exit 1)
    let error_cases = vec![
        "let x = y", // undefined y
        "fn f() { return unknown_var }; f()", // undefined in function when called
    ];
    
    for code in error_cases {
        let repl_result = Command::new("bash")
            .arg("-c")
            .arg(format!("echo '{code}' | timeout 5 ./target/release/ruchy repl 2>&1"))
            .output()
            .unwrap();
            
        let test_file = "/tmp/error_test.ruchy";
        fs::write(test_file, code).unwrap();
        let file_result = Command::new("timeout")
            .arg("5")
            .arg("./target/release/ruchy")
            .arg(test_file)
            .output()
            .unwrap();
        
        // Both should show error messages
        let repl_output = String::from_utf8_lossy(&repl_result.stdout);
        let file_output = String::from_utf8_lossy(&file_result.stderr);
        
        assert!(repl_output.contains("Error:") || repl_output.contains("error:"),
            "REPL should show error for: {code}");
        assert!(file_output.contains("Error:") || file_output.contains("error:"),
            "File should show error for: {code}");
    }
}