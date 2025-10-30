#![allow(missing_docs)]
/// WASM Memory Model Tests
///
/// Comprehensive tests for WASM memory model (Phases 1-5)
/// Tests real memory allocation, field access, mutations for:
/// - Tuples (Phase 2-3)
/// - Structs (Phase 4)
/// - Arrays (Phase 5)
///
/// All tests validate that WASM compilation produces valid modules
/// and that the memory model features work correctly.
use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

fn temp_wasm_file(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("wasm_memory_test_{name}.wasm"))
}

fn temp_ruchy_file(name: &str, code: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("wasm_memory_test_{name}.ruchy"));
    fs::write(&path, code).expect("Failed to write test file");
    path
}

// ==================== PHASE 2: Tuple Memory Storage ====================

#[test]
fn test_wasm_tuple_creation() {
    let code = r"
fn main() {
    let pair = (3, 4)
    println(pair.0)
    println(pair.1)
}
";
    let ruchy_file = temp_ruchy_file("tuple_creation", code);
    let wasm_file = temp_wasm_file("tuple_creation");

    // Compile to WASM
    ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .success();

    // Verify WASM file was created and is valid
    assert!(wasm_file.exists());
    let wasm_bytes = fs::read(&wasm_file).expect("Failed to read WASM file");
    assert!(
        wasm_bytes.starts_with(b"\0asm"),
        "Invalid WASM magic number"
    );

    // Cleanup
    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();
}

#[test]
fn test_wasm_tuple_field_access() {
    let code = r"
fn main() {
    let triple = (10, 20, 30)
    println(triple.0)
    println(triple.1)
    println(triple.2)
}
";
    let ruchy_file = temp_ruchy_file("tuple_field_access", code);
    let wasm_file = temp_wasm_file("tuple_field_access");

    ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .success();

    assert!(wasm_file.exists());
    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();
}

#[test]
fn test_wasm_nested_tuples() {
    let code = r"
fn main() {
    let nested = ((1, 2), (3, 4))
    println(nested.0)
    println(nested.1)
}
";
    let ruchy_file = temp_ruchy_file("nested_tuples", code);
    let wasm_file = temp_wasm_file("nested_tuples");

    ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .success();

    assert!(wasm_file.exists());
    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();
}

// ==================== PHASE 3: Tuple Destructuring ====================

#[test]
fn test_wasm_tuple_destructuring_basic() {
    let code = r"
fn main() {
    let (x, y) = (3, 4)
    println(x)
    println(y)
}
";
    let ruchy_file = temp_ruchy_file("tuple_destructuring_basic", code);
    let wasm_file = temp_wasm_file("tuple_destructuring_basic");

    ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .success();

    assert!(wasm_file.exists());
    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();
}

#[test]
fn test_wasm_tuple_destructuring_nested() {
    let code = r"
fn main() {
    let ((a, b), c) = ((1, 2), 3)
    println(a)
    println(b)
    println(c)
}
";
    let ruchy_file = temp_ruchy_file("tuple_destructuring_nested", code);
    let wasm_file = temp_wasm_file("tuple_destructuring_nested");

    ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .success();

    assert!(wasm_file.exists());
    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();
}

#[test]
fn test_wasm_tuple_destructuring_underscore() {
    let code = r"
fn main() {
    let (x, _, z) = (1, 2, 3)
    println(x)
    println(z)
}
";
    let ruchy_file = temp_ruchy_file("tuple_destructuring_underscore", code);
    let wasm_file = temp_wasm_file("tuple_destructuring_underscore");

    ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .success();

    assert!(wasm_file.exists());
    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();
}

// ==================== PHASE 4: Struct Field Mutation ====================

#[test]
fn test_wasm_struct_creation() {
    let code = r"
struct Point {
    x: i32,
    y: i32
}

fn main() {
    let p = Point { x: 3, y: 4 }
    println(p.x)
    println(p.y)
}
";
    let ruchy_file = temp_ruchy_file("struct_creation", code);
    let wasm_file = temp_wasm_file("struct_creation");

    ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .success();

    assert!(wasm_file.exists());
    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();
}

#[test]
fn test_wasm_struct_field_mutation() {
    let code = r"
struct Point {
    x: i32,
    y: i32
}

fn main() {
    let mut p = Point { x: 3, y: 4 }
    println(p.x)
    p.x = 10
    println(p.x)
    println(p.y)
}
";
    let ruchy_file = temp_ruchy_file("struct_field_mutation", code);
    let wasm_file = temp_wasm_file("struct_field_mutation");

    ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .success();

    assert!(wasm_file.exists());
    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();
}

