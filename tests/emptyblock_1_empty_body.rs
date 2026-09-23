//! EMPTYBLOCK-1: an empty `{ }` in statement-body position (if / else /
//! while / for / fun body) is an empty block, not an empty object literal.
//! `if a < 2 { }` used to give the `if` a BTreeMap-typed branch that rustc
//! rejects. In expression position (`let m = {}`) an empty `{}` stays an
//! empty object literal, as the parser documents. Each test checks the
//! emitted Rust text AND that interpreter output equals the rustc output.

use assert_cmd::Command;
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
fn assert_no_map(rust: &str) {
    assert!(
        !rust.contains("BTreeMap"),
        "empty body must not become an object literal:\n{rust}"
    );
}

#[test]
fn test_emptyblock_1_if_without_else_empty_body() {
    let src = "fun f(a: i32) -> i32 {\n    if a < 2 { }\n    a + 1\n}\n\
               fun main() {\n    println(f(1))\n    println(f(5))\n}\n";
    let rust = differential(src);
    assert_no_map(&rust);
    // the branch is unit-typed: `{}` or `{ () }`, never a map
    assert_shape(&rust, "if a < 2 {");
}

#[test]
fn test_emptyblock_1_empty_then_with_else() {
    let src = "fun f(a: i32) {\n    if a < 2 { } else { println(\"big\") }\n}\n\
               fun main() {\n    f(1)\n    f(5)\n    println(\"end\")\n}\n";
    let rust = differential(src);
    assert_no_map(&rust);
}

#[test]
fn test_emptyblock_1_empty_else_branch() {
    let src = "fun f(a: i32) {\n    if a < 2 { println(\"small\") } else { }\n}\n\
               fun main() {\n    f(1)\n    f(5)\n    println(\"end\")\n}\n";
    let rust = differential(src);
    assert_no_map(&rust);
}

#[test]
fn test_emptyblock_1_while_and_for_empty_body() {
    let src = "fun main() {\n    let a = 5\n    while a < 0 { }\n    for i in 0..3 { }\n    println(a)\n}\n";
    let rust = differential(src);
    assert_no_map(&rust);
}

#[test]
fn test_emptyblock_1_fun_empty_body_returns_unit() {
    let src = "fun noop() { }\nfun main() {\n    noop()\n    println(\"done\")\n}\n";
    let rust = differential(src);
    assert_no_map(&rust);
    // no `-> BTreeMap` return type is inferred for an empty body
    assert_shape(&rust, "fn noop() {");
}

#[test]
fn test_emptyblock_1_let_empty_braces_stays_object_literal() {
    let src = "fun main() {\n    let m = {}\n    println(m)\n}\n";
    let rust = differential(src);
    assert!(
        rust.contains("BTreeMap"),
        "`let m = {{}}` must stay an empty object literal:\n{rust}"
    );
}
