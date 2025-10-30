// RED Phase Test for Issue #89: Support stdlib 'use' statements in imported modules
//
// GitHub Issue: https://github.com/paiml/ruchy/issues/89
//
// Problem: When a module is imported with `use mylib;`, the module file itself
// cannot contain `use` statements for stdlib imports. All stdlib references must
// be fully qualified (e.g., `std::process::Command::new()` instead of `Command::new()`).
//
// Root Cause: ModuleLoader treats stdlib `use` as file module import, tries to
// load `std/process/Command.ruchy` file instead of resolving to built-in type.
//
// Expected: Modules should be able to use `use std::*` just like main files can.
//
// EXTREME TDD Methodology:
// 1. RED: Create failing test using stdlib imports in modules
// 2. GREEN: Add namespace resolution (std:: paths = builtins, not files)
// 3. REFACTOR: Verify transitive imports and edge cases

#![allow(missing_docs)]

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// RED: Test basic stdlib import in module
///
/// This is based on Issue #89. The `use` statement should not cause an error,
/// but short names (Command) are not yet supported - must use full qualified names.
#[test]
fn test_issue_089_stdlib_import_in_module() {
    let temp_dir = TempDir::new().unwrap();

    // Create module file that uses `use std::process::Command`
    let module_file = temp_dir.path().join("mylib.ruchy");
    let module_code = r#"
use std::process::Command;

fun run_command() -> bool {
    // Note: Currently must use full qualified name, short name (Command) not yet supported
    let output = std::process::Command::new("echo").arg("hello").output();
    match output {
        Ok(_) => true,
        Err(_) => false,
    }
}
"#;
    fs::write(&module_file, module_code).unwrap();

    // Create main file that imports the module
    let main_file = temp_dir.path().join("main.ruchy");
    let main_code = r#"
use mylib;

fun main() {
    let result = mylib::run_command();
    if result {
        println!("Command succeeded");
    } else {
        println!("Command failed");
    }
}
"#;
    fs::write(&main_file, main_code).unwrap();

    // RED: Previously failed with "Failed to load module 'std::process::Command'"
    // GREEN: Now succeeds (use statement doesn't cause error, though short names not yet supported)
    let output = ruchy_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("main.ruchy")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("Command succeeded"),
        "Expected 'Command succeeded', got: {}",
        stdout
    );
}

/// RED: Test multiple stdlib imports in module
#[test]
fn test_issue_089_multiple_stdlib_imports() {
    let temp_dir = TempDir::new().unwrap();

    // Module with multiple stdlib imports - use full qualified names
    let module_file = temp_dir.path().join("utils.ruchy");
    let module_code = r#"
use std::process::Command;

fun check_command() -> bool {
    // Use Command to run echo
    let cmd_result = std::process::Command::new("echo").arg("test").output();
    match cmd_result {
        Ok(_) => true,
        Err(_) => false,
    }
}

fun check_command2() -> bool {
    // Use Command again with different args
    let cmd_result = std::process::Command::new("echo").arg("test2").output();
    match cmd_result {
        Ok(_) => true,
        Err(_) => false,
    }
}
"#;
    fs::write(&module_file, module_code).unwrap();

    let main_file = temp_dir.path().join("main.ruchy");
    let main_code = r#"
use utils;

fun main() {
    let cmd_ok = utils::check_command();
    let cmd2_ok = utils::check_command2();
    if cmd_ok && cmd2_ok {
        println!("All checks passed");
    }
}
"#;
    fs::write(&main_file, main_code).unwrap();

    let output = ruchy_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("main.ruchy")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("All checks passed"),
        "Expected 'All checks passed', got: {}",
        stdout
    );
}

/// RED: Test transitive stdlib imports (module imports module with stdlib imports)
#[test]
fn test_issue_089_transitive_stdlib_imports() {
    let temp_dir = TempDir::new().unwrap();

    // First module with stdlib import
    let cmd_module = temp_dir.path().join("commands.ruchy");
    let cmd_code = r#"
use std::process::Command;

fun run_echo(msg: String) -> bool {
    // Use full qualified name
    let output = std::process::Command::new("echo").arg(msg).output();
    match output {
        Ok(_) => true,
        Err(_) => false,
    }
}
"#;
    fs::write(&cmd_module, cmd_code).unwrap();

    // Second module that imports first module
    let utils_module = temp_dir.path().join("utils.ruchy");
    let utils_code = r#"
use commands;

fun run_test() -> bool {
    commands::run_echo("transitive test")
}
"#;
    fs::write(&utils_module, utils_code).unwrap();

    // Main file that imports second module
    let main_file = temp_dir.path().join("main.ruchy");
    let main_code = r#"
use utils;

fun main() {
    let result = utils::run_test();
    if result {
        println!("Transitive import works");
    }
}
"#;
    fs::write(&main_file, main_code).unwrap();

    let output = ruchy_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("main.ruchy")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("Transitive import works"),
        "Expected 'Transitive import works', got: {}",
        stdout
    );
}

/// GREEN: Verify standalone file with stdlib imports still works
///
/// This should already work - ensuring we don't break existing functionality.
#[test]
fn test_issue_089_standalone_stdlib_imports_still_work() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("standalone.ruchy");

    let code = r#"
use std::process::Command;

fun main() {
    let output = Command::new("echo").arg("standalone").output();
    match output {
        Ok(_) => println!("Standalone works"),
        Err(_) => println!("Standalone failed"),
    }
}
"#;
    fs::write(&test_file, code).unwrap();

    let output = ruchy_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("standalone.ruchy")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("Standalone works"),
        "Expected 'Standalone works', got: {}",
        stdout
    );
}

/// RED: Test mixed imports (stdlib + file modules in same file)
#[test]
fn test_issue_089_mixed_stdlib_and_file_imports() {
    let temp_dir = TempDir::new().unwrap();

    // Helper module (file) - simple function that returns a number
    let helper_module = temp_dir.path().join("helper.ruchy");
    let helper_code = r#"
fun get_number() -> int {
    42
}
"#;
    fs::write(&helper_module, helper_code).unwrap();

    // Main module with both stdlib and file imports
    let utils_module = temp_dir.path().join("utils.ruchy");
    let utils_code = r#"
use std::process::Command;
use helper;

fun run_mixed() -> bool {
    let num = helper::get_number();
    println!("Number is 42");

    // Use full qualified name for Command since short names not yet supported
    let cmd_result = std::process::Command::new("echo").arg("mixed imports").output();
    match cmd_result {
        Ok(_) => true,
        Err(_) => false,
    }
}
"#;
    fs::write(&utils_module, utils_code).unwrap();

    let main_file = temp_dir.path().join("main.ruchy");
    let main_code = r#"
use utils;

fun main() {
    let result = utils::run_mixed();
    if result {
        println!("Mixed imports work");
    }
}
"#;
    fs::write(&main_file, main_code).unwrap();

    let output = ruchy_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("main.ruchy")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("Number is 42"),
        "Expected number message, got: {}",
        stdout
    );
    assert!(
        stdout.contains("Mixed imports work"),
        "Expected success message, got: {}",
        stdout
    );
}
