//! TRANSPILECMP-1: a parenthesised comparison used as a comparison operand
//! must stay parenthesised. Rust comparison operators are non-associative, so
//! `(a < 1) == false` emitted as `a < 1 == false` is rejected by rustc
//! ("comparison operators cannot be chained"). Each test checks the emitted
//! Rust text AND that interpreter output equals the rustc-compiled output.

use assert_cmd::Command;
use proptest::prelude::*;
use proptest::test_runner::Config;
use std::path::Path;
use std::time::Duration;

fn ruchy() -> Command {
    let mut cmd = Command::cargo_bin("ruchy").expect("ruchy binary is built for tests");
    cmd.timeout(Duration::from_secs(30));
    cmd
}

fn write_source(dir: &Path, src: &str) -> std::path::PathBuf {
    let file = dir.join("prog.ruchy");
    std::fs::write(&file, src).expect("write ruchy source");
    file
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

fn transpile(dir: &Path, src: &str) -> String {
    let file = write_source(dir, src);
    let mut cmd = ruchy();
    cmd.arg("transpile").arg(file);
    stdout_of(cmd, "ruchy transpile")
}

fn interpret(dir: &Path, src: &str) -> String {
    let file = write_source(dir, src);
    let mut cmd = ruchy();
    cmd.arg("run").arg(file);
    stdout_of(cmd, "ruchy run")
}

fn rustc_run(dir: &Path, rust: &str) -> String {
    let main_rs = dir.join("main.rs");
    let bin = dir.join("prog_bin");
    std::fs::write(&main_rs, rust).expect("write main.rs");
    let mut rustc = Command::new("rustc");
    rustc
        .args([
            "--edition",
            "2021",
            "-A",
            "warnings",
            "-D",
            "unused_parens",
            "-o",
        ])
        .arg(&bin)
        .arg(&main_rs)
        .timeout(Duration::from_secs(120));
    stdout_of(rustc, &format!("rustc on transpiled Rust:\n{rust}\n"));
    let mut run = Command::new(&bin);
    run.timeout(Duration::from_secs(30));
    stdout_of(run, "transpiled binary")
}

/// Transpile, compile, run; assert interpreter == rustc output. Returns the Rust text.
fn differential(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let rust = transpile(dir.path(), src);
    let expected = interpret(dir.path(), src);
    let actual = rustc_run(dir.path(), &rust);
    assert_eq!(
        actual, expected,
        "transpiled Rust disagrees with the interpreter\nsource:\n{src}\nrust:\n{rust}"
    );
    rust
}

fn squash(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

fn assert_shape(rust: &str, needle: &str) {
    assert!(
        squash(rust).contains(&squash(needle)),
        "expected `{needle}` in transpiled Rust:\n{rust}"
    );
}
#[test]
fn test_transpilecmp_1_less_under_equal_keeps_parens() {
    let src = "fun f(a: i32) -> bool { (a < 1) == false }\n\
               fun main() {\n    println(f(5))\n    println(f(0))\n}\n";
    let rust = differential(src);
    assert_shape(&rust, "(a < 1) == false");
}

#[test]
fn test_transpilecmp_1_let_binding_of_compared_comparison() {
    let src = "fun main() {\n    let a = 5\n    let u = (a < 1) == false\n    println(u)\n}\n";
    let rust = differential(src);
    assert_shape(&rust, "(a < 1) == false");
}

#[test]
fn test_transpilecmp_1_equal_under_not_equal_keeps_parens() {
    let src = "fun f(a: i32, b: i32, p: bool) -> bool { (a == b) != p }\n\
               fun main() {\n    println(f(1, 1, true))\n    println(f(1, 2, true))\n}\n";
    let rust = differential(src);
    assert_shape(&rust, "(a == b) != p");
}

#[test]
fn test_transpilecmp_1_comparison_on_both_sides() {
    let src = "fun f(a: i32, b: i32) -> bool { (a < b) == (b >= 3) }\n\
               fun main() {\n    println(f(1, 4))\n    println(f(5, 4))\n}\n";
    let rust = differential(src);
    assert_shape(&rust, "(a < b) == (b >= 3)");
}

#[test]
fn test_transpilecmp_1_right_nested_equality_keeps_parens() {
    let src = "fun f(p: bool, q: bool, r: bool) -> bool { p == (q == r) }\n\
               fun main() {\n    println(f(false, true, false))\n}\n";
    let rust = differential(src);
    assert_shape(&rust, "p == (q == r)");
}

#[test]
fn test_transpilecmp_1_bitwise_under_comparison_keeps_rust_meaning() {
    let src = "fun f(a: i32, b: i32) -> bool { (a & b) == 0 }\n\
               fun g(a: i32, b: i32) -> i32 { (a | b) & 6 }\n\
               fun main() {\n    println(f(4, 3))\n    println(g(1, 4))\n}\n";
    let rust = differential(src);
    // Rust binds `&` tighter than `==`, so no parentheses are required here
    assert_shape(&rust, "a & b == 0");
    assert_shape(&rust, "(a | b) & 6");
}

// ---- property: comparisons nested under comparisons --------------------

fn cmp_leaf() -> impl Strategy<Value = String> {
    let int = prop::sample::select(vec!["a", "b", "1", "4"]);
    let op = prop::sample::select(vec!["<", "<=", ">", ">=", "==", "!="]);
    (int.clone(), op, int).prop_map(|(l, op, r)| format!("({l} {op} {r})"))
}

fn cmp_tree() -> impl Strategy<Value = String> {
    let leaf = prop_oneof![
        cmp_leaf(),
        prop::sample::select(vec!["p", "true", "false"]).prop_map(str::to_string),
    ];
    leaf.prop_recursive(3, 8, 2, |inner| {
        (inner.clone(), prop::sample::select(vec!["==", "!="]), inner)
            .prop_map(|(l, op, r)| format!("({l} {op} {r})"))
    })
}

proptest! {
    #![proptest_config(Config {
        cases: 12,
        failure_persistence: None,
        ..Config::default()
    })]

    #[test]
    fn test_transpilecmp_1_property_nested_comparisons(
        exprs in prop::collection::vec(cmp_tree(), 4),
    ) {
        let mut src = String::new();
        let mut calls = String::new();
        for (i, e) in exprs.iter().enumerate() {
            src += &format!("fun c{i}(a: i32, b: i32, p: bool) -> bool {{ {e} }}\n");
            calls += &format!("    println(c{i}(2, 4, true))\n");
        }
        differential(&format!("{src}fun main() {{\n{calls}}}\n"));
    }
}
