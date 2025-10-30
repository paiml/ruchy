#![allow(missing_docs)]
//! Sprint 2 Phase 4: High Priority `ExprKind` Variants
//!
//! Ticket: [FMT-PERFECT-009]
//! Goal: Implement 15-20 high-priority remaining `ExprKind` variants
//! TDD: RED → GREEN → REFACTOR
//!
//! Priority Variants:
//! - Loop (infinite loop)
//! - Pipeline (|> operator)
//! - Reference (&, &mut)
//! - PreIncrement/PostIncrement (++x, x++)
//! - PreDecrement/PostDecrement (--x, x--)
//! - `ActorSend` (actor <- message)
//! - `ActorQuery` (actor <? query)
//! - Ask (ask expression)
//! - `ListComprehension` ([x for x in list])
//! - `DictComprehension` ({k: v for k, v in dict})
//! - `SetComprehension` ({x for x in set})
//! - `ImportAll` (import `module::`*)
//! - `ImportDefault` (import default from module)
//! - `ExportList` (export {a, b, c})
//! - `ExportDefault` (export default value)
//! - Command (shell command execution)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

fn setup_test_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp dir")
}

// ==================== Loop ====================

#[test]
fn test_fmt_loop_infinite() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("loop.ruchy");

    fs::write(&test_file, "loop { break }").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("loop"), "Should preserve loop keyword");
    assert!(formatted.contains("break"), "Should preserve break statement");
}

// ==================== Pipeline ====================

#[test]
fn test_fmt_pipeline_chain() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("pipeline.ruchy");

    fs::write(&test_file, "let result = 5 |> double |> triple").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("|>"), "Should preserve pipeline operator");
}

// ==================== Reference ====================

#[test]
fn test_fmt_reference_immutable() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("reference.ruchy");

    fs::write(&test_file, "let x = 42\nlet y = &x").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("&x"), "Should preserve immutable reference");
}

#[test]
#[ignore = "Parser doesn't support &mut syntax yet - needs PARSER enhancement"]
fn test_fmt_reference_mutable() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("reference_mut.ruchy");

    fs::write(&test_file, "let x = 42\nlet y = &mut x").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("&mut"), "Should preserve mutable reference");
}

// ==================== Increment/Decrement ====================

#[test]
fn test_fmt_pre_increment() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("pre_inc.ruchy");

    fs::write(&test_file, "let x = 5\n++x").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("++"), "Should preserve pre-increment");
}

#[test]
fn test_fmt_post_increment() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("post_inc.ruchy");

    fs::write(&test_file, "let x = 5\nx++").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("++"), "Should preserve post-increment");
}

#[test]
fn test_fmt_pre_decrement() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("pre_dec.ruchy");

    fs::write(&test_file, "let x = 5\n--x").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("--"), "Should preserve pre-decrement");
}

#[test]
fn test_fmt_post_decrement() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("post_dec.ruchy");

    fs::write(&test_file, "let x = 5\nx--").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("--"), "Should preserve post-decrement");
}

// ==================== Actor Operations ====================

#[test]
fn test_fmt_actor_send() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("actor_send.ruchy");

    fs::write(&test_file, "counter <- Increment").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("<-"), "Should preserve actor send operator");
}

#[test]
fn test_fmt_actor_query() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("actor_query.ruchy");

    fs::write(&test_file, "let value = counter <? GetCount").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("<?"), "Should preserve actor query operator");
}

#[test]
#[ignore = "Parser doesn't recognize ask keyword yet - needs PARSER enhancement"]
fn test_fmt_ask() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("ask.ruchy");

    fs::write(&test_file, "let result = ask counter GetCount").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("ask"), "Should preserve ask keyword");
}

// ==================== Comprehensions ====================

#[test]
fn test_fmt_list_comprehension() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("list_comp.ruchy");

    fs::write(&test_file, "let squares = [x * x for x in numbers]").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("for"), "Should preserve comprehension syntax");
    assert!(formatted.contains("in"), "Should preserve in keyword");
}

#[test]
#[ignore = "Parser doesn't support dict comprehension syntax yet - needs PARSER enhancement"]
fn test_fmt_dict_comprehension() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("dict_comp.ruchy");

    fs::write(&test_file, "let mapping = {k: v * 2 for k, v in pairs}").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("for"), "Should preserve comprehension syntax");
}

#[test]
fn test_fmt_set_comprehension() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("set_comp.ruchy");

    fs::write(&test_file, "let unique = {x for x in items}").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("for"), "Should preserve comprehension syntax");
}

// ==================== Import/Export Variants ====================

#[test]
#[ignore = "Parser doesn't support import module::* syntax yet - needs PARSER enhancement"]
fn test_fmt_import_all() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("import_all.ruchy");

    fs::write(&test_file, "import std::collections::*").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("import"), "Should preserve import keyword");
    assert!(formatted.contains('*'), "Should preserve wildcard import");
}

#[test]
#[ignore = "Parser doesn't support import default from syntax yet - needs PARSER enhancement"]
fn test_fmt_import_default() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("import_default.ruchy");

    fs::write(&test_file, "import default from std::collections").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("import"), "Should preserve import keyword");
    assert!(formatted.contains("default"), "Should preserve default keyword");
}

#[test]
fn test_fmt_export_list() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("export_list.ruchy");

    fs::write(&test_file, "export { add, subtract, multiply }").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("export"), "Should preserve export keyword");
}

#[test]
fn test_fmt_export_default() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("export_default.ruchy");

    fs::write(&test_file, "export default calculator").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains("export"), "Should preserve export keyword");
    assert!(formatted.contains("default"), "Should preserve default keyword");
}

// ==================== Command ====================

#[test]
#[ignore = "Parser doesn't support backtick command syntax yet - needs PARSER enhancement"]
fn test_fmt_command() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("command.ruchy");

    fs::write(&test_file, r"let output = `ls -la`").expect("Failed to write test file");

    ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .assert()
        .success();

    let formatted = fs::read_to_string(&test_file).expect("Failed to read formatted file");
    assert!(formatted.contains('`'), "Should preserve command backticks");
}
