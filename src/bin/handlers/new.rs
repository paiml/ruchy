//! Handler for `ruchy new` command (CARGO-002)
//!
//! Creates new Ruchy projects with Cargo integration

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Handle `ruchy new` command - create new Ruchy project with Cargo integration
///
/// # Complexity
///
/// Complexity: 6 (within Toyota Way limits ≤10)
pub fn handle_new_command(name: &str, is_lib: bool, verbose: bool) -> Result<()> {
    // Step 1: Run `cargo new` to create base project
    create_cargo_project(name, is_lib, verbose)?;

    // Step 2: Add Ruchy-specific files
    let project_dir = Path::new(name);
    add_ruchy_files(project_dir, is_lib)?;

    // Step 3: Print success message
    println!("Created Ruchy project `{name}`");
    if verbose {
        println!(
            "Project type: {}",
            if is_lib { "library" } else { "binary" }
        );
        println!("Next steps:");
        println!("  cd {name}");
        println!("  cargo build");
        println!("  cargo run");
    }

    Ok(())
}

/// Create base Cargo project using `cargo new`
///
/// # Complexity
///
/// Complexity: 5 (within Toyota Way limits ≤10)
fn create_cargo_project(name: &str, is_lib: bool, verbose: bool) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("new").arg(name);

    if is_lib {
        cmd.arg("--lib");
    }

    if verbose {
        println!(
            "Running: cargo new {}{}",
            name,
            if is_lib { " --lib" } else { "" }
        );
    }

    let output = cmd
        .output()
        .context("Failed to run cargo new - ensure cargo is installed")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("cargo new failed: {stderr}");
    }

    Ok(())
}

/// Add Ruchy-specific files to the project
///
/// # Complexity
///
/// Complexity: 4 (within Toyota Way limits ≤10)
fn add_ruchy_files(project_dir: &Path, is_lib: bool) -> Result<()> {
    // Add build.rs
    create_build_rs(project_dir)?;

    // Modify Cargo.toml
    modify_cargo_toml(project_dir)?;

    // Add .ruchy source files
    create_ruchy_source(project_dir, is_lib)?;

    // Create/update README
    create_readme(project_dir, is_lib)?;

    Ok(())
}

/// Create build.rs file
///
/// # Complexity
///
/// Complexity: 2 (within Toyota Way limits ≤10)
fn create_build_rs(project_dir: &Path) -> Result<()> {
    let build_rs_content = r#"//! Build script for Ruchy project
//!
//! Automatically transpiles .ruchy files to .rs files during cargo build

fn main() {
    // Transpile all .ruchy files in src/ to .rs files
    ruchy::build_transpiler::transpile_all("src", "**/*.ruchy", "src")
        .expect("Failed to transpile Ruchy files");

    // Tell Cargo to re-run this build script if any .ruchy files change
    println!("cargo:rerun-if-changed=src");
}
"#;

    let build_rs_path = project_dir.join("build.rs");
    fs::write(&build_rs_path, build_rs_content)
        .with_context(|| format!("Failed to write build.rs to {}", build_rs_path.display()))?;

    Ok(())
}

/// Modify Cargo.toml to add ruchy as build dependency
///
/// # Complexity
///
/// Complexity: 7 (within Toyota Way limits ≤10)
fn modify_cargo_toml(project_dir: &Path) -> Result<()> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml_path).with_context(|| {
        format!(
            "Failed to read Cargo.toml from {}",
            cargo_toml_path.display()
        )
    })?;

    // Add build script reference and build-dependencies
    let modified_content = if content.contains("build =") {
        // Already has build key, just add build-dependencies
        add_build_dependencies(&content)
    } else {
        // Need to add both build key and build-dependencies
        add_build_script_and_dependencies(&content)
    };

    fs::write(&cargo_toml_path, modified_content).with_context(|| {
        format!(
            "Failed to write Cargo.toml to {}",
            cargo_toml_path.display()
        )
    })?;

    Ok(())
}

