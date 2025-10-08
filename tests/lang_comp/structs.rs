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
    ruchy_cmd()
        .write_stdin("struct Point { x: i32, y: i32 }; fn main() { let p = Point { x: 10, y: 20 }; println(p.x) }")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
}

#[test]
fn test_langcomp_014_01_struct_field_access() {
    ruchy_cmd()
        .write_stdin("struct Point { x: i32, y: i32 }; fn main() { let p = Point { x: 10, y: 20 }; println(p.y) }")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("20"));
}

#[test]
fn test_langcomp_014_01_nested_struct_field_access() {
    ruchy_cmd()
        .write_stdin("struct Point { x: i32 }; struct Rectangle { top_left: Point }; fn main() { let rect = Rectangle { top_left: Point { x: 10 } }; println(rect.top_left.x) }")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
}

// ==================== LANG-COMP-014-02: Struct Methods ====================

#[test]
fn test_langcomp_014_02_struct_methods_example_file() {
    let example = example_path("02_struct_methods.ruchy");
    ruchy_cmd().arg("run").arg(&example).assert().success();
}

#[test]
fn test_langcomp_014_02_struct_method_call() {
    ruchy_cmd()
        .write_stdin("struct Rectangle { width: i32, height: i32 }; impl Rectangle { fn area(self) -> i32 { self.width * self.height } }; fn main() { let r = Rectangle { width: 10, height: 20 }; let a = r.area(); println(a) }")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("200"));
}

// ==================== LANG-COMP-014-03: Tuple Structs ====================

#[test]
fn test_langcomp_014_03_tuple_structs_example_file() {
    let example = example_path("03_tuple_structs.ruchy");
    ruchy_cmd().arg("run").arg(&example).assert().success();
}

#[test]
fn test_langcomp_014_03_tuple_struct_creation() {
    ruchy_cmd()
        .write_stdin("struct Point(i32, i32); fn main() { let p = Point(10, 20); println(p.0) }")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
}

#[test]
fn test_langcomp_014_03_tuple_struct_field_access() {
    ruchy_cmd()
        .write_stdin("struct Point(i32, i32); fn main() { let p = Point(10, 20); println(p.1) }")
        .arg("repl")
        .assert()
        .success()
        .stdout(predicate::str::contains("20"));
}
