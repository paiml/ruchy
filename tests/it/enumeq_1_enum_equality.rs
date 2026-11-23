#![allow(missing_docs)]

//! ENUMEQ-1: enum values compare structurally in the interpreter. Two enum
//! values are equal when they are the same enum, the same variant, and their
//! payloads are pairwise equal (recursively, with the same equality as every
//! other value). `!=` is the negation. The transpiled path derives
//! `PartialEq`, so interpreter output must equal rustc output.

use assert_cmd::Command;
use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::Interpreter;
use std::path::Path;
use std::time::Duration;

fn ruchy() -> Command {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.timeout(Duration::from_secs(30));
    cmd
}

/// Evaluate `src` as a program in a fresh interpreter and render the value.
fn eval(src: &str) -> String {
    let mut interp = Interpreter::new();
    let expr = Parser::new(src)
        .parse()
        .unwrap_or_else(|e| panic!("`{src}` did not parse: {e}"));
    let value = interp
        .eval_expr(&expr)
        .unwrap_or_else(|e| panic!("`{src}` failed: {e}"));
    value.to_string()
}

fn assert_true(src: &str) {
    assert_eq!(eval(src), "true", "`{src}` should be true");
}

fn assert_false(src: &str) {
    assert_eq!(eval(src), "false", "`{src}` should be false");
}

fn stdout_of(mut cmd: Command, what: &str) -> String {
    let out = cmd.output().expect("spawn command");
    assert!(
        out.status.success(),
        "{what} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

fn write_source(dir: &Path, src: &str) -> std::path::PathBuf {
    let file = dir.join("prog.ruchy");
    std::fs::write(&file, src).expect("write ruchy source");
    file
}

fn rustc_run(dir: &Path, rust: &str) -> String {
    let main_rs = dir.join("main.rs");
    let bin = dir.join("prog_bin");
    std::fs::write(&main_rs, rust).expect("write main.rs");
    let mut rustc = Command::new("rustc");
    rustc
        .args(["--edition", "2021", "-A", "warnings", "-o"])
        .arg(&bin)
        .arg(&main_rs)
        .timeout(Duration::from_secs(120));
    stdout_of(rustc, &format!("rustc on transpiled Rust:\n{rust}\n"));
    let mut run = Command::new(&bin);
    run.timeout(Duration::from_secs(30));
    stdout_of(run, "transpiled binary")
}

/// Interpret and transpile+rustc `src`; assert both print the same lines.
fn differential(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_source(dir.path(), src);
    let mut transpile = ruchy();
    transpile.arg("transpile").arg(&file);
    let rust = stdout_of(transpile, "ruchy transpile");
    let mut run = ruchy();
    run.arg("run").arg(&file);
    let interpreted = stdout_of(run, "ruchy run");
    let compiled = rustc_run(dir.path(), &rust);
    assert_eq!(
        interpreted, compiled,
        "interpreter disagrees with transpiled Rust\nsource:\n{src}\nrust:\n{rust}"
    );
    interpreted
}

#[test]
fn test_enumeq_1_01_option_some_equal() {
    assert_true("Some(3) == Some(3)");
    assert_false("Some(3) == Some(4)");
    assert_false("Some(3) != Some(3)");
    assert_true("Some(3) != Some(4)");
}

#[test]
fn test_enumeq_1_02_option_none_equal() {
    assert_true("None == None");
    assert_false("None != None");
    assert_false("Some(1) == None");
    assert_false("None == Some(1)");
    assert_true("Some(1) != None");
}

#[test]
fn test_enumeq_1_03_result_equal() {
    assert_true("Ok(1) == Ok(1)");
    assert_false("Ok(1) == Ok(2)");
    assert_true(r#"Err("e") == Err("e")"#);
    assert_false("Ok(1) == Err(1)");
    assert_true("Ok(1) != Err(1)");
}

#[test]
fn test_enumeq_1_04_bound_values_equal() {
    assert_true("{ let a = Some(3); a == Some(3) }");
    assert_true("{ let a = Some(3); let b = Some(3); a == b }");
    assert_false("{ let a = Some(3); let b = Some(4); a == b }");
}

#[test]
fn test_enumeq_1_05_nested_payloads_equal() {
    assert_true("Some(Some(2)) == Some(Some(2))");
    assert_false("Some(Some(2)) == Some(None)");
    assert_true("Some([1, 2]) == Some([1, 2])");
    assert_false("Some([1, 2]) == Some([2, 1])");
    assert_true(r#"Ok((1, "a")) == Ok((1, "a"))"#);
}

#[test]
fn test_enumeq_1_06_payload_uses_value_equality() {
    // The payload comparison is the same equality `1 == 1.0` uses.
    assert_eq!(eval("1 == 1.0"), eval("Some(1) == Some(1.0)"));
}

#[test]
fn test_enumeq_1_07_user_unit_variants() {
    assert_true("{ enum Color { Red, Green }\nColor::Red == Color::Red }");
    assert_false("{ enum Color { Red, Green }\nColor::Red == Color::Green }");
    assert_true("{ enum Color { Red, Green }\nColor::Red != Color::Green }");
}

#[test]
fn test_enumeq_1_08_user_tuple_variants() {
    let decl = "enum Shape { Circle(i64), Square(i64) }\n";
    assert_true(&format!("{{ {decl}Shape::Circle(2) == Shape::Circle(2) }}"));
    assert_false(&format!("{{ {decl}Shape::Circle(2) == Shape::Circle(3) }}"));
    assert_false(&format!("{{ {decl}Shape::Circle(2) == Shape::Square(2) }}"));
}

#[test]
fn test_enumeq_1_09_different_enums_same_variant_differ() {
    let decl = "enum A { X }\nenum B { X }\n";
    assert_false(&format!("{{ {decl}A::X == B::X }}"));
}

#[test]
fn test_enumeq_1_10_assert_eq_accepts_equal_enums() {
    let out = ruchy()
        .arg("-e")
        .arg(r#"assert_eq(Some(3), Some(3)); assert_eq(None, None); assert_eq(Ok(1), Ok(1)); println("ok")"#)
        .output()
        .expect("spawn ruchy");
    assert!(
        out.status.success(),
        "assert_eq on equal enums failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "ok\n");
}

#[test]
fn test_enumeq_1_11_assert_eq_rejects_unequal_enums() {
    ruchy()
        .arg("-e")
        .arg("assert_eq(Some(3), Some(4))")
        .assert()
        .failure();
}

#[test]
fn test_enumeq_1_12_differential_matches_rustc() {
    let src = r#"
enum Color { Red, Green }
enum Shape { Circle(i64), Square(i64) }
fun main() {
    let a = Some(3)
    let n: Option<i64> = None
    let r: Result<i64, String> = Ok(1)
    println(a == Some(3))
    println(a == Some(4))
    println(a != Some(3))
    println(n == None)
    println(r == Ok(1))
    println(Color::Red == Color::Red)
    println(Color::Red == Color::Green)
    println(Color::Red != Color::Green)
    println(Shape::Circle(2) == Shape::Circle(2))
    println(Shape::Circle(2) == Shape::Circle(3))
    println(Shape::Circle(2) == Shape::Square(2))
}
"#;
    let out = differential(src);
    assert_eq!(
        out,
        "true\nfalse\nfalse\ntrue\ntrue\ntrue\nfalse\ntrue\ntrue\nfalse\nfalse\n"
    );
}
