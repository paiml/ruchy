#![allow(missing_docs)]
//! RHLGA-1 (G2b S3): parser/transpiler defects checked end to end.
//!
//! Each program is run by the interpreter (`ruchy run`) and transpiled,
//! compiled with rustc and executed; both must print the expected output.
//!
//! - PARENCALL-1: `(obj.f)(3)` calls a closure stored in a field. The parser
//!   gave FieldAccess nodes a 0..0 span, so the same-line check for a postfix
//!   call failed and `(h.f)(3)` was split into `h.f; 3`.
//! - NESTEDMOD-1: `pub mod b { .. }` inside `mod a { .. }` and `pub struct`
//!   inside a module.

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::frontend::ast::{Expr, ExprKind};
use ruchy::{Parser, Transpiler};
use std::process::Command;

fn scratch_dir(name: &str) -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix(&format!("g2b_s3_{name}_"))
        .tempdir()
        .expect("scratch dir")
}

fn compile_and_run(rust: &str, name: &str) -> String {
    let dir = scratch_dir(name);
    let src = dir.path().join("main.rs");
    let bin = dir.path().join("main");
    std::fs::write(&src, rust).expect("write rust");
    let out = Command::new("rustc")
        .args(["--edition", "2021", "-A", "warnings", "-o"])
        .arg(&bin)
        .arg(&src)
        .output()
        .expect("rustc runs");
    assert!(
        out.status.success(),
        "rustc rejected the transpiled program:\n{}\n--- rust ---\n{rust}",
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&bin).output().expect("binary runs");
    assert!(run.status.success(), "binary failed: {run:?}");
    String::from_utf8_lossy(&run.stdout).into_owned()
}

fn ruchy_run(code: &str, name: &str) -> String {
    let dir = scratch_dir(name);
    let file = dir.path().join("prog.ruchy");
    std::fs::write(&file, code).expect("write ruchy");
    let out = Command::new(env!("CARGO_BIN_EXE_ruchy"))
        .arg("run")
        .arg(&file)
        .output()
        .expect("ruchy runs");
    assert!(out.status.success(), "ruchy run failed: {out:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn parse(code: &str) -> Expr {
    Parser::new(code).parse().expect("parses")
}

/// Interpreter and transpiled binary both print `expected`.
fn differential(code: &str, name: &str, expected: &str) -> String {
    let rust = Transpiler::new()
        .transpile_to_program(&parse(code))
        .expect("transpiles")
        .to_string();
    assert_eq!(ruchy_run(code, name).trim(), expected, "interpreter");
    assert_eq!(compile_and_run(&rust, name).trim(), expected, "{rust}");
    rust
}

// ===== PARENCALL-1 =====

const PARENCALL_IN_MAIN: &str = "struct H { f: fn(i32) -> i32 }\n\
fun main() {\n    let h = H { f: |x| x * 2 }\n    let r = (h.f)(3)\n    println(r)\n}\n";

#[test]
fn test_parencall_1_01_field_closure_call_in_block() {
    differential(PARENCALL_IN_MAIN, "parencall_block", "6");
}

#[test]
fn test_parencall_1_02_field_closure_call_as_argument() {
    let code = "struct H { f: fn(i32) -> i32 }\n\
fun main() {\n    let h = H { f: |x| x * 2 }\n    println((h.f)(3))\n}\n";
    differential(code, "parencall_arg", "6");
}

#[test]
fn test_parencall_1_03_top_level_after_let() {
    let code = "let h = { f: |x| x * 2 }\nprintln((h.f)(3))\n";
    assert_eq!(ruchy_run(code, "parencall_top").trim(), "6");
}

/// The parsed `let r = (h.f)(3)` binds a Call whose callee is the field.
#[test]
fn test_parencall_1_04_ast_is_a_call_on_the_field() {
    let ast = parse("let h = 1\nlet r = (h.f)(3)\nr");
    let text = format!("{ast:?}");
    assert!(
        text.contains("Call { func: Expr { kind: FieldAccess"),
        "not a call on the field: {text}"
    );
}

/// A field access carries the source span of `receiver.field`.
#[test]
fn test_parencall_1_05_field_access_span_covers_source() {
    let src = "obj.field";
    let ast = parse(src);
    assert!(matches!(ast.kind, ExprKind::FieldAccess { .. }), "{ast:?}");
    assert_eq!((ast.span.start, ast.span.end), (0, src.len()));
    let tuple = parse("t.0");
    assert_eq!((tuple.span.start, tuple.span.end), (0, 3), "{tuple:?}");
}

/// A parenthesised field on one line and a parenthesised expression on the
/// next stay two statements: the postfix call still requires the same line.
#[test]
fn test_parencall_1_06_paren_on_next_line_is_not_a_call() {
    let ast = parse("let h = 1\n(h.f)\n(3)");
    let text = format!("{ast:?}");
    assert!(
        !text.contains("Call {"),
        "split lines became a call: {text}"
    );
}

// ===== NESTEDMOD-1 =====

const NESTED_MOD: &str = "mod a {\n    pub fun g() -> i32 { 1 }\n    pub mod b {\n        \
pub fun f() -> i32 { 41 }\n    }\n}\n\
fun main() {\n    println(a::b::f() + a::g())\n}\n";

#[test]
fn test_nestedmod_1_01_nested_pub_mod_path_call() {
    let rust = differential(NESTED_MOD, "nested_mod", "42");
    assert!(rust.contains("pub mod b"), "{rust}");
}

/// `pub struct` inside a module parses (it was refused: "'pub' can only be
/// used with function declarations ..."), and the transpiled module keeps the
/// struct and its `pub` field public. The interpreter run avoids constructing
/// the struct inside the module (see the receipt: runtime struct lookup).
#[test]
fn test_nestedmod_1_02_pub_struct_inside_mod() {
    let code = "mod shapes {\n    pub struct P { pub x: i32 }\n    \
pub fun make() -> P { P { x: 5 } }\n}\n\
fun main() {\n    let p = shapes::make()\n    println(p.x)\n}\n";
    let rust = Transpiler::new()
        .transpile_to_program(&parse(code))
        .expect("transpiles")
        .to_string();
    assert!(rust.contains("pub struct P"), "{rust}");
    assert_eq!(compile_and_run(&rust, "pub_struct").trim(), "5", "{rust}");
    let interp = "mod shapes {\n    pub struct P { pub x: i32 }\n    \
pub fun five() -> i32 { 5 }\n}\n\
fun main() {\n    println(shapes::five())\n}\n";
    assert_eq!(ruchy_run(interp, "pub_struct_run").trim(), "5");
}

#[test]
fn test_nestedmod_1_03_private_nested_mod_used_inside_parent() {
    let code = "mod a {\n    mod inner {\n        pub fun v() -> i32 { 7 }\n    }\n    \
pub fun g() -> i32 { inner::v() * 2 }\n}\n\
fun main() {\n    println(a::g())\n}\n";
    let rust = differential(code, "private_nested", "14");
    assert!(!rust.contains("pub mod inner"), "{rust}");
}

#[test]
fn test_nestedmod_1_04_three_levels() {
    let code = "mod a {\n    pub mod b {\n        pub mod c {\n            \
pub fun f() -> i32 { 3 }\n        }\n    }\n}\n\
fun main() {\n    println(a::b::c::f())\n}\n";
    differential(code, "three_levels", "3");
}
