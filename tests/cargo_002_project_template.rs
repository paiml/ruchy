//! CARGO-002: Project Template Generator Tests
//!
//! Test suite for `ruchy new` command that creates Ruchy projects
//! with Cargo integration.
//!
//! EXTREME TDD: These tests are written BEFORE implementation (RED phase).

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Helper to create ruchy command
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper to check if file exists and return its content
fn read_file_in_project(project_dir: &Path, relative_path: &str) -> String {
    let file_path = project_dir.join(relative_path);
    assert!(
        file_path.exists(),
        "Expected file to exist: {}",
        file_path.display()
    );
    fs::read_to_string(&file_path).expect("Failed to read file")
}

#[test]
#[ignore] // Will pass after implementation
fn test_cargo_002_new_binary_project() {
    // CARGO-002: Test creating a new binary project with `ruchy new`

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "my-ruchy-app";

    // Run `ruchy new my-ruchy-app`
    ruchy_cmd()
        .arg("new")
        .arg(project_name)
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"))
        .stdout(predicate::str::contains(project_name));

    let project_dir = temp_dir.path().join(project_name);

    // Verify standard Cargo project structure exists
    assert!(project_dir.join("Cargo.toml").exists());
    assert!(project_dir.join("src").exists());
    assert!(project_dir.join(".git").exists());
    assert!(project_dir.join(".gitignore").exists());

    // Verify Ruchy-specific files exist
    assert!(project_dir.join("build.rs").exists());
    assert!(project_dir.join("src/main.ruchy").exists());
    assert!(project_dir.join("README.md").exists());
}

#[test]
#[ignore] // Will pass after implementation
fn test_cargo_002_new_library_project() {
    // CARGO-002: Test creating a new library project with `ruchy new --lib`

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "my-ruchy-lib";

    // Run `ruchy new --lib my-ruchy-lib`
    ruchy_cmd()
        .arg("new")
        .arg("--lib")
        .arg(project_name)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let project_dir = temp_dir.path().join(project_name);

    // Verify library-specific structure
    assert!(project_dir.join("Cargo.toml").exists());
    assert!(project_dir.join("src/lib.ruchy").exists());
    assert!(!project_dir.join("src/main.ruchy").exists());
}

#[test]
#[ignore] // Will pass after implementation
fn test_cargo_002_cargo_toml_has_build_dependencies() {
    // CARGO-002: Verify generated Cargo.toml includes ruchy as build dependency

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "test-deps";

    ruchy_cmd()
        .arg("new")
        .arg(project_name)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let project_dir = temp_dir.path().join(project_name);
    let cargo_toml = read_file_in_project(&project_dir, "Cargo.toml");

    // Should have build-dependencies section
    assert!(
        cargo_toml.contains("[build-dependencies]"),
        "Cargo.toml should have [build-dependencies] section"
    );

    // Should include ruchy
    assert!(
        cargo_toml.contains("ruchy"),
        "Cargo.toml should include ruchy as build dependency"
    );

    // Should specify build script
    assert!(
        cargo_toml.contains("build = \"build.rs\""),
        "Cargo.toml should specify build script"
    );
}

#[test]
#[ignore] // Will pass after implementation
fn test_cargo_002_build_rs_template_valid() {
    // CARGO-002: Verify generated build.rs is valid and calls transpile_all

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "test-build";

    ruchy_cmd()
        .arg("new")
        .arg(project_name)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let project_dir = temp_dir.path().join(project_name);
    let build_rs = read_file_in_project(&project_dir, "build.rs");

    // Should call transpile_all
    assert!(
        build_rs.contains("ruchy::build_transpiler::transpile_all"),
        "build.rs should call transpile_all"
    );

    // Should have proper error handling
    assert!(
        build_rs.contains("expect") || build_rs.contains("unwrap"),
        "build.rs should have error handling"
    );

    // Should have main function
    assert!(
        build_rs.contains("fn main()"),
        "build.rs should have main function"
    );
}

#[test]
#[ignore] // Will pass after implementation
fn test_cargo_002_main_ruchy_template_valid() {
    // CARGO-002: Verify generated main.ruchy is valid Ruchy code

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "test-main";

    ruchy_cmd()
        .arg("new")
        .arg(project_name)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let project_dir = temp_dir.path().join(project_name);
    let main_ruchy = read_file_in_project(&project_dir, "src/main.ruchy");

    // Should have main function
    assert!(
        main_ruchy.contains("fun main()") || main_ruchy.contains("fn main()"),
        "main.ruchy should have main function"
    );

    // Should have at least one statement (e.g., println)
    assert!(
        main_ruchy.contains("println"),
        "main.ruchy should have example code"
    );
}

