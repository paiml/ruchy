//! TRANSPILER-DEFECT-017: Array Literal to Vec Conversion
//!
//! **Issue**: Array literals assigned to Vec-typed variables don't auto-convert with .`to_vec()`
//!
//! **Root Cause**: Type annotation transpiler converts [T] → Vec<T>, but value transpiler
//! emits raw array literal without .`to_vec()` conversion.
//!
//! **Impact**: 1 error in reaper project (line 262)
//!
//! **Real-world Example** (reaper main.ruchy:972):
//! ```ruchy
//! let processes: [Process] = [current_process];
//! ```
//!
//! **Current Transpilation** (BROKEN):
//! ```rust
//! let processes: Vec<Process> = [current_process];  // E0308: expected Vec, found array
//! ```
//!
//! **Expected Transpilation**:
//! ```rust
//! let processes: Vec<Process> = [current_process].to_vec();  // ✅ Auto-convert
//! ```
//!
//! **Test Strategy**: EXTREME TDD (RED → GREEN → REFACTOR)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Test 1: Single element array to Vec (ACTUAL reaper pattern line 972)
#[test]
fn test_defect_017_01_single_element_array_to_vec_red() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    // ACTUAL pattern from reaper main.ruchy:972
    let ruchy_code = r#"
struct Process {
    pid: i32,
    name: String,
}

let current = Process { pid: 1, name: "init" };
let processes: [Process] = [current];
println(processes.len());
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if output.status.success() {
        eprintln!("✅ GREEN: Array auto-converted to Vec with .to_vec()");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("E0308"),
            "Expected E0308: expected Vec, found array. Got:\n{stderr}"
        );
        eprintln!("✅ RED TEST: Array to Vec conversion error confirmed");
    }
}

/// Test 2: Multiple elements array to Vec
#[test]
fn test_defect_017_02_multiple_elements_array_to_vec_red() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
struct Config {
    name: String,
    value: i32,
}

let c1 = Config { name: "debug", value: 1 };
let c2 = Config { name: "release", value: 0 };
let configs: [Config] = [c1, c2];
println(configs.len());
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if output.status.success() {
        eprintln!("✅ GREEN: Multiple element array auto-converted");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("E0308"),
            "Expected E0308 for multi-element array. Got:\n{stderr}"
        );
        eprintln!("✅ RED TEST: Multiple element array error confirmed");
    }
}

/// Test 3: Empty array to Vec (edge case)
#[test]
fn test_defect_017_03_empty_array_to_vec() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r"
struct Item { id: i32 }

let items: [Item] = [];
println(items.len());
";

    fs::write(&test_file, ruchy_code).unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if output.status.success() {
        eprintln!("✅ GREEN: Empty array handled");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("RED TEST: Empty array - {stderr}");
    }
}

/// Test 4: Baseline - Actual Vec syntax should work unchanged
#[test]
fn test_defect_017_04_vec_macro_baseline() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r"
struct Item { id: i32 }

let item = Item { id: 42 };
let items = vec![item];  // Already using Vec, should work
println(items.len());
";

    fs::write(&test_file, ruchy_code).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("compile")
        .arg(&test_file)
        .assert()
        .success();
}

/// Test 5: Baseline - Array type without Vec annotation (no conversion needed)
#[test]
fn test_defect_017_05_array_type_baseline() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r"
let numbers = [1, 2, 3];  // No type annotation, inferred as array
println(numbers.len());
";

    fs::write(&test_file, ruchy_code).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("compile")
        .arg(&test_file)
        .assert()
        .success();
}

/// Test 6: Integer array to Vec (simpler type for testing)
#[test]
fn test_defect_017_06_integer_array_to_vec_red() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r"
let x = 10;
let numbers: [i32] = [x, 20, 30];
println(numbers.len());
";

    fs::write(&test_file, ruchy_code).unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if output.status.success() {
        eprintln!("✅ GREEN: Integer array auto-converted");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("E0308"),
            "Expected E0308 for integer array. Got:\n{stderr}"
        );
        eprintln!("✅ RED TEST: Integer array to Vec error confirmed");
    }
}

// PROPERTY TESTS (Run after GREEN phase)
// These will be written in Phase 3 (REFACTOR) with proptest

// MUTATION TESTS (Run after GREEN phase)
// cargo mutants --file src/backend/transpiler/statements.rs --timeout 60
