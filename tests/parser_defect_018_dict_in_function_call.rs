//! PARSER-DEFECT-018: Dictionary literals inside function calls fail to parse
//!
//! Root Cause: Parser fails with "Expected `RightBrace`, found Identifier" when dict literal passed as argument
//! Impact: CRITICAL - Blocks all examples using object literals in method calls
//! Pattern: `array.append({ key: value })` fails to parse

#![allow(missing_docs)]

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// RED: Test dictionary literal as function argument
#[test]
fn test_dict_literal_in_function_call() {
    let temp_dir = TempDir::new().unwrap();
    let script = temp_dir.path().join("test.ruchy");

    let code = r#"
fun main() {
    let items = [];
    items.append({ name: "test", value: 42 });
    println!("Done");
}

main()
"#;
    fs::write(&script, code).unwrap();

    // RED: Currently fails with "Expected RightBrace, found Identifier(println)"
    // GREEN: Should parse and transpile successfully
    ruchy_cmd().arg("transpile").arg(&script).assert().success();
}

/// RED: Test nested dict literal (real-world pattern from `21_concurrency.ruchy`)
#[test]
fn test_transactions_append_pattern() {
    let temp_dir = TempDir::new().unwrap();
    let script = temp_dir.path().join("test.ruchy");

    let code = r#"
fun main() {
    let transactions = [];
    let amount = 100.0;
    
    transactions.append({
        type: "deposit",
        amount: amount,
        timestamp: 12345
    });
    
    println!("Transaction added");
}

main()
"#;
    fs::write(&script, code).unwrap();

    ruchy_cmd().arg("transpile").arg(&script).assert().success();
}

/// RED: Test multiple dict literals in sequence
#[test]
fn test_multiple_dict_literals_in_calls() {
    let temp_dir = TempDir::new().unwrap();
    let script = temp_dir.path().join("test.ruchy");

    let code = r#"
fun main() {
    let list = [];
    list.append({ id: 1, name: "first" });
    list.append({ id: 2, name: "second" });
    println!("Done");
}

main()
"#;
    fs::write(&script, code).unwrap();

    ruchy_cmd().arg("transpile").arg(&script).assert().success();
}

/// RED: Test dict literal with computed values
#[test]
fn test_dict_literal_with_expressions() {
    let temp_dir = TempDir::new().unwrap();
    let script = temp_dir.path().join("test.ruchy");

    let code = r#"
fun process(data) {
    println!("Processing");
}

fun main() {
    let x = 5;
    let y = 10;
    process({ sum: x + y, product: x * y });
    println!("Done");
}

main()
"#;
    fs::write(&script, code).unwrap();

    ruchy_cmd().arg("transpile").arg(&script).assert().success();
}