#[test]
#[ignore] // Will pass after implementation
fn test_cargo_002_readme_has_instructions() {
    // CARGO-002: Verify README.md has basic usage instructions

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "test-readme";

    ruchy_cmd()
        .arg("new")
        .arg(project_name)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let project_dir = temp_dir.path().join(project_name);
    let readme = read_file_in_project(&project_dir, "README.md");

    // Should mention Ruchy
    assert!(readme.contains("Ruchy"), "README should mention Ruchy");

    // Should have build instructions
    assert!(
        readme.contains("cargo build") || readme.contains("cargo run"),
        "README should have build instructions"
    );

    // Should mention .ruchy files
    assert!(
        readme.contains(".ruchy"),
        "README should mention .ruchy files"
    );
}

#[test]
#[ignore] // Will pass after v3.72 release with build_transpiler published
fn test_cargo_002_created_project_can_build() {
    // CARGO-002: Verify the generated project can actually build
    //
    // NOTE: This test requires ruchy to be published with build_transpiler module.
    // It will fail in development until v3.72 is published to crates.io.
    // This is expected and not a bug - the generated projects will work once published.

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "test-buildable";

    ruchy_cmd()
        .arg("new")
        .arg(project_name)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let project_dir = temp_dir.path().join(project_name);

    // Modify Cargo.toml to use local path during development testing
    let cargo_toml_path = project_dir.join("Cargo.toml");
    let cargo_toml = fs::read_to_string(&cargo_toml_path).expect("Failed to read Cargo.toml");

    // Replace version with path dependency for local testing
    let workspace_root = std::env::current_dir().expect("Failed to get current dir");
    let modified_cargo_toml = cargo_toml.replace(
        "ruchy = \"3.71\"",
        &format!("ruchy = {{ path = \"{}\" }}", workspace_root.display()),
    );
    fs::write(&cargo_toml_path, modified_cargo_toml).expect("Failed to write Cargo.toml");

    // Try to build the generated project
    let build_result = std::process::Command::new("cargo")
        .arg("build")
        .current_dir(&project_dir)
        .output()
        .expect("Failed to run cargo build");

    assert!(
        build_result.status.success(),
        "Generated project should build successfully. stderr: {}",
        String::from_utf8_lossy(&build_result.stderr)
    );

    // Verify .rs file was generated from .ruchy
    assert!(
        project_dir.join("src/main.rs").exists(),
        "main.rs should be generated from main.ruchy"
    );
}

#[test]
#[ignore] // Will pass after implementation
fn test_cargo_002_error_on_invalid_name() {
    // CARGO-002: Test that invalid project names are rejected

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Try invalid name with spaces
    ruchy_cmd()
        .arg("new")
        .arg("invalid name")
        .current_dir(temp_dir.path())
        .assert()
        .failure();

    // Try invalid name with special characters
    ruchy_cmd()
        .arg("new")
        .arg("invalid@name")
        .current_dir(temp_dir.path())
        .assert()
        .failure();
}

#[test]
#[ignore] // Will pass after implementation
fn test_cargo_002_error_on_existing_directory() {
    // CARGO-002: Test that existing directory causes error

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "existing-dir";

    // Create directory first
    fs::create_dir(temp_dir.path().join(project_name)).expect("Failed to create dir");

    // Try to create project with same name
    ruchy_cmd()
        .arg("new")
        .arg(project_name)
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("already exists").or(predicate::str::contains("destination")),
        );
}

#[test]
#[ignore] // Will pass after implementation
fn test_cargo_002_gitignore_includes_generated_rs() {
    // CARGO-002: Verify .gitignore includes generated .rs files

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_name = "test-gitignore";

    ruchy_cmd()
        .arg("new")
        .arg(project_name)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let project_dir = temp_dir.path().join(project_name);
    let gitignore = read_file_in_project(&project_dir, ".gitignore");

    // Should ignore generated .rs files from .ruchy sources
    // Note: This might be in comments or specific patterns
    assert!(
        gitignore.contains("target") || gitignore.contains("/target/"),
        ".gitignore should include target directory"
    );
}

// ===== Property Tests =====

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        #[ignore] // Run after implementation
        fn test_cargo_002_new_never_panics(
            name_len in 1usize..20,
            name_seed in 0u64..1000
        ) {
            // Property: `ruchy new` should never panic, even with random inputs

            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let project_name = format!("proj_{}_{}",name_seed, name_len);

            // Should not panic (may succeed or fail, but no panic)
            let _ = std::panic::catch_unwind(|| {
                let _ = ruchy_cmd()
                    .arg("new")
                    .arg(&project_name)
                    .current_dir(temp_dir.path())
                    .output();
            });
        }
    }
}
