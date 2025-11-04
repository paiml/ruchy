/// RED Phase Test: Detect and fix deadlock bug in global variable assignments
///
/// ROOT CAUSE: Double-locking when assigning to globals
/// Example: counter = counter + 1 transpiles to:
///   *counter.lock().unwrap() = *counter.lock().unwrap() + 1;
///   ^^^^^^^^^^^^^^^^^^^^^       ^^^^^^^^^^^^^^^^^^^^^^^^
///   Lock #1                     Lock #2 → DEADLOCK!
///
/// EXPECTED: Should lock once and operate on guard
///   {
///       let mut guard = counter.lock().unwrap();
///       *guard = *guard + 1;
///   }

use assert_cmd::Command;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::NamedTempFile;

/// Test 1: Simple assignment doesn't deadlock
/// Tests: counter = counter + 1
#[test]
fn test_no_deadlock_simple_assignment() {
    let code = r#"
let mut counter = 0

fn increment() {
    counter = counter + 1
}

increment()
println!("{}", counter)
"#;

    // Write to temp file
    let temp = NamedTempFile::new().unwrap();
    let ruchy_path = temp.path().with_extension("ruchy");
    fs::write(&ruchy_path, code).unwrap();

    // Transpile
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    let output = cmd
        .arg("transpile")
        .arg(&ruchy_path)
        .output()
        .expect("Failed to transpile");

    assert!(output.status.success(), "Transpile failed");
    let transpiled = String::from_utf8(output.stdout).unwrap();

    // Write transpiled code
    let rs_path = temp.path().with_extension("rs");
    fs::write(&rs_path, transpiled).unwrap();

    // Compile
    let compile = StdCommand::new("rustc")
        .arg(&rs_path)
        .arg("--crate-name")
        .arg("test_deadlock")
        .arg("-o")
        .arg(temp.path().with_extension("exe"))
        .output()
        .unwrap();

    assert!(
        compile.status.success(),
        "Compilation failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    // Run with timeout (2 seconds)
    // If it hangs, this will timeout
    let exe_path = temp.path().with_extension("exe");
    let run = StdCommand::new("timeout")
        .arg("2")
        .arg(&exe_path)
        .output()
        .expect("Failed to run");

    // Check it didn't timeout (exit code 124 = timeout)
    assert_ne!(
        run.status.code(),
        Some(124),
        "Program DEADLOCKED (timeout)! Output:\n{}\nStderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    assert!(
        run.status.success(),
        "Program failed:\n{}\nStderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    // Verify output
    let output_str = String::from_utf8_lossy(&run.stdout);
    assert!(
        output_str.contains("1"),
        "Expected output '1', got: {}",
        output_str
    );

    // Cleanup
    let _ = fs::remove_file(&ruchy_path);
    let _ = fs::remove_file(&rs_path);
    let _ = fs::remove_file(&exe_path);
}

/// Test 2: Compound assignment doesn't deadlock
/// Tests: counter += 1
#[test]
fn test_no_deadlock_compound_assignment() {
    let code = r#"
let mut total = 0

fn add_value(x: i32) {
    total += x
}

add_value(5)
add_value(10)
println!("{}", total)
"#;

    let temp = NamedTempFile::new().unwrap();
    let ruchy_path = temp.path().with_extension("ruchy");
    fs::write(&ruchy_path, code).unwrap();

    // Transpile
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    let output = cmd
        .arg("transpile")
        .arg(&ruchy_path)
        .output()
        .expect("Failed to transpile");

    assert!(output.status.success(), "Transpile failed");
    let transpiled = String::from_utf8(output.stdout).unwrap();

    let rs_path = temp.path().with_extension("rs");
    fs::write(&rs_path, transpiled).unwrap();

    // Compile
    let compile = StdCommand::new("rustc")
        .arg(&rs_path)
        .arg("--crate-name")
        .arg("test_compound")
        .arg("-o")
        .arg(temp.path().with_extension("exe"))
        .output()
        .unwrap();

    assert!(
        compile.status.success(),
        "Compilation failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    // Run with timeout
    let exe_path = temp.path().with_extension("exe");
    let run = StdCommand::new("timeout")
        .arg("2")
        .arg(&exe_path)
        .output()
        .expect("Failed to run");

    assert_ne!(
        run.status.code(),
        Some(124),
        "Program DEADLOCKED (timeout)!"
    );

    assert!(run.status.success(), "Program failed");

    let output_str = String::from_utf8_lossy(&run.stdout);
    assert!(
        output_str.contains("15"),
        "Expected '15', got: {}",
        output_str
    );

    // Cleanup
    let _ = fs::remove_file(&ruchy_path);
    let _ = fs::remove_file(&rs_path);
    let _ = fs::remove_file(&exe_path);
}

/// Test 3: Multiple references don't deadlock
/// Tests: result = global1 + global2
#[test]
fn test_no_deadlock_multiple_globals() {
    let code = r#"
let mut x = 10
let mut y = 20

fn compute() {
    x = x + y
}

compute()
println!("{}", x)
"#;

    let temp = NamedTempFile::new().unwrap();
    let ruchy_path = temp.path().with_extension("ruchy");
    fs::write(&ruchy_path, code).unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    let output = cmd
        .arg("transpile")
        .arg(&ruchy_path)
        .output()
        .expect("Failed to transpile");

    assert!(output.status.success(), "Transpile failed");
    let transpiled = String::from_utf8(output.stdout).unwrap();

    let rs_path = temp.path().with_extension("rs");
    fs::write(&rs_path, transpiled).unwrap();

    let compile = StdCommand::new("rustc")
        .arg(&rs_path)
        .arg("--crate-name")
        .arg("test_multiple")
        .arg("-o")
        .arg(temp.path().with_extension("exe"))
        .output()
        .unwrap();

    assert!(
        compile.status.success(),
        "Compilation failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let exe_path = temp.path().with_extension("exe");
    let run = StdCommand::new("timeout")
        .arg("2")
        .arg(&exe_path)
        .output()
        .expect("Failed to run");

    assert_ne!(
        run.status.code(),
        Some(124),
        "Program DEADLOCKED (timeout)!"
    );

    assert!(run.status.success(), "Program failed");

    let output_str = String::from_utf8_lossy(&run.stdout);
    assert!(
        output_str.contains("30"),
        "Expected '30', got: {}",
        output_str
    );

    // Cleanup
    let _ = fs::remove_file(&ruchy_path);
    let _ = fs::remove_file(&rs_path);
    let _ = fs::remove_file(&exe_path);
}
