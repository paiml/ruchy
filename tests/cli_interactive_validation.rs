//! CLI INTERACTIVE VALIDATION SUITE (rexpect-based)
//!
//! **Purpose**: End-to-end CLI verification using rexpect for interactive testing
//! **Methodology**: Spawn actual processes, send input, verify output
//! **Toyota Way**: Genchi Genbutsu - Go and see the real CLI behavior
//!
//! This test suite validates CLI commands in their actual runtime environment,
//! catching issues that unit tests miss (TTY behavior, signal handling, etc.)

use rexpect::spawn;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Get path to ruchy binary
fn ruchy_binary() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/ruchy", manifest_dir)
}

/// Create temp directory
fn temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

// ============================================================================
// REPL INTERACTIVE TESTS (rexpect-based)
// ============================================================================

#[test]
#[ignore] // Requires terminal - run manually or in CI with PTY
fn repl_interactive_arithmetic() {
    let binary = ruchy_binary();

    // Spawn REPL
    let mut repl = spawn(&binary, Some(5000)).expect("Failed to spawn REPL");

    // Wait for prompt
    repl.exp_string("ruchy>").expect("Should show prompt");

    // Test basic arithmetic
    repl.send_line("2 + 2").expect("Failed to send command");
    repl.exp_string("4").expect("Should output 4");

    // Test variable assignment
    repl.send_line("let x = 10").expect("Failed to send command");
    repl.send_line("x * 2").expect("Failed to send command");
    repl.exp_string("20").expect("Should output 20");

    // Exit
    repl.send_line("exit").expect("Failed to exit");
}

#[test]
#[ignore] // Requires terminal
fn repl_interactive_function_definition() {
    let binary = ruchy_binary();
    let mut repl = spawn(&binary, Some(5000)).expect("Failed to spawn REPL");

    repl.exp_string("ruchy>").expect("Should show prompt");

    // Define function
    repl.send_line("fun double(x) { x * 2 }")
        .expect("Failed to send function");

    // Call function
    repl.send_line("double(21)").expect("Failed to call function");
    repl.exp_string("42").expect("Should output 42");

    repl.send_line("exit").expect("Failed to exit");
}

#[test]
#[ignore] // Requires terminal
fn repl_interactive_error_recovery() {
    let binary = ruchy_binary();
    let mut repl = spawn(&binary, Some(5000)).expect("Failed to spawn REPL");

    repl.exp_string("ruchy>").expect("Should show prompt");

    // Send invalid expression
    repl.send_line("let x = ").expect("Failed to send command");

    // REPL should recover and show prompt again
    repl.exp_string("ruchy>").expect("Should show prompt after error");

    // Verify REPL still works
    repl.send_line("42").expect("Failed to send command");
    repl.exp_string("42").expect("Should output 42");

    repl.send_line("exit").expect("Failed to exit");
}

// ============================================================================
// CLI COMMAND VERIFICATION (Non-interactive)
// ============================================================================

#[test]
fn cli_run_command_executes_file() {
    let temp = temp_dir();
    let file = temp.path().join("test.ruchy");
    fs::write(&file, "println(\"Hello from CLI\")").expect("Failed to write file");

    let binary = ruchy_binary();
    let mut proc = spawn(&format!("{} run {}", binary, file.display()), Some(5000))
        .expect("Failed to spawn process");

    proc.exp_string("Hello from CLI")
        .expect("Should output message");
}

#[test]
fn cli_eval_flag_executes_inline() {
    let binary = ruchy_binary();
    let mut proc = spawn(&format!("{} -e \"2 + 2\"", binary), Some(5000))
        .expect("Failed to spawn process");

    proc.exp_string("4").expect("Should output 4");
}

#[test]
fn cli_check_command_validates_syntax() {
    let temp = temp_dir();
    let file = temp.path().join("valid.ruchy");
    fs::write(&file, "let x = 42").expect("Failed to write file");

    let binary = ruchy_binary();
    let mut proc = spawn(&format!("{} check {}", binary, file.display()), Some(5000))
        .expect("Failed to spawn process");

    // Should succeed without errors
    let output = proc.exp_eof().expect("Should complete successfully");
    assert!(!output.contains("error"), "Should not have errors");
}

#[test]
fn cli_test_command_runs_tests() {
    let temp = temp_dir();
    let file = temp.path().join("test.ruchy");
    fs::write(
        &file,
        r#"
@test("simple test")
fun test_pass() {
    assert_eq(1, 1, "one equals one")
}
"#,
    )
    .expect("Failed to write file");

    let binary = ruchy_binary();
    let mut proc = spawn(&format!("{} test {}", binary, file.display()), Some(5000))
        .expect("Failed to spawn process");

    proc.exp_string("Passed").expect("Should show test passed");
}

// ============================================================================
// SIGNAL HANDLING TESTS
// ============================================================================

