//! Handler for `ruchy add` command (CARGO-003)
//!
//! Adds Rust crate dependencies to Ruchy projects

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Handle `ruchy add` command - add a Rust crate dependency to the project
///
/// # Arguments
///
/// * `package` - Crate name to add (e.g., "serde", "tokio")
/// * `version` - Optional version constraint (e.g., "1.0", "^0.3")
/// * `dev` - Whether to add as dev-dependency
/// * `verbose` - Enable verbose output
///
/// # Examples
///
/// ```no_run
/// # use ruchy::handle_add_command;
/// // Add latest version of serde
/// handle_add_command("serde", None, false, false).expect("Failed to add serde");
///
/// // Add specific version as dev dependency
/// handle_add_command("proptest", Some("1.0"), true, false).expect("Failed to add proptest");
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - Cargo.toml doesn't exist (not in a Cargo project)
/// - cargo add command fails
/// - File I/O operations fail
///
/// # Complexity
///
/// Complexity: 7 (within Toyota Way limits ≤10)
pub fn handle_add_command(
    package: &str,
    version: Option<&str>,
    dev: bool,
    verbose: bool,
) -> Result<()> {
    // Step 1: Verify we're in a Cargo project
    verify_cargo_project()?;

    // Step 2: Build and execute cargo add command
    run_cargo_add(package, version, dev, verbose)?;

    // Step 3: Print success message
    print_success_message(package, version, dev);

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

/// Run `cargo add` command to add the dependency
///
/// # Complexity
///
/// Complexity: 8 (within Toyota Way limits ≤10)
fn run_cargo_add(package: &str, version: Option<&str>, dev: bool, verbose: bool) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("add");

    // Add package with version constraint using @version syntax
    if let Some(ver) = version {
        cmd.arg(format!("{package}@{ver}"));
    } else {
        cmd.arg(package);
    }

    // Add --dev flag if needed
    if dev {
        cmd.arg("--dev");
    }

    if verbose {
        let dev_flag = if dev { " --dev" } else { "" };
        let version_flag = version.map_or(String::new(), |v| format!("@{v}"));
        println!("Running: cargo add {package}{version_flag}{dev_flag}");
    }

    let output = cmd
        .output()
        .context("Failed to run cargo add - ensure cargo is installed")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("cargo add failed: {stderr}");
    }

    // Show cargo output if verbose
    if verbose && !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{stdout}");
    }

    Ok(())
}

/// Print success message after adding dependency
///
/// # Complexity
///
/// Complexity: 2 (within Toyota Way limits ≤10)
fn print_success_message(package: &str, version: Option<&str>, dev: bool) {
    let dep_type = if dev { "dev-dependency" } else { "dependency" };
    let version_str = version.map_or(String::new(), |v| format!(" (version {v})"));

    println!("Added {package}{version_str} as {dep_type}");
    println!("Run `cargo build` to compile with the new dependency");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    /// Test helper: Create a temporary Cargo project for testing
    fn create_test_project() -> (TempDir, String) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_name = "test_project";
        let project_path = temp_dir.path().join(project_name);

        // Create minimal Cargo.toml
        let cargo_toml_content = r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

        fs::create_dir(&project_path).expect("Failed to create project dir");
        fs::write(project_path.join("Cargo.toml"), cargo_toml_content)
            .expect("Failed to write Cargo.toml");

        (temp_dir, project_path.to_str().unwrap().to_string())
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
    fn test_print_success_message_basic() {
        // This test just ensures the function doesn't panic
        print_success_message("serde", None, false);
    }

    #[test]
    fn test_print_success_message_with_version() {
        // This test just ensures the function doesn't panic
        print_success_message("tokio", Some("1.0"), false);
    }

    #[test]
    fn test_print_success_message_dev_dependency() {
        // This test just ensures the function doesn't panic
        print_success_message("proptest", Some("1.0"), true);
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
