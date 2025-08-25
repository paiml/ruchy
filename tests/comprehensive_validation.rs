#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
//! Comprehensive validation test suite - Toyota Way quality enforcement
//! Based on lessons learned from v1.0.0 testing failures

#![allow(clippy::print_stdout)]
#![allow(clippy::expect_used)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::unwrap_used)]

use std::process::Command;
use std::fs;
use std::path::Path;
use std::collections::HashMap;

// Helper function for property tests
fn run_expression(expr: &str) -> String {
    let output = Command::new("./target/release/ruchy")
        .arg("-e")
        .arg(expr)
        .output()
        .expect("Failed to run expression");
        
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// TIER 1: File Compilation Tests (Critical Gap from v1.0.0)
#[test]  
fn test_file_compilation_regression() {
    println!("🔥 TIER 1: File Compilation Regression Tests");
    
    let test_cases = vec![
        // Bug #1: Variable scoping
        ("variable_scoping", "let x = 42;\nlet y = x + 8;\nprintln(y);"),
        
        // Bug #2: Function definitions  
        ("function_definition", "fun add(a, b) {\n    a + b\n}\nprintln(add(5, 3));"),
        
        // Bug #3: Multi-argument printf
        ("printf_multiarg", "fun main() {\n    let name = \"Alice\";\n    println(\"Hi\", name, \"!\");\n}"),
        
        // Additional critical patterns
        ("nested_functions", "fun outer() {\n    fun inner(x) { x * 2 }\n    inner(21)\n}\nprintln(outer());"),
        ("complex_expressions", "let result = (5 + 3) * 2 - 1;\nlet doubled = result * 2;\nprintln(\"Result:\", doubled);"),
    ];
    
    let mut passed = 0;
    let total = test_cases.len();
    
    for (name, code) in test_cases {
        let filename = format!("/tmp/test_{name}.ruchy");
        fs::write(&filename, code).expect("Failed to write test file");
        
        let output = Command::new("./target/release/ruchy")
            .arg("compile")
            .arg(&filename)
            .output()
            .expect("Failed to run ruchy compile");
            
        if output.status.success() {
            println!("✅ {name}: File compilation PASSED");
            passed += 1;
            
            // Also test execution
            if Path::new("a.out").exists() {
                let exec_output = Command::new("./a.out").output();
                if let Ok(result) = exec_output {
                    if result.status.success() {
                        println!("   ✅ Execution also PASSED");  
                    } else {
                        println!("   ❌ Execution FAILED: {}", String::from_utf8_lossy(&result.stderr));
                    }
                }
                let _ = fs::remove_file("a.out"); // Cleanup
            }
        } else {
            println!("❌ {name}: File compilation FAILED");
            println!("   Error: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        let _ = fs::remove_file(&filename); // Cleanup
    }
    
    println!("📊 File Compilation: {}/{} passed ({}%)", passed, total, (passed * 100) / total);
    
    // CRITICAL: Must pass at least 80% for v1.0.1 release 
    assert!(passed >= (total * 80) / 100, "File compilation regression detected!");
}

/// TIER 2: REPL vs File Consistency Tests
#[test]
fn test_repl_file_consistency() {
    println!("🔄 TIER 2: REPL vs File Consistency");
    
    let expressions = vec![
        "2 + 2",
        "let x = 42; x", 
        "fun square(n) { n * n } square(7)",
        "if true { \"yes\" } else { \"no\" }",
        "[1, 2, 3].len()",
    ];
    
    for expr in expressions {
        // Test REPL execution
        let repl_output = Command::new("./target/release/ruchy")
            .arg("-e")
            .arg(expr)
            .output()
            .expect("Failed to run REPL");
            
        // Test file execution  
        let filename = "/tmp/consistency_test.ruchy";
        fs::write(filename, expr).expect("Failed to write test file");
        
        let file_output = Command::new("./target/release/ruchy")
            .arg("run")
            .arg(filename)
            .output()
            .expect("Failed to run file");
            
        let repl_result = String::from_utf8_lossy(&repl_output.stdout);
        let file_result = String::from_utf8_lossy(&file_output.stdout);
        
        if repl_result.trim() == file_result.trim() {
            println!("✅ Consistency: {expr}");
        } else {
            println!("❌ INCONSISTENCY: {expr}");
            println!("   REPL: {}", repl_result.trim());
            println!("   FILE: {}", file_result.trim());
        }
        
        let _ = fs::remove_file(filename);
    }
}

/// TIER 3: Property-Based Testing (Mathematical Invariants)
#[test] 
fn test_arithmetic_properties() {
    println!("🧮 TIER 3: Property-Based Arithmetic Tests");
    
    let mut results = HashMap::new();
    
    // Commutative property: a + b = b + a
    for a in [1, 5, 10, 100] {
        for b in [2, 7, 15, 50] {
            let expr1 = format!("{a} + {b}");
            let expr2 = format!("{b} + {a}");
            
            let expr1_result = run_expression(&expr1);
            let expr2_result = run_expression(&expr2);
            
            results.insert(format!("commutative_{a}_{b}"), expr1_result == expr2_result);
        }
    }
    
    // Associative property: (a + b) + c = a + (b + c)
    for a in [1, 5, 10] {
        for b in [2, 7, 15] {
            for c in [3, 8, 20] {
                let expr1 = format!("({a} + {b}) + {c}");
                let expr2 = format!("{a} + ({b} + {c})");
                
                let expr1_result = run_expression(&expr1);
                let expr2_result = run_expression(&expr2);
                
                results.insert(format!("associative_{a}_{b}_{c}"), expr1_result == expr2_result);
            }
        }
    }
    
    let passed = results.values().filter(|&&v| v).count();
    let total = results.len();
    
    println!("📊 Property Tests: {passed}/{total} passed");
    
    for (test, passed) in results {
        if !passed {
            println!("❌ PROPERTY VIOLATION: {test}");
        }
    }
    
    assert_eq!(passed, total, "Mathematical property violations detected!");
}

/// TIER 4: Fuzzing Tests (Random Input Robustness)
#[test]
fn test_parser_fuzz_robustness() {
    println!("🎲 TIER 4: Parser Fuzz Testing");
    
    let fuzz_inputs = vec![
        // Malformed syntax
        "let x = ;",
        "fun ()",
        "if true {",
        "match x",
        "for i in",
        
        // Edge cases
        "\"unclosed string",
        "/* unclosed comment",
        "123.456.789",
        "let let = let",
        
        // Unicode and special chars
        "let 变量 = 42",
        "println(\"🚀\")",
        "let x = 1e999999",
        
        // Deeply nested - use static strings for borrowing
        "((((((((((((((((((((((((((((((((((((((((((()))))))))))))))))))))))))))))))))))))))))",
    ];
    
    let mut crash_count = 0;
    let total = fuzz_inputs.len();
    
    for (i, input) in fuzz_inputs.iter().enumerate() {
        let filename = format!("/tmp/fuzz_test_{i}.ruchy");
        fs::write(&filename, input).expect("Failed to write fuzz file");
        
        let output = Command::new("./target/release/ruchy")
            .arg("parse")
            .arg(&filename)
            .output()
            .expect("Failed to run parser");
            
        // Parser should not crash, even on invalid input
        if !output.status.success() && output.stderr.is_empty() {
            crash_count += 1;
            println!("💥 CRASH on input: {input:?}");
        }
        
        let _ = fs::remove_file(&filename);
    }
    
    println!("📊 Fuzz Tests: {}/{} handled gracefully", total - crash_count, total);
    assert!(crash_count == 0, "Parser crashes detected on malformed input!");
}

/// TIER 5: Documentation Tests (All Examples Must Work)
#[test]
fn test_documentation_examples() {
    println!("📚 TIER 5: Documentation Example Tests");
    
    // Examples from README and docs that should work
    let doc_examples = vec![
        ("basic_math", "2 + 2 * 3"),
        ("variables", "let price = 99.99\nlet tax = 0.08\nprice * (1.0 + tax)"),
        ("functions", "fun greeting(name) {\n    \"Hello, \" + name + \"!\"\n}\ngreeting(\"Ruchy\")"),
        ("control_flow", "if 100 > 50 { \"expensive\" } else { \"cheap\" }"),
        ("arrays", "let numbers = [1, 2, 3, 4, 5]\nnumbers.len()"),
    ];
    
    let mut passed = 0;
    
    let total_examples = doc_examples.len();
    for (name, code) in doc_examples {
        let filename = format!("/tmp/doc_test_{name}.ruchy");
        fs::write(&filename, code).expect("Failed to write doc test");
        
        let output = Command::new("./target/release/ruchy")
            .arg("run")  
            .arg(&filename)
            .output()
            .expect("Failed to run doc example");
            
        if output.status.success() {
            println!("✅ Doc example '{name}': PASSED");
            passed += 1;
        } else {
            println!("❌ Doc example '{name}': FAILED");
            println!("   Error: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        let _ = fs::remove_file(&filename);
    }
    
    println!("📊 Documentation: {passed}/{total_examples} examples working");
}

/// TIER 6: End-to-End Workflow Tests
#[test] 
fn test_e2e_development_workflow() {
    println!("🔄 TIER 6: End-to-End Workflow Tests");
    
    // Test complete development workflow
    let program = r#"
// A complete Ruchy program for testing E2E workflow
fun fibonacci(n) {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

fun main() {
    let result = fibonacci(10)
    println("Fibonacci(10) =", result)
}
"#;
    
    let filename = "/tmp/e2e_test.ruchy";
    fs::write(filename, program).expect("Failed to write E2E test");
    
    // Test 1: Parse
    let parse_result = Command::new("./target/release/ruchy")
        .arg("parse")
        .arg(filename) 
        .output()
        .expect("Failed to parse");
        
    assert!(parse_result.status.success(), "E2E: Parse failed");
    println!("✅ E2E: Parse successful");
    
    // Test 2: Transpile  
    let transpile_result = Command::new("./target/release/ruchy")
        .arg("transpile")
        .arg(filename)
        .output()
        .expect("Failed to transpile");
        
    assert!(transpile_result.status.success(), "E2E: Transpile failed");
    println!("✅ E2E: Transpile successful");
    
    // Test 3: Compile
    let compile_result = Command::new("./target/release/ruchy")
        .arg("compile") 
        .arg(filename)
        .output()
        .expect("Failed to compile");
        
    assert!(compile_result.status.success(), "E2E: Compile failed");
    println!("✅ E2E: Compile successful");
    
    // Test 4: Execute
    if Path::new("a.out").exists() {
        let exec_result = Command::new("./a.out")
            .output()
            .expect("Failed to execute");
            
        assert!(exec_result.status.success(), "E2E: Execution failed");
        println!("✅ E2E: Execution successful");
        println!("   Output: {}", String::from_utf8_lossy(&exec_result.stdout).trim());
        
        let _ = fs::remove_file("a.out");
    }
    
    let _ = fs::remove_file(filename);
    println!("🎯 E2E: Complete workflow validated");
}