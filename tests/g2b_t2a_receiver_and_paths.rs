//! G2B-T5 (RHLGA-1): a `::` callee is a path, a `.` callee is a field.
//!
//! The parser records `a::b` as a FieldAccess marked with `Expr::is_path_access`,
//! so the transpiler emits `a::b(..)` for a path callee and `(obj.f)(..)` for a
//! field callee (a closure/fn pointer stored in a struct field). Each program is
//! transpiled, compiled with rustc and run; its output is compared with
//! `ruchy run` where the interpreter accepts the program.

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::frontend::ast::{Expr, ExprKind};
use ruchy::{Parser, Transpiler};
use std::path::Path;
use std::process::Command;

/// A scratch directory for one program, removed when the returned guard drops.
fn scratch_dir(name: &str) -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix(&format!("g2b_t2a_{name}_"))
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

fn transpile(ast: &Expr) -> String {
    Transpiler::new()
        .transpile_to_program(ast)
        .expect("transpiles")
        .to_string()
}

fn ruchy_run(code: &str, name: &str) -> String {
    let dir = scratch_dir(name);
    let file = dir.path().join("prog.ruchy");
    std::fs::write(&file, code).expect("write ruchy");
    let out = Command::new(env!("CARGO_BIN_EXE_ruchy"))
        .arg("run")
        .arg(Path::new(&file))
        .output()
        .expect("ruchy runs");
    assert!(out.status.success(), "ruchy run failed: {out:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Transpile, compile, run, and compare with the interpreter.
fn differential(code: &str, name: &str, expected: &str) -> String {
    let ast = Parser::new(code).parse().expect("parses");
    let rust = transpile(&ast);
    assert_eq!(compile_and_run(&rust, name).trim(), expected, "{rust}");
    assert_eq!(ruchy_run(code, name).trim(), expected);
    rust
}

#[test]
fn test_g2b_t5_01_module_path_call() {
    let code = r#"
mod helpers {
    pub fun double(x: i32) -> i32 { x * 2 }
}
fun main() {
    println!("{}", helpers::double(4))
}
"#;
    let rust = differential(code, "module", "8");
    assert!(rust.contains("helpers :: double"), "{rust}");
}

/// A three-segment path `std::cmp::max(..)`, compiled with rustc. The
/// interpreter has no `std::cmp` (`Object has no field named 'cmp'`) and a
/// nested `pub mod`/`pub struct` inside a `mod` does not parse or transpile, so
/// this case is transpiler-only.
#[test]
fn test_g2b_t5_02_nested_path_call() {
    let code = r#"
fun main() {
    println!("{}", std::cmp::max(3, 7))
}
"#;
    let ast = Parser::new(code).parse().expect("parses");
    let rust = transpile(&ast);
    assert!(rust.contains("std :: cmp :: max (3"), "{rust}");
    assert_eq!(compile_and_run(&rust, "nested").trim(), "7", "{rust}");
}

#[test]
fn test_g2b_t5_03_associated_function_call() {
    let code = r#"
struct Point { x: i32, y: i32 }
impl Point {
    pub fun new(x: i32, y: i32) -> Point { Point { x: x, y: y } }
}
fun main() {
    let p = Point::new(1, 2)
    println!("{}", p.x + p.y)
}
"#;
    let rust = differential(code, "assoc", "3");
    assert!(rust.contains("Point :: new"), "{rust}");
}

/// Replace the callee `probe` of every Call with `object.field` (a `.` access).
fn retarget_probe(expr: &mut Expr, object: &str, field: &str) {
    if let ExprKind::Call { func, .. } = &mut expr.kind {
        if matches!(&func.kind, ExprKind::Identifier(n) if n == "probe") {
            let obj = Expr::new(ExprKind::Identifier(object.to_string()), func.span);
            let access = ExprKind::FieldAccess {
                object: Box::new(obj),
                field: field.to_string(),
            };
            **func = Expr::new(access, func.span);
        }
    }
    match &mut expr.kind {
        ExprKind::Block(items) => items
            .iter_mut()
            .for_each(|e| retarget_probe(e, object, field)),
        ExprKind::Function { body, .. } => retarget_probe(body, object, field),
        _ => {}
    }
}

/// A field holding a fn pointer, called through `(h.f)(3)`. The callee is a
/// `.` FieldAccess (not a path), so Rust needs `(h.f)(3)`, not `h::f(3)`.
/// The AST is built from source with a `probe` callee retargeted to `h.f`
/// because the parser splits `(h.f)(3)` inside a block (FieldAccess span 0..0).
#[test]
fn test_g2b_t5_04_field_callee_is_not_a_path() {
    let code = r#"
struct Holder { f: fn(i32) -> i32 }
fun double(x: i32) -> i32 { x * 2 }
fun apply(h: Holder) -> i32 {
    probe(3)
}
fun main() {
    let h = Holder { f: double }
    println!("{}", apply(h))
}
"#;
    let mut ast = Parser::new(code).parse().expect("parses");
    retarget_probe(&mut ast, "h", "f");
    let rust = transpile(&ast);
    assert!(
        !rust.contains("h :: f"),
        "field callee emitted as a path: {rust}"
    );
    assert_eq!(compile_and_run(&rust, "field").trim(), "6", "{rust}");
}

/// The parser marks `a::b` (path) and leaves `a.b` (field) unmarked.
#[test]
fn test_g2b_t5_05_parser_marks_path_access() {
    let path = Parser::new("helpers::double").parse().expect("parses");
    assert!(path.is_path_access(), "{path:?}");
    let field = Parser::new("helpers.double").parse().expect("parses");
    assert!(!field.is_path_access(), "{field:?}");
}