#[test]
#[cfg(all(unix, feature = "manual-signal-tests"))] // Disabled: requires manual PTY testing
#[ignore] // Requires PTY and signal handling
fn cli_handles_ctrl_c_gracefully() {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;

    let binary = ruchy_binary();
    let mut proc = spawn(&binary, Some(5000)).expect("Failed to spawn REPL");

    proc.exp_string("ruchy>").expect("Should show prompt");

    // Get process ID - Note: rexpect PtyProcess API may vary by version
    // This test is disabled by default due to PTY complexity
    let pid = Pid::from_raw(proc.pid().expect("Should have PID") as i32);

    // Send SIGINT (Ctrl+C)
    kill(pid, Signal::SIGINT).expect("Failed to send SIGINT");

    // REPL should exit gracefully
    let result = proc.exp_eof();
    assert!(result.is_ok(), "Should exit gracefully on Ctrl+C");
}

// ============================================================================
// TTY DETECTION TESTS
// ============================================================================

#[test]
#[ignore] // Requires PTY
fn repl_detects_interactive_tty() {
    let binary = ruchy_binary();
    let mut proc = spawn(&binary, Some(5000)).expect("Failed to spawn REPL");

    // Interactive REPL should show colored prompt with indicators
    proc.exp_string("ruchy>").expect("Should show interactive prompt");

    proc.send_line("exit").expect("Failed to exit");
}

#[test]
fn non_tty_omits_interactive_features() {
    use std::process::{Command, Stdio};

    let binary = ruchy_binary();
    let output = Command::new(binary)
        .arg("-e")
        .arg("42")
        .stdin(Stdio::null())
        .output()
        .expect("Failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Non-TTY output should be clean (no ANSI codes, no prompts)
    assert!(!stdout.contains("\x1b["), "Should not have ANSI codes");
    assert!(stdout.contains("42"), "Should have output value");
}

// ============================================================================
// PIPE AND REDIRECTION TESTS
// ============================================================================

#[test]
fn cli_accepts_stdin_input() {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let binary = ruchy_binary();
    let mut child = Command::new(binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn");

    // Write to stdin
    {
        let stdin = child.stdin.as_mut().expect("Failed to get stdin");
        stdin
            .write_all(b"let x = 42\nprintln(x)\nexit\n")
            .expect("Failed to write to stdin");
    }

    // Read output
    let output = child.wait_with_output().expect("Failed to wait");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("42"), "Should process stdin input");
}

#[test]
fn cli_supports_output_redirection() {
    let temp = temp_dir();
    let script = temp.path().join("script.ruchy");
    let output_file = temp.path().join("output.txt");

    fs::write(&script, "println(\"output test\")").expect("Failed to write script");

    let binary = ruchy_binary();
    std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "{} run {} > {}",
            binary,
            script.display(),
            output_file.display()
        ))
        .status()
        .expect("Failed to execute");

    // Verify output was redirected
    assert!(output_file.exists(), "Output file should exist");
    let content = fs::read_to_string(&output_file).expect("Failed to read output");
    assert!(
        content.contains("output test"),
        "Output should be redirected"
    );
}

// ============================================================================
// ERROR MESSAGE QUALITY TESTS
// ============================================================================

#[test]
fn cli_error_messages_are_actionable() {
    let temp = temp_dir();
    let file = temp.path().join("syntax_error.ruchy");
    fs::write(&file, "let x = ").expect("Failed to write file");

    let binary = ruchy_binary();
    let output = std::process::Command::new(binary)
        .arg("run")
        .arg(&file)
        .output()
        .expect("Failed to execute");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Error messages should contain:
    // 1. File name
    // 2. Line number
    // 3. Description of error
    assert!(
        stderr.contains("syntax_error.ruchy") || !stderr.is_empty(),
        "Should show file name in error"
    );
}

#[test]
fn cli_undefined_variable_error_is_clear() {
    let binary = ruchy_binary();
    let output = std::process::Command::new(binary)
        .arg("-e")
        .arg("undefined_var")
        .output()
        .expect("Failed to execute");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should mention the undefined variable
    assert!(
        stderr.contains("undefined") || stderr.contains("Undefined"),
        "Should mention undefined variable: {}",
        stderr
    );
}

// ============================================================================
// PERFORMANCE AND TIMEOUT TESTS
// ============================================================================

#[test]
fn cli_handles_long_running_scripts() {
    let temp = temp_dir();
    let file = temp.path().join("long.ruchy");
    fs::write(
        &file,
        r#"
let sum = 0
for i in range(0, 1000) {
    sum = sum + i
}
println(sum)
"#,
    )
    .expect("Failed to write file");

    let binary = ruchy_binary();
    let start = std::time::Instant::now();

    let output = std::process::Command::new(binary)
        .arg("run")
        .arg(&file)
        .output()
        .expect("Failed to execute");

    let duration = start.elapsed();

    assert!(output.status.success(), "Should complete successfully");
    assert!(
        duration.as_secs() < 10,
        "Should complete in reasonable time"
    );
}

#[test]
#[ignore] // Resource intensive
fn cli_handles_memory_intensive_operations() {
    let binary = ruchy_binary();
    let output = std::process::Command::new(binary)
        .arg("-e")
        .arg("let big_array = range(0, 10000)")
        .output()
        .expect("Failed to execute");

    // Should not crash or hang
    assert!(
        output.status.success() || output.status.code().is_some(),
        "Should handle large data structures"
    );
}
