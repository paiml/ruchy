// RED TEST: Transpiler E0382 - Nested loop ownership bug
// Blocking Reaper v1.0.0 crates.io publication (99.1% complete)
//
// Root cause: When a value is used in an inner loop body multiple times,
// the transpiler generates code that moves the value on first use, causing
// E0382 on subsequent iterations.
//
// Pattern from Reaper src/main.ruchy:299-308:
// ```ruchy
// while i < procs.len() {
//     let proc = procs[i].clone();
//     while j < rules.len() {
//         let rule = rules[j].clone();
//         if rule.enabled && rule_matches_process(rule, proc) {  // proc moved here
//             break;
//         }
//         j = j + 1;
//     }
//     i = i + 1;
// }
// ```
//
// Expected transpiled Rust:
// ```rust
// if rule.enabled && rule_matches_process(rule, proc.clone()) {
// ```
//
// This test will FAIL until transpiler auto-clones values used in inner loops

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// RED TEST: Nested loop with value used in inner loop body
#[test]
fn test_defect_018_red_nested_loop_value_moved_in_inner_loop() {
    let temp_dir = TempDir::new().unwrap();
    let ruchy_file = temp_dir.path().join("nested_loop_ownership.ruchy");

    // Minimal reproduction of Reaper pattern
    let code = r#"
struct Process {
    name: String,
    pid: i32
}

struct Rule {
    enabled: bool,
    pattern: String
}

fun rule_matches_process(rule: Rule, proc: Process) -> bool {
    proc.name == rule.pattern
}

fun scan_processes(procs: Vec<Process>, rules: Vec<Rule>) {
    let mut i = 0;
    while i < procs.len() {
        let proc = procs[i];  // EXACT PATTERN: No .clone()
        let mut j = 0;
        while j < rules.len() {
            let rule = rules[j];  // EXACT PATTERN: No .clone()
            // BUG: proc is moved here on first iteration, can't be used again
            if rule.enabled && rule_matches_process(rule, proc) {
                println("Match found!");
                break;
            }
            j = j + 1;
        }
        i = i + 1;
    }
}

fun main() {
    let procs = vec![
        Process { name: "firefox".to_string(), pid: 1234 },
        Process { name: "chrome".to_string(), pid: 5678 }
    ];
    let rules = vec![
        Rule { enabled: true, pattern: "firefox".to_string() },
        Rule { enabled: false, pattern: "chrome".to_string() }
    ];
    scan_processes(procs, rules);
}
"#;

    fs::write(&ruchy_file, code).unwrap();

    // Try to transpile
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile").arg(&ruchy_file);

    let output = cmd.output().unwrap();
    let rust_code = String::from_utf8_lossy(&output.stdout);

    println!("Generated Rust code:\n{rust_code}");

    // Now try to compile the transpiled Rust
    let transpiled_file = ruchy_file.with_extension("rs");
    fs::write(&transpiled_file, rust_code.as_ref()).unwrap();

    // Attempt cargo check
    let output_bin = temp_dir.path().join("test_bin");
    let check_output = std::process::Command::new("rustc")
        .arg("--crate-type")
        .arg("bin")
        .arg("--edition")
        .arg("2021")
        .arg(&transpiled_file)
        .arg("-o")
        .arg(&output_bin)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&check_output.stderr);

    println!("Rustc output:\n{stderr}");

    // RED: This should fail with E0382 until fixed
    if !check_output.status.success()
        && stderr.contains("E0382") && stderr.contains("use of moved value") {
            println!("✅ RED TEST: E0382 ownership error confirmed");
            println!("   Transpiler generates buggy code for nested loop pattern");
            return; // RED test passes (confirms bug exists)
        }

    panic!("RED TEST FAILED: Expected E0382 error but compilation succeeded or had different error");
}

/// RED TEST 2: Simpler nested loop with function call
#[test]
fn test_defect_018_red_simple_nested_loop_function_call() {
    let temp_dir = TempDir::new().unwrap();
    let ruchy_file = temp_dir.path().join("simple_nested.ruchy");

    let code = r#"
struct Item {
    value: String
}

fun process_item(item: Item) -> bool {
    item.value.len() > 0
}

fun main() {
    let items = vec![
        Item { value: "hello".to_string() },
        Item { value: "world".to_string() }
    ];

    let mut i = 0;
    while i < items.len() {
        let item = items[i];  // EXACT PATTERN: No .clone()
        let mut j = 0;
        while j < 2 {
            // BUG: item moved here, can't be used in next iteration
            if process_item(item) {
                println("Item processed");
            }
            j = j + 1;
        }
        i = i + 1;
    }
}
"#;

    fs::write(&ruchy_file, code).unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile").arg(&ruchy_file);

    let output = cmd.output().unwrap();
    let rust_code = String::from_utf8_lossy(&output.stdout);

    let transpiled_file = ruchy_file.with_extension("rs");
    fs::write(&transpiled_file, rust_code.as_ref()).unwrap();

    let output_bin = temp_dir.path().join("test_bin2");
    let check_output = std::process::Command::new("rustc")
        .arg("--crate-type")
        .arg("bin")
        .arg("--edition")
        .arg("2021")
        .arg(&transpiled_file)
        .arg("-o")
        .arg(&output_bin)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&check_output.stderr);

    if !check_output.status.success()
        && (stderr.contains("E0382") || stderr.contains("use of moved value")) {
            println!("✅ RED TEST 2: Nested loop ownership error confirmed");
            return;
        }

    panic!("RED TEST 2 FAILED: Expected E0382 but got different result");
}

/// BASELINE: Single loop (no nesting) should work
#[test]
fn test_defect_018_baseline_single_loop_works() {
    let temp_dir = TempDir::new().unwrap();
    let ruchy_file = temp_dir.path().join("single_loop.ruchy");

    let code = r#"
struct Item {
    value: String
}

fun process_item(item: Item) -> bool {
    item.value.len() > 0
}

fun main() {
    let items = vec![
        Item { value: "hello".to_string() }
    ];

    let mut i = 0;
    while i < items.len() {
        let item = items[i].clone();
        // Single use - no problem
        if process_item(item) {
            println("Item processed");
        }
        i = i + 1;
    }
}
"#;

    fs::write(&ruchy_file, code).unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile").arg(&ruchy_file).assert().success();
}
