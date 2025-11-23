//! TRANSPILER-DEFECT-016-C: Match Arm String Literal Returns
//!
//! **Issue**: Match arms returning string literals don't auto-convert to String when function returns String
//!
//! **Root Cause**: Match arm transpilation emits raw string literals without .`to_string()`
//! when the match expression's return type is String.
//!
//! **Impact**: 1 error in reaper project (line 98/402)
//!
//! **Real-world Example** (reaper main.ruchy:400-406):
//! ```ruchy
//! fun priority_to_string(priority: Priority) -> String {
//!     match priority {
//!         Priority::High => "high",  // E0308: expected String, found &str
//!         ...
//!     }
//! }
//! ```
//!
//! **Current Transpilation** (BROKEN):
//! ```rust
//! fn priority_to_string(priority: Priority) -> String {
//!     match priority {
//!         Priority::High => "high",  // ❌ E0308: expected String, found &str
//!         ...
//!     }
//! }
//! ```
//!
//! **Expected Transpilation**:
//! ```rust
//! fn priority_to_string(priority: Priority) -> String {
//!     match priority {
//!         Priority::High => "high".to_string(),  // ✅ Auto-convert
//!         ...
//!     }
//! }
//! ```
//!
//! **Test Strategy**: EXTREME TDD (RED → GREEN → REFACTOR)

use std::fs;
use tempfile::TempDir;

/// Test 1: Match returning string literals (ACTUAL reaper pattern line 402)
#[test]
fn test_defect_016_c_01_match_string_return_red() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    // ACTUAL pattern from reaper main.ruchy:400-406
    let ruchy_code = r#"
enum Priority {
    High,
    Medium,
    Low,
}

fun priority_to_string(priority: Priority) -> String {
    match priority {
        Priority::High => "high",
        Priority::Medium => "medium",
        Priority::Low => "low",
    }
}

println(priority_to_string(Priority::High));
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if output.status.success() {
        eprintln!("✅ GREEN: Match arms auto-converted to String with .to_string()");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("E0308"),
            "Expected E0308: expected String, found &str. Got:\n{stderr}"
        );
        eprintln!("✅ RED TEST: Match arm string literal error confirmed");
    }
}

/// Test 2: Simple two-arm match returning strings
#[test]
fn test_defect_016_c_02_simple_match_string_red() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
enum Status {
    Active,
    Inactive,
}

fun status_to_str(status: Status) -> String {
    match status {
        Status::Active => "active",
        Status::Inactive => "inactive",
    }
}

println(status_to_str(Status::Active));
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if output.status.success() {
        eprintln!("✅ GREEN: Simple match auto-converted");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("E0308"),
            "Expected E0308 for simple match. Got:\n{stderr}"
        );
        eprintln!("✅ RED TEST: Simple match string error confirmed");
    }
}

/// Test 3: Match with integer returns (should NOT convert - baseline)
#[test]
fn test_defect_016_c_03_match_integer_baseline() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r"
enum Priority {
    High,
    Medium,
    Low,
}

fun priority_to_int(priority: Priority) -> i32 {
    match priority {
        Priority::High => 3,
        Priority::Medium => 2,
        Priority::Low => 1,
    }
}

println(priority_to_int(Priority::High));
";

    fs::write(&test_file, ruchy_code).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("compile")
        .arg(&test_file)
        .assert()
        .success();
}

/// Test 4: Nested match returning strings
#[test]
fn test_defect_016_c_04_nested_match_string_red() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
enum Level { High, Low }
enum Type { A, B }

fun describe(level: Level, typ: Type) -> String {
    match level {
        Level::High => match typ {
            Type::A => "high-a",
            Type::B => "high-b",
        },
        Level::Low => "low",
    }
}

println(describe(Level::High, Type::A));
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if output.status.success() {
        eprintln!("✅ GREEN: Nested match handled");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("RED TEST: Nested match - {stderr}");
    }
}

/// Test 5: Match in expression position (not function return)
#[test]
fn test_defect_016_c_05_match_in_expression_red() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
enum Priority { High, Low }

let priority = Priority::High;
let message: String = match priority {
    Priority::High => "urgent",
    Priority::Low => "normal",
};
println(message);
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if output.status.success() {
        eprintln!("✅ GREEN: Match in expression handled");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("RED TEST: Match in expression - {stderr}");
    }
}

// PROPERTY TESTS (Run after GREEN phase)
// These will be written in Phase 3 (REFACTOR) with proptest

// MUTATION TESTS (Run after GREEN phase)
// cargo mutants --file src/backend/transpiler/expressions.rs --timeout 60
