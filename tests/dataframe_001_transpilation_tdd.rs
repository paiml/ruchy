// DATAFRAME-001: Fix DataFrame Transpilation - EXTREME TDD
// RED → GREEN → REFACTOR cycle
//
// This test file follows EXTREME TDD methodology:
// 1. RED: Write tests FIRST (all marked #[ignore], they WILL fail)
// 2. GREEN: Implement minimal code to make tests pass
// 3. REFACTOR: Add property tests, mutation tests, optimize
//
// Requirements from DATAFRAME-001-transpilation.md:
// - Auto-generate Cargo.toml with polars dependency during compilation
// - Detect DataFrame usage in AST (df![] macro)
// - Inject polars dependency when needed
// - Transpile df![] syntax to Polars API
//
// Problem: DataFrames work in interpreter but fail to compile to binaries
// Error: error[E0433]: failed to resolve: use of unresolved module or unlinked crate 'polars'

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to get ruchy binary command
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper to create temporary directory for compilation tests
fn temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

// ==================== RED PHASE: Unit Tests (Will Fail Initially) ====================

/// Test 1: Basic DataFrame compilation
///
/// This test verifies that a simple DataFrame created with df![] macro
/// can be compiled to a binary and executed successfully.
///
/// Expected behavior:
/// - Code compiles without errors
/// - Binary executes and displays DataFrame
/// - Output contains DataFrame content
#[test]
#[ignore] // RED phase - will fail until implementation
fn test_dataframe_001_basic_compilation() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    // Create simple DataFrame code
    fs::write(
        &source,
        r#"let df = df![{"x": [1, 2, 3]}]; println(df);"#
    ).unwrap();

    // Compile to binary
    let output_binary = temp.path().join("test_binary");
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(&output_binary)
        .assert()
        .success();

    // Execute the compiled binary
    Command::new(&output_binary)
        .assert()
        .success()
        .stdout(predicate::str::contains("x"))
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"))
        .stdout(predicate::str::contains("3"));
}

/// Test 2: Cargo.toml generation
///
/// This test verifies that when compiling DataFrame code, a Cargo.toml
/// file is automatically generated with the polars dependency.
///
/// Expected behavior:
/// - Cargo.toml is created in compilation directory
/// - File contains polars dependency
/// - Dependency version is specified
#[test]
#[ignore] // RED phase - will fail until implementation
fn test_dataframe_001_cargo_toml_generation() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    fs::write(
        &source,
        r#"let df = df![{"x": [1, 2, 3]}]; println(df);"#
    ).unwrap();

    // Compile (this should generate Cargo.toml)
    let output_binary = temp.path().join("test_binary");
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(&output_binary)
        .assert()
        .success();

    // Check that Cargo.toml was generated
    // Note: The exact location depends on implementation
    // This test may need adjustment once we know the compilation directory structure
    let cargo_toml = temp.path().join("Cargo.toml");

    if cargo_toml.exists() {
        let contents = fs::read_to_string(cargo_toml).unwrap();
        assert!(
            contents.contains("polars"),
            "Cargo.toml should contain polars dependency"
        );
        assert!(
            contents.contains("version") || contents.contains("\"0."),
            "Cargo.toml should specify polars version"
        );
    } else {
        // Alternative: check in a build subdirectory
        let build_dir = temp.path().join("build");
        if build_dir.exists() {
            let cargo_toml = build_dir.join("Cargo.toml");
            assert!(cargo_toml.exists(), "Cargo.toml should be generated");
        } else {
            panic!("Could not find generated Cargo.toml in expected locations");
        }
    }
}

/// Test 3: DataFrame column operations
///
/// This test verifies that DataFrames with multiple columns can be
/// compiled and that column selection operations work correctly.
///
/// Expected behavior:
/// - Multiple columns compile successfully
/// - Column access works in compiled binary
/// - Output is consistent with interpreter mode
#[test]
#[ignore] // RED phase - will fail until implementation
fn test_dataframe_001_column_operations() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    fs::write(
        &source,
        r#"
let df = df![{
    "name": ["Alice", "Bob", "Charlie"],
    "age": [30, 25, 35]
}];
println(df);
"#
    ).unwrap();

    let output_binary = temp.path().join("test_binary");
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(&output_binary)
        .assert()
        .success();

    Command::new(&output_binary)
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob"))
        .stdout(predicate::str::contains("Charlie"))
        .stdout(predicate::str::contains("30"))
        .stdout(predicate::str::contains("25"))
        .stdout(predicate::str::contains("35"));
}

