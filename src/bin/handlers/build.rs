//! Handler for `ruchy build` command (CARGO-004)
//!
//! Wrapper around `cargo build` for Ruchy projects

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Handle `ruchy build` command - wrapper around cargo build
///
/// # Arguments
///
/// * `release` - Whether to build in release mode (--release)
/// * `verbose` - Enable verbose output
///
/// # Examples
///
/// ```no_run
/// # use ruchy::handle_build_command;
/// // Debug build
/// handle_build_command(false, false).expect("Failed to build");
///
/// // Release build
/// handle_build_command(true, false).expect("Failed to build");
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Cargo.toml doesn't exist (not in a Cargo project)
/// - cargo build command fails
///
/// # Complexity
///
/// Complexity: 6 (within Toyota Way limits ≤10)
pub fn handle_build_command(release: bool, verbose: bool) -> Result<()> {
    // Step 1: Verify we're in a Cargo project
    verify_cargo_project()?;

    // Step 2: Run cargo build
    run_cargo_build(release, verbose)?;

    // Step 3: Print success message
    print_build_success_message(release);

    Ok(())
}

/// Verify that Cargo.toml exists in current directory
///
/// # Complexity
///
/// Complexity: 2 (within Toyota Way limits ≤10)
fn verify_cargo_project() -> Result<()> {
    let cargo_toml = Path::new("Cargo.toml");
    if !cargo_toml.exists() {
        anyhow::bail!(
            "Cargo.toml not found. Run this command from a Cargo project directory.\n\
             Hint: Use `ruchy new <name>` to create a new Ruchy project."
        );
    }
    Ok(())
}

/// Run `cargo build` command
///
/// # Complexity
///
/// Complexity: 7 (within Toyota Way limits ≤10)
fn run_cargo_build(release: bool, verbose: bool) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build");

    if release {
        cmd.arg("--release");
    }

    if verbose {
        cmd.arg("--verbose");
        let mode = if release { "release" } else { "debug" };
        println!("Running: cargo build --{mode}");
    }

    let output = cmd
        .output()
        .context("Failed to run cargo build - ensure cargo is installed")?;

    // Always show cargo output
    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        print!("{stdout}");
    }

    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprint!("{stderr}");
    }

    if !output.status.success() {
        anyhow::bail!("cargo build failed");
    }

    Ok(())
}

/// Print success message after building
///
/// # Complexity
///
/// Complexity: 2 (within Toyota Way limits ≤10)
fn print_build_success_message(release: bool) {
    let mode = if release { "release" } else { "debug" };
    println!("Build complete ({mode} mode)");
    println!("Run `cargo run` to execute the program");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    /// Test helper: Create a temporary Cargo project for testing
    fn create_test_project() -> (TempDir, String) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_name = "test_project";
        let project_path = temp_dir.path().join(project_name);

        // Use cargo new to create real project
        std::process::Command::new("cargo")
            .arg("new")
            .arg(project_name)
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to run cargo new");

        (
            temp_dir,
            project_path
                .to_str()
                .expect("Project path should be valid UTF-8")
                .to_string(),
        )
    }

    #[test]
    fn test_verify_cargo_project_missing_cargo_toml() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let _original_dir = env::current_dir().expect("Failed to get current dir");

        // Change to temp dir without Cargo.toml
        env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

        let result = verify_cargo_project();

        // Change back to original dir
        env::set_current_dir(_original_dir).expect("Failed to restore dir");

        assert!(result.is_err(), "Should fail when Cargo.toml doesn't exist");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cargo.toml not found"),
            "Error message should mention Cargo.toml"
        );
    }

    #[test]
    fn test_verify_cargo_project_success() {
        let (_temp_dir, project_path) = create_test_project();
        let _original_dir = env::current_dir().expect("Failed to get current dir");

        // Change to project dir
        env::set_current_dir(&project_path).expect("Failed to change dir");

        let result = verify_cargo_project();

        // Change back to original dir
        env::set_current_dir(_original_dir).expect("Failed to restore dir");

        assert!(result.is_ok(), "Should succeed when Cargo.toml exists");
    }

    #[test]
    fn test_print_build_success_message_debug() {
        // This test just ensures the function doesn't panic
        print_build_success_message(false);
    }

    #[test]
    fn test_print_build_success_message_release() {
        // This test just ensures the function doesn't panic
        print_build_success_message(true);
    }

    // Property-based test: verify_cargo_project is idempotent
    #[test]
    fn test_verify_cargo_project_idempotent() {
        let (_temp_dir, project_path) = create_test_project();
        let _original_dir = env::current_dir().expect("Failed to get current dir");

        env::set_current_dir(&project_path).expect("Failed to change dir");

        let result1 = verify_cargo_project();
        let result2 = verify_cargo_project();

        env::set_current_dir(_original_dir).expect("Failed to restore dir");

        assert_eq!(
            result1.is_ok(),
            result2.is_ok(),
            "Multiple calls should have same result"
        );
    }
}
