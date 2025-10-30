#![allow(missing_docs)]
// Matrix Test 02: CSV Processing Workflow (Native Platform)
//
// Companion to: tests/e2e/matrix/02-csv-workflow.spec.ts (WASM - DEFERRED)
//
// Goal: Verify data processing workflow (simulating CSV operations)
// This test uses rexpect to interact with the native `ruchy` REPL
//
// NOTE: Full CSV/HTTP modules not yet implemented, so we simulate
// CSV operations using arrays and basic data manipulation

use rexpect::session::spawn_command;
use std::process::Command;

/// Helper to create a ruchy REPL session
fn spawn_ruchy_repl() -> rexpect::session::PtySession {
    let cmd = Command::new("ruchy");
    spawn_command(cmd, Some(10000)).expect("Failed to spawn ruchy REPL")
}

#[test]
fn test_matrix_native_02_01_array_creation() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Create array simulating CSV rows (id, age, salary)
    repl.send_line("let data = [[1, 25, 50000], [2, 35, 75000], [3, 45, 100000]]")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Verify array creation (use .len() method, not .length property)
    repl.send_line("data.len()")
        .expect("Failed to send command");

    // Expect 3 rows
    repl.exp_string("3").expect("Array should have 3 rows");
}

#[test]
fn test_matrix_native_02_02_data_filtering() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Create data
    repl.send_line("let data = [[1, 25, 50000], [2, 35, 75000], [3, 45, 100000]]")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Filter: age > 30 (index 1)
    repl.send_line("let filtered = data.filter(|row| row[1] > 30)")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Verify filtered length
    repl.send_line("filtered.len()")
        .expect("Failed to send command");

    // Expect 2 rows (age 35 and 45)
    repl.exp_string("2").expect("Filtered data should have 2 rows");
}

#[test]
fn test_matrix_native_02_03_data_mapping() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Create data
    repl.send_line("let data = [[1, 25, 50000], [2, 35, 75000], [3, 45, 100000]]")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Map: extract salaries (index 2)
    repl.send_line("let salaries = data.map(|row| row[2])")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Verify first salary
    repl.send_line("salaries[0]")
        .expect("Failed to send command");

    repl.exp_string("50000").expect("First salary should be 50000");
}

#[test]
fn test_matrix_native_02_04_data_aggregation() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Create simple numeric array
    repl.send_line("let numbers = [10, 20, 30, 40, 50]")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Sum using reduce
    repl.send_line("let sum = numbers.reduce(|acc, x| acc + x, 0)")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Verify sum
    repl.send_line("sum")
        .expect("Failed to send command");

    repl.exp_string("150").expect("Sum should be 150");
}

#[test]
fn test_matrix_native_02_05_workflow_filter_map_reduce() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Complete workflow in one chained expression
    // Create data → Filter → Map → Reduce (all in one)
    repl.send_line("[[1, 25, 50000], [2, 35, 75000], [3, 45, 100000], [4, 32, 80000]].filter(|row| row[1] > 30).map(|row| row[2]).reduce(|acc, x| acc + x, 0)")
        .expect("Failed to send command");

    // Total should be: 75000 + 100000 + 80000 = 255000
    repl.exp_string("255000").expect("Sum of salaries should be 255000");
}

#[test]
fn test_matrix_native_02_06_nested_data_structures() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Simulate CSV with headers via struct-like objects
    repl.send_line(r#"let row1 = {"id": 1, "name": "Alice", "age": 30}"#)
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Access field
    repl.send_line("row1.age")
        .expect("Failed to send command");

    repl.exp_string("30").expect("Age should be 30");
}

#[test]
fn test_matrix_native_02_07_chained_operations() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Create array
    repl.send_line("let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Chain: filter even numbers, map to squares, sum
    repl.send_line("let result = data.filter(|x| x % 2 == 0).map(|x| x * x).reduce(|acc, x| acc + x, 0)")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    repl.send_line("result")
        .expect("Failed to send command");

    // Even numbers: 2,4,6,8,10 → squares: 4,16,36,64,100 → sum: 220
    repl.exp_string("220").expect("Result should be 220");
}

#[test]
fn test_matrix_native_02_08_real_world_data_pipeline() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Simulate sales data: [product_id, quantity, price]
    repl.send_line("let sales = [[101, 5, 10], [102, 3, 20], [103, 8, 15], [104, 2, 25]]")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Calculate total revenue: quantity * price for each row
    repl.send_line("let revenues = sales.map(|row| row[1] * row[2])")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    // Sum total revenue
    repl.send_line("let total_revenue = revenues.reduce(|acc, x| acc + x, 0)")
        .expect("Failed to send command");

    repl.exp_string("ruchy>").expect("Failed to find REPL prompt");

    repl.send_line("total_revenue")
        .expect("Failed to send command");

    // (5*10) + (3*20) + (8*15) + (2*25) = 50 + 60 + 120 + 50 = 280
    repl.exp_string("280").expect("Total revenue should be 280");
}
