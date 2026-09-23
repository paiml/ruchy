//! TRANSPILENOT-1: unary operators must keep the grouping of a binary operand.
//!
//! The parser keeps no parenthesis node, so the emitter must re-insert the
//! parentheses a Rust reader needs: `!(b && x)` transpiled to `!b && x` (a
//! silent meaning change), `!(s == "p")` to `!s == "p"` (E0600), and
//! `a - (b - c)` to `a - b - c`. Every test checks the emitted Rust text AND
//! the behaviour: the interpreter (`ruchy run`) and the transpiled program
//! compiled with rustc must print the same thing.

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
        .args(["--edition", "2021", "-A", "warnings", "-o"])
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
fn test_transpilenot_1_not_over_and_keeps_parens() {
    let src = "fun f(b: bool, x: bool) -> bool { !(b && x) }\n\
               fun main() {\n    println(f(true, false))\n    println(f(true, true))\n}\n";
    let rust = differential(src);
    assert_shape(&rust, "!(b && x)");
}

#[test]
fn test_transpilenot_1_not_over_or_keeps_parens() {
    let src = "fun f(b: bool, x: bool) -> bool { !(b || x) }\n\
               fun main() {\n    println(f(false, true))\n    println(f(false, false))\n}\n";
    let rust = differential(src);
    assert_shape(&rust, "!(b || x)");
}

#[test]
fn test_transpilenot_1_not_over_string_equality_compiles() {
    let src = "fun g(s: String) -> bool { !(s == \"p\") }\n\
               fun main() {\n    println(g(\"q\".to_string()))\n    println(g(\"p\".to_string()))\n}\n";
    let rust = differential(src);
    assert_shape(&rust, "!(s == \"p\")");
}

#[test]
fn test_transpilenot_1_negate_over_sum_keeps_parens() {
    let src = "fun f(a: i32, b: i32) -> i32 { -(a + b) * 2 }\n\
               fun main() {\n    println(f(3, 4))\n}\n";
    let rust = differential(src);
    assert_shape(&rust, "-(a + b) * 2");
}

#[test]
fn test_transpilenot_1_right_operand_same_precedence_keeps_parens() {
    let src = "fun f(a: i32, b: i32, c: i32) -> i32 { a - (b - c) }\n\
               fun g(a: i32, b: i32, c: i32) -> i32 { a / (b * c) }\n\
               fun main() {\n    println(f(10, 4, 3))\n    println(g(100, 5, 2))\n}\n";
    let rust = differential(src);
    assert_shape(&rust, "a - (b - c)");
    assert_shape(&rust, "a / (b * c)");
}

#[test]
fn test_transpilenot_1_or_under_and_keeps_parens() {
    let src = "fun f(a: bool, b: bool, c: bool) -> bool { (a || b) && c }\n\
               fun main() {\n    println(f(true, false, false))\n}\n";
    let rust = differential(src);
    assert_shape(&rust, "(a || b) && c");
}

#[test]
fn test_transpilenot_1_redundant_parens_not_added_to_atoms() {
    let src = "fun f(b: bool, x: i32) -> bool { !b && -x < 0 }\n\
               fun main() {\n    println(f(false, 3))\n}\n";
    let rust = differential(src);
    assert_shape(&rust, "!b && -x < 0");
}

// ---- property: random parenthesised int/bool expression trees ----------

#[derive(Clone, Debug)]
enum IntE {
    Leaf(&'static str),
    Neg(Box<IntE>),
    Bin(Box<IntE>, &'static str, Box<IntE>),
}

#[derive(Clone, Debug)]
enum BoolE {
    Leaf(&'static str),
    Not(Box<BoolE>),
    Bin(Box<BoolE>, &'static str, Box<BoolE>),
    Cmp(IntE, &'static str, IntE),
}

fn render_int(e: &IntE) -> String {
    match e {
        IntE::Leaf(s) => (*s).to_string(),
        IntE::Neg(x) => format!("-({})", render_int(x)),
        IntE::Bin(l, op, r) => format!("({} {op} {})", render_int(l), render_int(r)),
    }
}

fn render_bool(e: &BoolE) -> String {
    match e {
        BoolE::Leaf(s) => (*s).to_string(),
        BoolE::Not(x) => format!("!({})", render_bool(x)),
        BoolE::Bin(l, op, r) => format!("({} {op} {})", render_bool(l), render_bool(r)),
        BoolE::Cmp(l, op, r) => format!("({} {op} {})", render_int(l), render_int(r)),
    }
}

fn int_expr() -> impl Strategy<Value = IntE> {
    let leaf = prop::sample::select(vec!["a", "b", "c", "1", "2", "7"]).prop_map(IntE::Leaf);
    leaf.prop_recursive(3, 8, 2, |inner| {
        prop_oneof![
            inner.clone().prop_map(|x| IntE::Neg(Box::new(x))),
            (
                inner.clone(),
                prop::sample::select(vec!["+", "-", "*"]),
                inner
            )
                .prop_map(|(l, op, r)| IntE::Bin(Box::new(l), op, Box::new(r))),
        ]
    })
}

fn bool_expr() -> impl Strategy<Value = BoolE> {
    let cmp_op = prop::sample::select(vec!["<", "<=", ">", ">=", "==", "!="]);
    let leaf = prop_oneof![
        prop::sample::select(vec!["p", "q", "true", "false"]).prop_map(BoolE::Leaf),
        (int_expr(), cmp_op, int_expr()).prop_map(|(l, op, r)| BoolE::Cmp(l, op, r)),
    ];
    leaf.prop_recursive(3, 8, 2, |inner| {
        prop_oneof![
            inner.clone().prop_map(|x| BoolE::Not(Box::new(x))),
            (
                inner.clone(),
                prop::sample::select(vec!["&&", "||", "==", "!="]),
                inner
            )
                .prop_map(|(l, op, r)| BoolE::Bin(Box::new(l), op, Box::new(r))),
        ]
    })
}

const PARAMS: &str = "a: i32, b: i32, c: i32, p: bool, q: bool";
const ARGS: &str = "3, -2, 5, true, false";

fn program(ints: &[IntE], bools: &[BoolE]) -> String {
    let mut src = String::new();
    let mut calls = String::new();
    for (i, e) in ints.iter().enumerate() {
        src += &format!("fun i{i}({PARAMS}) -> i32 {{ {} }}\n", render_int(e));
        calls += &format!("    println(i{i}({ARGS}))\n");
    }
    for (i, e) in bools.iter().enumerate() {
        src += &format!("fun b{i}({PARAMS}) -> bool {{ {} }}\n", render_bool(e));
        calls += &format!("    println(b{i}({ARGS}))\n");
    }
    format!("{src}fun main() {{\n{calls}}}\n")
}

proptest! {
    #![proptest_config(Config {
        cases: 16,
        failure_persistence: None,
        ..Config::default()
    })]

    #[test]
    fn test_transpilenot_1_property_interpreter_matches_rustc(
        ints in prop::collection::vec(int_expr(), 3),
        bools in prop::collection::vec(bool_expr(), 3),
    ) {
        differential(&program(&ints, &bools));
    }
}
