//! RHLGA-1 transpiler fixes (sixth batch), each checked end to end:
//! `ruchy transpile`, `ruchy compile` (rustc), run the binary, compare exact
//! stdout, and require `ruchy run` to print the same.
//!
//! - PRINTPARAM-1: an untyped function or closure parameter whose emitted
//!   (or call-site inferred) type is `String` prints with `{}` through
//!   `println(s)`. Before, `p("hi")` and `f("hi")` printed `"hi"` with quotes.

use assert_cmd::Command;
use std::path::Path;
use std::time::Duration;

fn ruchy() -> Command {
    let mut cmd = Command::cargo_bin("ruchy").expect("ruchy binary is built for tests");
    cmd.timeout(Duration::from_secs(10));
    cmd
}

fn write_source(dir: &Path, src: &str) -> std::path::PathBuf {
    let file = dir.join("prog.ruchy");
    std::fs::write(&file, src).expect("write ruchy source");
    file
}

/// (stdout, stderr) of a command that must succeed.
fn output_of(mut cmd: Command, what: &str) -> (String, String) {
    let out = cmd.output().expect("spawn command");
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    let stderr = String::from_utf8(out.stderr).expect("utf8 stderr");
    assert!(out.status.success(), "{what} failed:\n{stdout}{stderr}");
    (stdout, stderr)
}

fn transpile(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_source(dir.path(), src);
    let mut cmd = ruchy();
    cmd.arg("transpile").arg(file);
    output_of(cmd, &format!("ruchy transpile of:\n{src}\n")).0
}

/// `ruchy compile` the source, run the binary, return (stdout, stderr).
fn compile_and_run(src: &str) -> (String, String) {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_source(dir.path(), src);
    let bin = dir.path().join("prog_bin");
    let mut cmd = ruchy();
    cmd.arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(&bin)
        .timeout(Duration::from_secs(90));
    output_of(cmd, &format!("ruchy compile of:\n{src}\n"));
    let mut run = Command::new(&bin);
    run.timeout(Duration::from_secs(30));
    output_of(run, "compiled binary")
}

fn interpret(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_source(dir.path(), src);
    let mut cmd = ruchy();
    cmd.arg("run").arg(file);
    output_of(cmd, &format!("ruchy run of:\n{src}\n")).0
}

fn main_with(body: &str) -> String {
    format!("fun main() {{\n{body}\n}}\n")
}

/// Compiled stdout is `expected`; with `interp`, `ruchy run` agrees.
fn check(src: &str, expected: &str, interp: bool) {
    let (stdout, _) = compile_and_run(src);
    assert_eq!(
        stdout,
        expected,
        "compiled stdout of:\n{src}\n{}",
        transpile(src)
    );
    if interp {
        assert_eq!(interpret(src), expected, "interpreter stdout of:\n{src}");
    }
}

// ------------------------------------------------------------- PRINTPARAM-1

#[test]
fn test_printparam_1_named_fn_string_param() {
    let src = format!("fun p(s) {{ println(s) }}\n{}", main_with("    p(\"hi\")"));
    check(&src, "hi\n", true);
}

#[test]
fn test_printparam_1_named_fn_mixed_params() {
    let src = format!(
        "fun p(s, n) {{\n    println(s)\n    println(n)\n}}\n{}",
        main_with("    p(\"a\", 3)")
    );
    check(&src, "a\n3\n", true);
}

#[test]
fn test_printparam_1_named_fn_int_param_unchanged() {
    let src = format!("fun p(n) {{ println(n) }}\n{}", main_with("    p(5)"));
    check(&src, "5\n", true);
}

#[test]
fn test_printparam_1_closure_string_param() {
    let src = main_with(
        r#"    let f = |s| println(s)
    f("hi")"#,
    );
    check(&src, "hi\n", true);
}

#[test]
fn test_printparam_1_closure_called_twice() {
    let src = main_with(
        r#"    let f = |s| println(s)
    f("a")
    f("b")"#,
    );
    check(&src, "a\nb\n", true);
}

#[test]
fn test_printparam_1_closure_top_level() {
    let src = "let f = |s| println(s)\nf(\"hi\")\n";
    check(src, "hi\n", true);
}

#[test]
fn test_printparam_1_closure_list_param_keeps_debug() {
    let src = main_with(
        r#"    let f = |v| println(v)
    f([1, 2])"#,
    );
    let rust = transpile(&src);
    assert!(
        rust.contains("{:?}"),
        "a non-string parameter keeps {{:?}}:\n{rust}"
    );
    check(&src, "[1, 2]\n", true);
}