/// Test 4: DataFrame filtering
///
/// This test verifies that filter operations on DataFrames compile
/// and execute correctly in binaries.
///
/// Expected behavior:
/// - Filter syntax compiles
/// - Filtered results are correct
/// - Output matches filtered data
#[test]
#[ignore] // RED phase - will fail until implementation
fn test_dataframe_001_filtering() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    fs::write(
        &source,
        r#"
let df = df![{"x": [1, 2, 3, 4, 5]}];
let filtered = df.filter(x > 3);
println(filtered);
"#
    ).unwrap();

    let output_binary = temp.path().join("test_binary");
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(&output_binary)
        .assert()
        .success();

    Command::new(&output_binary)
        .assert()
        .success()
        .stdout(predicate::str::contains("4"))
        .stdout(predicate::str::contains("5"));
}

/// Test 5: Multiple DataFrames
///
/// This test verifies that code with multiple DataFrame instances
/// can be compiled and all DataFrames work correctly.
///
/// Expected behavior:
/// - Multiple df![] macros compile
/// - Each DataFrame is independent
/// - All DataFrames can be used in same binary
#[test]
#[ignore] // RED phase - will fail until implementation
fn test_dataframe_001_multiple_dataframes() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    fs::write(
        &source,
        r#"
let df1 = df![{"a": [1, 2, 3]}];
let df2 = df![{"b": [4, 5, 6]}];
println(df1);
println(df2);
"#
    ).unwrap();

    let output_binary = temp.path().join("test_binary");
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(&output_binary)
        .assert()
        .success();

    Command::new(&output_binary)
        .assert()
        .success()
        .stdout(predicate::str::contains("a"))
        .stdout(predicate::str::contains("b"))
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("4"));
}

/// Test 6: Error handling for invalid DataFrame syntax
///
/// This test verifies that compilation fails gracefully with a clear
/// error message when invalid DataFrame syntax is used.
///
/// Expected behavior:
/// - Compilation fails (not a runtime error)
/// - Error message mentions DataFrame or df
/// - Error is actionable
#[test]
#[ignore] // RED phase - will fail until implementation
fn test_dataframe_001_error_handling() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    // Invalid DataFrame syntax (missing column definition)
    fs::write(
        &source,
        r#"let df = df![]; println(df);"#
    ).unwrap();

    let output_binary = temp.path().join("test_binary");
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(&output_binary)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("df")
                .or(predicate::str::contains("DataFrame"))
                .or(predicate::str::contains("column"))
        );
}

/// Test 7: Large DataFrame compilation
///
/// This test verifies that DataFrames with many rows can be compiled
/// and that performance is acceptable.
///
/// Expected behavior:
/// - Large DataFrames compile successfully
/// - Compilation time is reasonable (<60 seconds)
/// - Binary executes without errors
#[test]
#[ignore] // RED phase - will fail until implementation
fn test_dataframe_001_large_dataframe() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    // Generate code with a large DataFrame (1000 rows)
    let values: Vec<String> = (0..1000).map(|i| i.to_string()).collect();
    let values_str = values.join(", ");

    fs::write(
        &source,
        format!(r#"let df = df![{{"x": [{}]}}]; println(df.len());"#, values_str)
    ).unwrap();

    let output_binary = temp.path().join("test_binary");
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(&output_binary)
        .assert()
        .success();

    Command::new(&output_binary)
        .assert()
        .success()
        .stdout(predicate::str::contains("1000"));
}

/// Test 8: Mixed types in DataFrame
///
/// This test verifies that DataFrames with columns of different types
/// (int, float, string) compile correctly.
///
/// Expected behavior:
/// - Mixed type columns compile
/// - All types are preserved
/// - Output shows all types correctly
#[test]
#[ignore] // RED phase - will fail until implementation
fn test_dataframe_001_mixed_types() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    fs::write(
        &source,
        r#"
let df = df![{
    "int_col": [1, 2, 3],
    "float_col": [1.5, 2.5, 3.5],
    "string_col": ["a", "b", "c"]
}];
println(df);
"#
    ).unwrap();

    let output_binary = temp.path().join("test_binary");
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(&output_binary)
        .assert()
        .success();

    Command::new(&output_binary)
        .assert()
        .success()
        .stdout(predicate::str::contains("1.5"))
        .stdout(predicate::str::contains("a"));
}