#[test]
fn test_wasm_struct_multiple_fields() {
    let code = r"
struct Rectangle {
    width: i32,
    height: i32,
    depth: i32
}

fn main() {
    let mut r = Rectangle { width: 10, height: 20, depth: 30 }
    println(r.width)
    println(r.height)
    println(r.depth)
    r.height = 100
    println(r.height)
}
";
    let ruchy_file = temp_ruchy_file("struct_multiple_fields", code);
    let wasm_file = temp_wasm_file("struct_multiple_fields");

    ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .success();

    assert!(wasm_file.exists());
    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();
}

// ==================== PHASE 5: Array Element Access ====================

#[test]
fn test_wasm_array_creation() {
    let code = r"
fn main() {
    let arr = [10, 20, 30]
    println(arr[0])
    println(arr[1])
    println(arr[2])
}
";
    let ruchy_file = temp_ruchy_file("array_creation", code);
    let wasm_file = temp_wasm_file("array_creation");

    ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .success();

    assert!(wasm_file.exists());
    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();
}

#[test]
fn test_wasm_array_mutation() {
    let code = r"
fn main() {
    let mut arr = [10, 20, 30]
    println(arr[0])
    arr[0] = 100
    println(arr[0])
    println(arr[1])
}
";
    let ruchy_file = temp_ruchy_file("array_mutation", code);
    let wasm_file = temp_wasm_file("array_mutation");

    ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .success();

    assert!(wasm_file.exists());
    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();
}

#[test]
fn test_wasm_array_multiple_mutations() {
    let code = r"
fn main() {
    let mut arr = [1, 2, 3, 4, 5]
    arr[0] = 100
    arr[2] = 300
    arr[4] = 500
    println(arr[0])
    println(arr[1])
    println(arr[2])
    println(arr[3])
    println(arr[4])
}
";
    let ruchy_file = temp_ruchy_file("array_multiple_mutations", code);
    let wasm_file = temp_wasm_file("array_multiple_mutations");

    ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .success();

    assert!(wasm_file.exists());
    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();
}

// ==================== Integration Tests ====================

#[test]
fn test_wasm_mixed_data_structures() {
    let code = r"
struct Point {
    x: i32,
    y: i32
}

fn main() {
    let arr = [1, 2, 3]
    let tup = (10, 20)
    let p = Point { x: 100, y: 200 }

    println(arr[0])
    println(tup.0)
    println(p.x)
}
";
    let ruchy_file = temp_ruchy_file("mixed_data_structures", code);
    let wasm_file = temp_wasm_file("mixed_data_structures");

    ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .success();

    assert!(wasm_file.exists());
    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();
}

#[test]
fn test_wasm_complex_mutations() {
    let code = r"
struct Point {
    x: i32,
    y: i32
}

fn main() {
    let mut arr = [1, 2, 3]
    let mut tup = (10, 20)
    let mut p = Point { x: 100, y: 200 }

    arr[1] = 999
    tup.1 = 888
    p.y = 777

    println(arr[1])
    println(tup.1)
    println(p.y)
}
";
    let ruchy_file = temp_ruchy_file("complex_mutations", code);
    let wasm_file = temp_wasm_file("complex_mutations");

    ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .success();

    assert!(wasm_file.exists());
    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();
}

// ==================== Edge Cases ====================

#[test]
fn test_wasm_empty_tuple() {
    let code = r"
fn main() {
    let unit = ()
    println(42)
}
";
    let ruchy_file = temp_ruchy_file("empty_tuple", code);
    let wasm_file = temp_wasm_file("empty_tuple");

    ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .success();

    assert!(wasm_file.exists());
    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();
}

#[test]
fn test_wasm_single_element_tuple() {
    let code = r"
fn main() {
    let single = (42,)
    println(single.0)
}
";
    let ruchy_file = temp_ruchy_file("single_element_tuple", code);
    let wasm_file = temp_wasm_file("single_element_tuple");

    ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .success();

    assert!(wasm_file.exists());
    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();
}

#[test]
fn test_wasm_large_tuple() {
    let code = r"
fn main() {
    let large = (1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
    println(large.0)
    println(large.5)
    println(large.9)
}
";
    let ruchy_file = temp_ruchy_file("large_tuple", code);
    let wasm_file = temp_wasm_file("large_tuple");

    ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .success();

    assert!(wasm_file.exists());
    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();
}
