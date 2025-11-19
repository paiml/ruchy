#![allow(missing_docs)]
// Matrix Test 01: Simple Arithmetic Operations (Native Platform)
//
// Companion to: tests/e2e/matrix/01-simple-arithmetic.spec.ts (WASM)
//
// Goal: Verify identical behavior between WASM and native platforms
// This test uses rexpect to interact with the native `ruchy` REPL

use rexpect::session::spawn_command;
use std::process::Command;

/// Helper to create a ruchy REPL session
fn spawn_ruchy_repl() -> rexpect::session::PtySession {
    let cmd = Command::new("ruchy");
    spawn_command(cmd, Some(10000)).expect("Failed to spawn ruchy REPL")
}

#[test]
fn test_matrix_native_01_addition() {
    let mut repl = spawn_ruchy_repl();

    // Wait for REPL prompt
    repl.exp_string("ruchy>")
        .expect("Failed to find REPL prompt");

    // Execute: 10 + 20
    repl.send_line("10 + 20").expect("Failed to send command");

    // Expect result: 30
    repl.exp_string("30").expect("Result should be 30");
}

#[test]
fn test_matrix_native_02_subtraction() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>")
        .expect("Failed to find REPL prompt");

    // Execute: 100 - 42
    repl.send_line("100 - 42").expect("Failed to send command");

    // Expect result: 58
    repl.exp_string("58").expect("Result should be 58");
}

#[test]
fn test_matrix_native_03_multiplication() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>")
        .expect("Failed to find REPL prompt");

    // Execute: 6 * 7
    repl.send_line("6 * 7").expect("Failed to send command");

    // Expect result: 42
    repl.exp_string("42").expect("Result should be 42");
}

#[test]
fn test_matrix_native_04_division() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>")
        .expect("Failed to find REPL prompt");

    // Execute: 100 / 4
    repl.send_line("100 / 4").expect("Failed to send command");

    // Expect result: 25
    repl.exp_string("25").expect("Result should be 25");
}

#[test]
fn test_matrix_native_05_operator_precedence() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>")
        .expect("Failed to find REPL prompt");

    // Execute: 2 + 3 * 4
    repl.send_line("2 + 3 * 4").expect("Failed to send command");

    // Should respect precedence: 2 + (3 * 4) = 14, not (2 + 3) * 4 = 20
    repl.exp_string("14")
        .expect("Result should be 14 (precedence check)");
}

#[test]
fn test_matrix_native_06_parentheses() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>")
        .expect("Failed to find REPL prompt");

    // Execute: (2 + 3) * 4
    repl.send_line("(2 + 3) * 4")
        .expect("Failed to send command");

    // Should evaluate parentheses first: (2 + 3) * 4 = 20
    repl.exp_string("20").expect("Result should be 20");
}

#[test]
fn test_matrix_native_07_variables() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>")
        .expect("Failed to find REPL prompt");

    // Declare variable
    repl.send_line("let x = 10")
        .expect("Failed to send command");
    repl.exp_string("ruchy>")
        .expect("Failed to find REPL prompt");

    // Use variable
    repl.send_line("x * 2").expect("Failed to send command");

    // Expect result: 20
    repl.exp_string("20").expect("Result should be 20");
}

#[test]
fn test_matrix_native_08_multi_step_computation() {
    let mut repl = spawn_ruchy_repl();

    repl.exp_string("ruchy>")
        .expect("Failed to find REPL prompt");

    // Step 1: Define variables
    repl.send_line("let a = 5").expect("Failed to send command");
    repl.exp_string("ruchy>")
        .expect("Failed to find REPL prompt");

    repl.send_line("let b = 10")
        .expect("Failed to send command");
    repl.exp_string("ruchy>")
        .expect("Failed to find REPL prompt");

    repl.send_line("let c = 15")
        .expect("Failed to send command");
    repl.exp_string("ruchy>")
        .expect("Failed to find REPL prompt");

    // Step 2: Compute result
    repl.send_line("a + b + c").expect("Failed to send command");

    // Should be 5 + 10 + 15 = 30
    repl.exp_string("30").expect("Result should be 30");
}
