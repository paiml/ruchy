#![allow(missing_docs)]

//! QPAT-1: qualified variant patterns (`Option::Some(v)`, `Option::None`,
//! `Result::Ok(x)`, `Result::Err(e)`, user `Color::Red`, `Shape::Circle(r)`)
//! parse and match exactly like the unqualified forms, and the transpiled
//! Rust compiles and agrees with the interpreter.

use assert_cmd::Command;
use ruchy::frontend::parser::Parser;
use std::path::Path;
use std::time::Duration;

fn ruchy() -> Command {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.timeout(Duration::from_secs(30));
    cmd
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

fn eval(src: &str) -> String {
    let mut cmd = ruchy();
    cmd.arg("-e").arg(src);
    stdout_of(cmd, &format!("ruchy -e {src}"))
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
    let file = dir.path().join("prog.ruchy");
    std::fs::write(&file, src).expect("write ruchy source");
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
fn test_qpat_1_01_qualified_option_patterns_parse() {
    for src in [
        "match x { Option::Some(v) => v, _ => 0 }",
        "match x { Option::None => 0, _ => 1 }",
        "match x { Option::Some(v) => v, Option::None => 0 }",
        "match x { Result::Ok(v) => v, Result::Err(e) => 0 }",
    ] {
        assert!(
            Parser::new(src).parse().is_ok(),
            "`{src}` should parse: {:?}",
            Parser::new(src).parse().err()
        );
    }
}

#[test]
fn test_qpat_1_02_option_some_pattern_matches() {
    assert_eq!(
        eval("println(match Some(3) { Option::Some(v) => v, _ => 0 })"),
        "3\n"
    );
    assert_eq!(
        eval("println(match None { Option::Some(v) => v, _ => 0 })"),
        "0\n"
    );
}

#[test]
fn test_qpat_1_03_option_none_pattern_matches() {
    assert_eq!(
        eval("println(match None { Option::None => 1, _ => 0 })"),
        "1\n"
    );
    assert_eq!(
        eval("println(match Some(2) { Option::None => 1, _ => 0 })"),
        "0\n"
    );
}

#[test]
fn test_qpat_1_04_result_patterns_match() {
    let src = "fun f(r) { match r { Result::Ok(v) => v, Result::Err(e) => e + 10 } }\n\
               println(f(Ok(3)))\nprintln(f(Err(3)))";
    assert_eq!(eval(src), "3\n13\n");
}

#[test]
fn test_qpat_1_05_nested_qualified_patterns() {
    let src = "fun f(o) { match o { Option::Some(Option::Some(x)) => x, Option::Some(Option::None) => -1, Option::None => -2 } }\n\
               println(f(Some(Some(5))))\nprintln(f(Some(None)))\nprintln(f(None))";
    assert_eq!(eval(src), "5\n-1\n-2\n");
}

#[test]
fn test_qpat_1_06_qualified_agrees_with_unqualified() {
    let qualified = "fun f(o) { match o { Option::Some(v) => v * 2, Option::None => 0 } }\n\
                     fun g(r) { match r { Result::Ok(v) => v, Result::Err(_) => -1 } }\n\
                     println(f(Some(4)))\nprintln(f(None))\nprintln(g(Ok(7)))\nprintln(g(Err(\"x\")))";
    let unqualified = qualified.replace("Option::", "").replace("Result::", "");
    assert_eq!(eval(qualified), eval(&unqualified));
    assert_eq!(eval(qualified), "8\n0\n7\n-1\n");
}

#[test]
fn test_qpat_1_07_user_enum_qualified_patterns() {
    let src = "enum Color { Red, Green }\nenum Shape { Circle(i64), Square(i64) }\n\
               fun code(c) { match c { Color::Red => 1, Color::Green => 2 } }\n\
               fun area(s) { match s { Shape::Circle(r) => r * 3, Shape::Square(w) => w * w } }\n\
               println(code(Color::Green))\nprintln(area(Shape::Circle(2)))\nprintln(area(Shape::Square(5)))";
    assert_eq!(eval(src), "2\n6\n25\n");
}

#[test]
fn test_qpat_1_08_differential_matches_rustc() {
    let src = r#"
enum Color { Red, Green }
enum Shape { Circle(i64), Square(i64) }
fun opt(o: Option<i64>) -> i64 {
    match o {
        Option::Some(v) => v,
        Option::None => 0,
    }
}
fun res(r: Result<i64, String>) -> i64 {
    match r {
        Result::Ok(v) => v,
        Result::Err(_) => -1,
    }
}
fun code(c: Color) -> i64 {
    match c {
        Color::Red => 1,
        Color::Green => 2,
    }
}
fun area(s: Shape) -> i64 {
    match s {
        Shape::Circle(r) => r * 3,
        Shape::Square(w) => w * w,
    }
}
fun main() {
    println(opt(Some(4)))
    println(opt(None))
    println(res(Ok(9)))
    println(res(Err("bad".to_string())))
    println(code(Color::Red))
    println(code(Color::Green))
    println(area(Shape::Circle(2)))
    println(area(Shape::Square(5)))
}
"#;
    assert_eq!(differential(src), "4\n0\n9\n-1\n1\n2\n6\n25\n");
}

#[test]
fn test_qpat_1_09_if_let_and_guard_with_qualified_patterns() {
    assert_eq!(
        eval("if let Option::Some(v) = Some(5) { println(v) } else { println(0) }"),
        "5\n"
    );
    assert_eq!(
        eval("println(match Some(3) { Option::Some(x) if x > 2 => 1, _ => 0 })"),
        "1\n"
    );
    assert_eq!(
        eval("println(match Ok(3) { Option::Some(v) => v, _ => 0 })"),
        "0\n"
    );
}