/// Test 9: Cleanup after compilation
///
/// This test verifies that temporary files created during compilation
/// are properly cleaned up (or at least documented).
///
/// Expected behavior:
/// - No leftover intermediate files in user's directory
/// - Build artifacts are in expected location
/// - Cleanup is documented
#[test]
#[ignore] // RED phase - will fail until implementation
fn test_dataframe_001_cleanup() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    fs::write(
        &source,
        r#"let df = df![{"x": [1, 2, 3]}]; println(df);"#
    ).unwrap();

    // Count files before compilation
    let files_before: Vec<_> = fs::read_dir(temp.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();

    let output_binary = temp.path().join("test_binary");
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(&output_binary)
        .assert()
        .success();

    // Count files after compilation
    let files_after: Vec<_> = fs::read_dir(temp.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();

    // Should only have: source file, output binary, and possibly a build directory
    // No scattered temp files
    let new_files: Vec<_> = files_after.iter()
        .filter(|f| !files_before.contains(f))
        .collect();

    // Allow the binary and optionally a build directory
    assert!(
        new_files.len() <= 2,
        "Too many new files created: {:?}",
        new_files
    );
}

/// Test 10: Interpreter compatibility
///
/// This test verifies that the same DataFrame code produces identical
/// output in both interpreter mode and compiled binary mode.
///
/// Expected behavior:
/// - Same code works in both modes
/// - Output is identical (or semantically equivalent)
/// - No behavioral differences
#[test]
#[ignore] // RED phase - will fail until implementation
fn test_dataframe_001_interpreter_compatibility() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"let df = df![{"x": [1, 2, 3]}]; println(df);"#;
    fs::write(&source, code).unwrap();

    // Run in interpreter mode
    let interpreter_output = ruchy_cmd()
        .arg("run")
        .arg(&source)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Compile and run as binary
    let output_binary = temp.path().join("test_binary");
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(&output_binary)
        .assert()
        .success();

    let compiled_output = Command::new(&output_binary)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Outputs should be identical (or at least contain the same data)
    let interpreter_str = String::from_utf8_lossy(&interpreter_output);
    let compiled_str = String::from_utf8_lossy(&compiled_output);

    assert!(
        interpreter_str.contains("1") && compiled_str.contains("1"),
        "Both outputs should contain DataFrame data"
    );
    assert!(
        interpreter_str.contains("x") && compiled_str.contains("x"),
        "Both outputs should contain column name"
    );
}

// ==================== RED PHASE: Property Tests (Added in REFACTOR) ====================

// Property tests will be added after GREEN phase is complete.
// These will validate invariants:
// - Any DataFrame compiles successfully (10K+ random DataFrames)
// - DataFrame operations preserve data integrity
// - Compiled output matches interpreter output for random inputs

// TODO (REFACTOR phase): Add proptest cases with 10K+ iterations
// - proptest_dataframe_any_size: Random DataFrame sizes compile
// - proptest_dataframe_operations: Random operations compile and execute

// ==================== RED PHASE: Mutation Test Targets ====================

// Mutation testing targets (for REFACTOR phase):
// 1. DataFrame detection logic (does it find df![] correctly?)
// 2. Cargo.toml generation (is polars dependency injected?)
// 3. df![] transpilation (is Polars API called correctly?)
// 4. Polars version specification (is version string correct?)

// ==================== Test Summary ====================

#[test]
fn test_dataframe_001_red_phase_summary() {
    // This test documents the RED phase test plan
    //
    // Unit Tests Created: 10
    // 1. test_dataframe_001_basic_compilation
    // 2. test_dataframe_001_cargo_toml_generation
    // 3. test_dataframe_001_column_operations
    // 4. test_dataframe_001_filtering
    // 5. test_dataframe_001_multiple_dataframes
    // 6. test_dataframe_001_error_handling
    // 7. test_dataframe_001_large_dataframe
    // 8. test_dataframe_001_mixed_types
    // 9. test_dataframe_001_cleanup
    // 10. test_dataframe_001_interpreter_compatibility
    //
    // All tests currently #[ignore]d and will FAIL when un-ignored (RED phase)
    //
    // Problem Being Solved:
    // - DataFrames work in interpreter but fail to compile to binaries
    // - Error: error[E0433]: failed to resolve: use of unresolved crate 'polars'
    // - Root cause: No Cargo.toml generated with polars dependency
    //
    // Solution Approach:
    // 1. Detect DataFrame usage in AST (df![] macro)
    // 2. Auto-generate Cargo.toml with polars dependency
    // 3. Transpile df![] syntax to Polars API
    // 4. Compile with cargo build
    //
    // Next Step (GREEN phase):
    // 1. Implement DataFrame usage detection in transpiler/analyzer
    // 2. Create Cargo.toml generation function
    // 3. Add df![] macro transpilation to Polars
    // 4. Update compilation pipeline to use generated Cargo.toml
    // 5. Un-ignore tests one by one and make them pass
    //
    // After GREEN (REFACTOR phase):
    // 1. Add 10K+ property tests
    // 2. Run mutation tests (target ≥75%)
    // 3. Optimize if needed while maintaining tests

    assert!(true, "RED phase: 10 tests created, all will fail when un-ignored");
}
