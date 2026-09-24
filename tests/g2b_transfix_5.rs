//! RHLGA-1 transpiler fixes (fifth batch), each checked end to end:
//! `ruchy transpile`, `ruchy compile` (rustc), run the binary, compare exact
//! stdout, and require `ruchy run` to print the same.
//!
//! - NESTARRVEC-1: when an element of a nested array literal binding is
//!   grown through an index (`m[0].push(9)`), the inner literals are emitted
//!   as `vec![..]` too. Before, they stayed fixed-size arrays (E0599).

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

// -------------------------------------------------------------- NESTARRVEC-1

#[test]
fn test_nestarrvec_1_push_through_index() {
    let src = main_with(
        r#"    let mut m = [[1], [2]]
    m[0].push(9)
    println("{:?}", m)"#,
    );
    check(&src, "[[1, 9], [2]]\n", true);
}

#[test]
fn test_nestarrvec_1_insert_through_index() {
    let src = main_with(
        r#"    let mut m = [[1, 2], [3]]
    m[1].insert(0, 7)
    println("{:?}", m)"#,
    );
    check(&src, "[[1, 2], [7, 3]]\n", true);
}

#[test]
fn test_nestarrvec_1_push_in_loop() {
    let src = main_with(
        r#"    let mut m = [[0], [0], [0]]
    for i in 0..3 {
        m[i].push(i)
    }
    println("{:?}", m)"#,
    );
    check(&src, "[[0, 0], [0, 1], [0, 2]]\n", true);
}

#[test]
fn test_nestarrvec_1_outer_and_inner_grown() {
    let src = main_with(
        r#"    let mut m = [[1]]
    m[0].push(5)
    m.push(m[0].clone())
    println("{:?}", m)"#,
    );
    // `ruchy run` has no `clone` array method ("Unknown array method:
    // clone"): an interpreter defect outside this transpiler fix.
    check(&src, "[[1, 5], [1, 5]]\n", false);
}

#[test]
fn test_nestarrvec_1_ungrown_nested_stays_array() {
    let src = main_with(
        r#"    let m = [[1], [2]]
    println("{:?}", m)"#,
    );
    let rust = transpile(&src);
    assert!(
        !rust.contains("vec !"),
        "no vec! for an ungrown literal:\n{rust}"
    );
    check(&src, "[[1], [2]]\n", true);
}