/// Add build-dependencies section to Cargo.toml
///
/// # Complexity
///
/// Complexity: 2 (within Toyota Way limits ≤10)
fn add_build_dependencies(content: &str) -> String {
    // If build-dependencies already exists, add ruchy to it
    if content.contains("[build-dependencies]") {
        content.replace(
            "[build-dependencies]",
            "[build-dependencies]\nruchy = \"3.71\"",
        )
    } else {
        // Add new build-dependencies section at end
        format!("{content}\n[build-dependencies]\nruchy = \"3.71\"\n")
    }
}

/// Add both build script and build-dependencies to Cargo.toml
///
/// # Complexity
///
/// Complexity: 3 (within Toyota Way limits ≤10)
fn add_build_script_and_dependencies(content: &str) -> String {
    // Add build = "build.rs" to [package] section
    let with_build = content.replace("[package]", "[package]\nbuild = \"build.rs\"");

    // Add build-dependencies section
    add_build_dependencies(&with_build)
}

/// Create Ruchy source file (main.ruchy or lib.ruchy)
///
/// # Complexity
///
/// Complexity: 5 (within Toyota Way limits ≤10)
fn create_ruchy_source(project_dir: &Path, is_lib: bool) -> Result<()> {
    let (filename, content) = if is_lib {
        ("src/lib.ruchy", get_lib_template())
    } else {
        ("src/main.ruchy", get_main_template())
    };

    let file_path = project_dir.join(filename);
    fs::write(&file_path, content)
        .with_context(|| format!("Failed to write {filename} to {}", file_path.display()))?;

    Ok(())
}

/// Get template content for main.ruchy
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
fn get_main_template() -> &'static str {
    r#"// Ruchy main program
// This file will be automatically transpiled to main.rs during cargo build

fun main() {
    println("Hello, Ruchy!");

    // Example: Using variables
    let name = "World";
    println(f"Hello, {name}!");

    // Example: Using collections
    let numbers = [1, 2, 3, 4, 5];
    println(f"Numbers: {numbers:?}");
}
"#
}

/// Get template content for lib.ruchy
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
fn get_lib_template() -> &'static str {
    r"// Ruchy library
// This file will be automatically transpiled to lib.rs during cargo build

/// Add two numbers together
///
/// # Examples
///
/// ```
/// assert_eq!(add(2, 3), 5);
/// ```
pub fun add(a: i32, b: i32) -> i32 {
    a + b
}

/// Multiply two numbers
///
/// # Examples
///
/// ```
/// assert_eq!(multiply(2, 3), 6);
/// ```
pub fun multiply(a: i32, b: i32) -> i32 {
    a * b
}
"
}

/// Create/update README.md with Ruchy instructions
///
/// # Complexity
///
/// Complexity: 4 (within Toyota Way limits ≤10)
fn create_readme(project_dir: &Path, is_lib: bool) -> Result<()> {
    let readme_path = project_dir.join("README.md");
    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("ruchy-project");

    let content = get_readme_template(project_name, is_lib);

    // Append to existing README or create new one
    if readme_path.exists() {
        let existing = fs::read_to_string(&readme_path)?;
        fs::write(&readme_path, format!("{existing}\n\n{content}"))?;
    } else {
        fs::write(&readme_path, content)?;
    }

    Ok(())
}

/// Get README template
///
/// # Complexity
///
/// Complexity: 2 (within Toyota Way limits ≤10)
fn get_readme_template(project_name: &str, is_lib: bool) -> String {
    let project_type = if is_lib { "Library" } else { "Application" };

    format!(
        r"# {project_name}

A Ruchy {project_type} with Cargo integration.

## About Ruchy

This project uses Ruchy, a modern systems programming language that transpiles to Rust.
Source files written in `.ruchy` syntax are automatically converted to `.rs` files during build.

## Building

```bash
# Build the project (auto-transpiles .ruchy → .rs)
cargo build

# Run the project{}
cargo run

# Run tests
cargo test

# Clean generated files
cargo clean
```

## Project Structure

- `src/{}` - Ruchy source code (auto-transpiled)
- `build.rs` - Build script for transpilation
- `Cargo.toml` - Project dependencies

## Learn More

- Ruchy Language: https://github.com/paiml/ruchy
- Documentation: https://docs.rs/ruchy
",
        if is_lib { "" } else { "\ncargo run" },
        if is_lib { "lib.ruchy" } else { "main.ruchy" }
    )
}
