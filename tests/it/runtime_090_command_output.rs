#![allow(clippy::ignore_without_reason)] // Test file with known limitations
#![allow(missing_docs)]

// RUNTIME-090: Fix Command.output() hang (Issue #75)
// https://github.com/paiml/ruchy/issues/75
//
// RED Phase: Demonstrate that Command.output() hangs at runtime
// Expected: These tests should timeout/hang, proving the defect exists

use assert_cmd::Command;

#[test]
#[ignore = "RED phase: Will hang indefinitely"]
fn test_runtime_090_01_command_output_simple() {
    // Minimal reproduction: Case 1 from Issue #75
    let ruchy_script = r#"
use std::process::Command

println("Starting...")
let result = Command::new("ls").output()
println("Done!")
"#;

    let temp_file = std::env::temp_dir().join("test_command_simple.ruchy");
    std::fs::write(&temp_file, ruchy_script).unwrap();

    // This should complete in <1 second but will hang indefinitely
    let output = Command::new("timeout")
        .arg("5") // 5 second timeout
        .arg("ruchy")
        .arg("run")
        .arg(&temp_file)
        .output()
        .expect("Failed to run ruchy");

    // Clean up
    let _ = std::fs::remove_file(&temp_file);

    // If we get here, timeout killed the process (proving hang)
    assert!(
        !output.status.success(),
        "Command should timeout due to hang"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Starting..."), "Should print before hang");
    assert!(
        !stdout.contains("Done!"),
        "Should NOT reach after .output() call"
    );
}

#[test]
#[ignore = "RED phase: Will hang indefinitely"]
fn test_runtime_090_02_command_output_with_args() {
    // Case 2 from Issue #75: Command with arguments
    let ruchy_script = r#"
use std::process::Command

println("Starting...")
let mut cmd = Command::new("echo")
cmd.arg("hello")
let result = cmd.output()
println("Done!")
"#;

    let temp_file = std::env::temp_dir().join("test_command_args.ruchy");
    std::fs::write(&temp_file, ruchy_script).unwrap();

    let output = Command::new("timeout")
        .arg("5")
        .arg("ruchy")
        .arg("run")
        .arg(&temp_file)
        .output()
        .expect("Failed to run ruchy");

    let _ = std::fs::remove_file(&temp_file);

    assert!(
        !output.status.success(),
        "Command should timeout due to hang"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Starting..."), "Should print before hang");
    assert!(
        !stdout.contains("Done!"),
        "Should NOT reach after .output() call"
    );
}

#[test]
#[ignore = "RED phase: Will hang indefinitely"]
fn test_runtime_090_03_command_output_in_function() {
    // Case 3 from Issue #75: Command in function wrapper
    let ruchy_script = r#"
use std::process::Command

fun check_command() -> bool {
    println("Checking command...")
    let result = Command::new("ls").output()
    println("Command completed")
    true
}

println("Starting...")
let success = check_command()
println("Done!")
"#;

    let temp_file = std::env::temp_dir().join("test_command_function.ruchy");
    std::fs::write(&temp_file, ruchy_script).unwrap();

    let output = Command::new("timeout")
        .arg("5")
        .arg("ruchy")
        .arg("run")
        .arg(&temp_file)
        .output()
        .expect("Failed to run ruchy");

    let _ = std::fs::remove_file(&temp_file);

    assert!(
        !output.status.success(),
        "Command should timeout due to hang"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Checking command..."),
        "Should enter function"
    );
    assert!(
        !stdout.contains("Command completed"),
        "Should hang before completing"
    );
}

#[test]
#[ignore = "RED phase: Will hang indefinitely"]
fn test_runtime_090_04_command_output_full_conversion() {
    // Case 4 from Issue #75: Full system-command.ts conversion (simplified)
    let ruchy_script = r#"
use std::process::Command

fun run_command(cmd: String, args: [String]) -> Result<String, String> {
    println("Building command...")
    let mut command = Command::new(cmd)

    for arg in args {
        command.arg(arg)
    }

    println("Executing...")
    let output = command.output()

    println("Processing result...")
    Ok("success")
}

println("Starting system command...")
let result = run_command("ls", ["-la"])
println("Done!")
"#;

    let temp_file = std::env::temp_dir().join("test_command_full.ruchy");
    std::fs::write(&temp_file, ruchy_script).unwrap();

    let output = Command::new("timeout")
        .arg("5")
        .arg("ruchy")
        .arg("run")
        .arg(&temp_file)
        .output()
        .expect("Failed to run ruchy");

    let _ = std::fs::remove_file(&temp_file);

    assert!(
        !output.status.success(),
        "Command should timeout due to hang"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Executing..."),
        "Should reach .output() call"
    );
    assert!(
        !stdout.contains("Processing result..."),
        "Should hang at .output()"
    );
}
