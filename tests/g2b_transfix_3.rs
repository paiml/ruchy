//! RHLGA-1 transpiler fixes (third batch), each checked end to end:
//! `ruchy transpile`, `ruchy compile` (rustc), run the binary, compare exact
//! stdout, and require `ruchy run` to print the same.
//!
//! - ARRVECARG-1: an array literal argument to a `Vec<T>` parameter becomes a
//!   `Vec` (E0308 before).
//! - MAINUNIT-1: a call to a unit-returning user function as the last
//!   statement of `main` is not wrapped in `println!("{:?}", ..)` (printed
//!   `()` before); a value-returning tail is still printed (BOOK-COMPAT-015).
//! - DICTTYPE-1: a dict literal whose values are all ints, floats or bools is
//!   typed by those values, not stringified to `BTreeMap<String, String>`.

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

// --------------------------------------------------------------- ARRVECARG-1

#[test]
fn test_arrvecarg_1_array_literal_to_vec_param() {
    let src = "fun n(v: Vec<i32>) -> i32 {\n    v[0]\n}\n\
               fun main() {\n    println(\"{}\", n([1, 2, 3]))\n}\n";
    check(src, "1\n", true);
}

#[test]
fn test_arrvecarg_1_vec_param_iterated() {
    let src = "fun total(v: Vec<i32>) -> i32 {\n    let mut s = 0\n    for x in v {\n        s = s + x\n    }\n    s\n}\n\
               fun main() {\n    println(\"{}\", total([4, 5, 6]))\n}\n";
    check(src, "15\n", true);
}

#[test]
fn test_arrvecarg_1_second_vec_param() {
    let src = "fun pick(i: i32, v: Vec<i32>) -> i32 {\n    v[i]\n}\n\
               fun main() {\n    println(\"{}\", pick(2, [7, 8, 9]))\n}\n";
    check(src, "9\n", true);
}

// ---------------------------------------------------------------- MAINUNIT-1

#[test]
fn test_mainunit_1_unit_call_after_let() {
    let src = "fun hello() {\n    println(\"hi\")\n}\n\
               fun main() {\n    let x = 1\n    hello()\n}\n";
    check(src, "hi\n", true);
}

#[test]
fn test_mainunit_1_unit_call_only_statement() {
    let src = "fun hello() {\n    println(\"hi\")\n}\n\
               fun main() {\n    hello()\n}\n";
    check(src, "hi\n", true);
}

#[test]
fn test_mainunit_1_unit_function_with_loop() {
    let src = "fun count(n: i32) {\n    for i in 0..n {\n        println(\"{}\", i)\n    }\n}\n\
               fun main() {\n    println(\"start\")\n    count(2)\n}\n";
    check(src, "start\n0\n1\n", true);
}

#[test]
fn test_mainunit_1_value_tail_still_printed() {
    // BOOK-COMPAT-015: a value-returning tail call of main is printed.
    let src = "fun add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\
               fun main() {\n    let x = 1\n    add(x, 2)\n}\n";
    let (stdout, _) = compile_and_run(src);
    assert_eq!(stdout, "3\n", "compiled stdout of:\n{}", transpile(src));
}

// ---------------------------------------------------------------- DICTTYPE-1

#[test]
fn test_dicttype_1_int_values_assign_and_add() {
    let src = main_with(
        r#"    let mut d = {"a": 1, "b": 2}
    d["a"] = 5
    println("{}", d["a"] + d["b"])"#,
    );
    check(&src, "7\n", true);
}

#[test]
fn test_dicttype_1_float_values() {
    let src = main_with(
        r#"    let d = {"x": 1.5, "y": 2.25}
    println("{}", d["x"] + d["y"])"#,
    );
    check(&src, "3.75\n", true);
}

#[test]
fn test_dicttype_1_bool_values() {
    let src = main_with(
        r#"    let mut d = {"on": true, "off": false}
    d["off"] = true
    println("{}", d["on"] && d["off"])"#,
    );
    check(&src, "true\n", true);
}

#[test]
fn test_dicttype_1_string_values_unchanged() {
    let src = main_with(
        r#"    let d = {"a": "x", "b": "y"}
    println("{}", d["a"])"#,
    );
    check(&src, "x\n", true);
}

#[test]
fn test_dicttype_1_int_values_emit_no_string_map() {
    let out = transpile(&main_with(
        r#"    let d = {"a": 1, "b": 2}
    println("{}", d["a"])"#,
    ));
    assert!(
        !out.contains("BTreeMap < String , String >") && !out.contains("BTreeMap<String, String>"),
        "int-valued dict must not be a String map:\n{out}"
    );
}
