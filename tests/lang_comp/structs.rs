#![allow(deprecated)]
// LANG-COMP-014: Structs - Validation Tests with Traceability
// Links to: examples/lang_comp/14-structs/*.ruchy
// Validates: LANG-COMP-014 Structs (definition, fields, methods, tuple structs)
// EXTREME TDD Protocol: Tests use assert_cmd + mandatory naming convention

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

/// Helper to get ruchy binary command
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper to get example file path
fn example_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/lang_comp/14-structs")
        .join(relative_path)
}

// ==================== LANG-COMP-014-01: Basic Structs ====================

#[test]
fn test_langcomp_014_01_basic_structs_example_file() {
    let example = example_path("01_basic_structs.ruchy");
    ruchy_cmd().arg("run").arg(&example).assert().success();
}

#[test]
fn test_langcomp_014_01_struct_definition_and_creation() {
    let temp_file = std::env::temp_dir().join("langcomp_014_01_struct_def.ruchy");
    std::fs::write(&temp_file, "struct Point { x: i32, y: i32 }\nlet p = Point { x: 10, y: 20 }\nprintln(p.x)").unwrap();
    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_014_01_struct_field_access() {
    let temp_file = std::env::temp_dir().join("langcomp_014_01_struct_access.ruchy");
    std::fs::write(&temp_file, "struct Point { x: i32, y: i32 }\nlet p = Point { x: 10, y: 20 }\nprintln(p.y)").unwrap();
    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("20"));
    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_langcomp_014_01_nested_struct_field_access() {
    let temp_file = std::env::temp_dir().join("langcomp_014_01_nested_struct.ruchy");
    std::fs::write(&temp_file, "struct Point { x: i32 }\nstruct Rectangle { top_left: Point }\nlet rect = Rectangle { top_left: Point { x: 10 } }\nprintln(rect.top_left.x)").unwrap();
    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
    std::fs::remove_file(&temp_file).ok();
}

// ==================== LANG-COMP-014-02: Struct Methods ====================
// STATUS: PARTIALLY IMPLEMENTED - impl blocks parse but runtime method dispatch incomplete
// ROADMAP: RUNTIME-093 (Struct Method Dispatch)

#[test]
#[ignore = "RUNTIME-093: Struct methods via impl blocks not fully implemented in runtime"]
fn test_langcomp_014_02_struct_methods_example_file() {
    let example = example_path("02_struct_methods.ruchy");
    ruchy_cmd().arg("run").arg(&example).assert().success();
}

#[test]
#[ignore = "RUNTIME-093: Struct methods via impl blocks not fully implemented in runtime"]
fn test_langcomp_014_02_struct_method_call() {
    let temp_file = std::env::temp_dir().join("langcomp_014_02_struct_method.ruchy");
    std::fs::write(&temp_file, "struct Rectangle { width: i32, height: i32 }\nimpl Rectangle { fn area(self) -> i32 { self.width * self.height } }\nlet r = Rectangle { width: 10, height: 20 }\nlet a = r.area()\nprintln(a)").unwrap();
    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("200"));
    std::fs::remove_file(&temp_file).ok();
}

// ==================== LANG-COMP-014-03: Tuple Structs ====================
// STATUS: NOT IMPLEMENTED - Tuple struct numeric field access (p.0, p.1) not in runtime
// ROADMAP: LANG-COMP-STATUS-AUDIT notes this as not implemented

#[test]
#[ignore = "TUPLE-STRUCT: Numeric field access (p.0, p.1) not implemented in runtime"]
fn test_langcomp_014_03_tuple_structs_example_file() {
    let example = example_path("03_tuple_structs.ruchy");
    ruchy_cmd().arg("run").arg(&example).assert().success();
}

#[test]
#[ignore = "TUPLE-STRUCT: Numeric field access (p.0, p.1) not implemented in runtime"]
fn test_langcomp_014_03_tuple_struct_creation() {
    let temp_file = std::env::temp_dir().join("langcomp_014_03_tuple_struct.ruchy");
    std::fs::write(&temp_file, "struct Point(i32, i32)\nlet p = Point(10, 20)\nprintln(p.0)").unwrap();
    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
    std::fs::remove_file(&temp_file).ok();
}

#[test]
#[ignore = "TUPLE-STRUCT: Numeric field access (p.0, p.1) not implemented in runtime"]
fn test_langcomp_014_03_tuple_struct_field_access() {
    let temp_file = std::env::temp_dir().join("langcomp_014_03_tuple_access.ruchy");
    std::fs::write(&temp_file, "struct Point(i32, i32)\nlet p = Point(10, 20)\nprintln(p.1)").unwrap();
    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("20"));
    std::fs::remove_file(&temp_file).ok();
}
