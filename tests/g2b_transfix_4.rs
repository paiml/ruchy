//! RHLGA-1 transpiler fixes (fourth batch), each checked end to end:
//! `ruchy transpile`, `ruchy compile` (rustc), run the binary, compare exact
//! stdout, and require `ruchy run` to print the same.
//!
//! - PRINTSTRSCOPE-1: the PRINTSTR-1 string record of a name is dropped
//!   while a for-loop variable, closure parameter, or match / if-let /
//!   while-let pattern re-binds it (and restored after), and `replace` /
//!   `repeat` count as string results only on a string receiver. Before, a
//!   shadowing binding or `v.repeat(n)` on a `Vec` printed with `{}` (E0277).

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

// ----------------------------------------------------------- PRINTSTRSCOPE-1

#[test]
fn test_printstrscope_1_for_variable_shadows_string() {
    let src = main_with(
        r#"    let x = "a"
    for x in [[1]] {
        println(x)
    }
    println(x)"#,
    );
    // `ruchy run` leaks the loop variable into the enclosing scope (prints
    // `[1]` twice): an interpreter defect outside this transpiler fix.
    check(&src, "[1]\na\n", false);
}

#[test]
fn test_printstrscope_1_closure_param_shadows_string() {
    let src = main_with(
        r#"    let x = "a"
    let f = |x| println(x)
    f([5])
    println(x)"#,
    );
    check(&src, "[5]\na\n", true);
}

#[test]
fn test_printstrscope_1_closure_string_param_prints_text() {
    let src = main_with(
        r#"    let f = |s: String| println(s)
    f("hey".to_string())
    println("end")"#,
    );
    check(&src, "hey\nend\n", true);
}

#[test]
fn test_printstrscope_1_match_arm_binding_shadows_string() {
    let src = main_with(
        r#"    let x = "a"
    let o = Some([1, 2])
    match o {
        Some(x) => println(x),
        None => println("none"),
    }
    println(x)"#,
    );
    check(&src, "[1, 2]\na\n", true);
}

#[test]
fn test_printstrscope_1_if_let_binding_shadows_string() {
    let src = main_with(
        r#"    let x = "a"
    if let Some(x) = Some([3]) {
        println(x)
    }
    println(x)"#,
    );
    check(&src, "[3]\na\n", true);
}

#[test]
fn test_printstrscope_1_repeat_on_vec_is_not_a_string() {
    let src = main_with(
        r#"    let v = [1]
    println(v.repeat(2))"#,
    );
    // `ruchy run` has no array `repeat` ("Unknown array method: repeat"):
    // an interpreter gap outside this transpiler fix.
    check(&src, "[1, 1]\n", false);
}

#[test]
fn test_printstrscope_1_string_receiver_methods_keep_text() {
    let src = main_with(
        r#"    let s = "ab"
    println(s.repeat(2))
    println("a-b".replace("-", "+"))
    println(s.trim().replace("a", "z").repeat(2))"#,
    );
    check(&src, "abab\na+b\nzbzb\n", true);
}
