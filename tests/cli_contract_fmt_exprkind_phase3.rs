// Sprint 2 Phase 3: ExprKind Coverage Tests
// RED phase: Failing tests for declarations, modules, and patterns

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// Priority 1: Type declarations
#[test]
fn test_fmt_struct_declaration() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("struct_decl.ruchy");

    let original = "struct Point { x: i32, y: i32 }";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("struct Point"),
        "Struct declaration lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Struct not implemented! Got:\n{formatted}"
    );
}

#[test]
fn test_fmt_enum_declaration() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("enum_decl.ruchy");

    let original = "enum Result { Ok(i32), Err(String) }";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("enum Result"),
        "Enum declaration lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Enum not implemented! Got:\n{formatted}"
    );
}

#[test]
fn test_fmt_trait_declaration() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("trait_decl.ruchy");

    let original = "trait Display { fn fmt(&self) -> String; }";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("trait Display"),
        "Trait declaration lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Trait not implemented! Got:\n{formatted}"
    );
}

#[test]
fn test_fmt_impl_block() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("impl_block.ruchy");

    let original = "impl Point { fn new(x: i32, y: i32) -> Point { Point { x, y } } }";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("impl Point"),
        "Impl block lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Impl not implemented! Got:\n{formatted}"
    );
}

// Priority 2: Module system
#[test]
fn test_fmt_module_declaration() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("module_decl.ruchy");

    let original = "mod utils { fn helper() {} }";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("mod utils"),
        "Module declaration lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Module not implemented! Got:\n{formatted}"
    );
}

#[test]
fn test_fmt_import_statement() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("import_stmt.ruchy");

    let original = "import std::collections::HashMap";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("import"),
        "Import statement lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Import not implemented! Got:\n{formatted}"
    );
}

#[test]
fn test_fmt_export_statement() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("export_stmt.ruchy");

    let original = "export fn public_api() {}";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("export"),
        "Export statement lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Export not implemented! Got:\n{formatted}"
    );
}

// Priority 3: Pattern matching extensions
#[test]
fn test_fmt_let_pattern() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("let_pattern.ruchy");

    let original = "let (x, y) = point in x + y";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("let") && formatted.contains("in"),
        "Let pattern lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "LetPattern not implemented! Got:\n{formatted}"
    );
}

#[test]
fn test_fmt_while_let() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("while_let.ruchy");

    let original = "while let Some(x) = iter.next() { process(x) }";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("while let"),
        "While let lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "WhileLet not implemented! Got:\n{formatted}"
    );
}

// Priority 4: String interpolation
#[test]
fn test_fmt_string_interpolation() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("string_interp.ruchy");

    let original = r#"let msg = f"Hello {name}""#;
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("f\"") || formatted.contains('{'),
        "String interpolation lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "StringInterpolation not implemented! Got:\n{formatted}"
    );
}

// Priority 5: Actor system
#[test]
fn test_fmt_actor_declaration() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("actor_decl.ruchy");

    let original = "actor Counter { count: i32 }";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("actor Counter"),
        "Actor declaration lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Actor not implemented! Got:\n{formatted}"
    );
}

#[test]
fn test_fmt_send_message() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("send_msg.ruchy");

    let original = "send(actor, message)";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("send"),
        "Send message lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Send not implemented! Got:\n{formatted}"
    );
}

// Priority 6: Additional type declarations
#[test]
fn test_fmt_tuple_struct() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("tuple_struct.ruchy");

    let original = "struct Color(u8, u8, u8)";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("struct Color") && formatted.contains('('),
        "Tuple struct lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "TupleStruct not implemented! Got:\n{formatted}"
    );
}

#[test]
fn test_fmt_class_declaration() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("class_decl.ruchy");

    let original = "class Animal { name: String }";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("class Animal"),
        "Class declaration lost! Got:\n{formatted}"
    );
    assert!(
        !formatted.contains("UNIMPLEMENTED"),
        "Class not implemented! Got:\n{formatted}"
    );
}
