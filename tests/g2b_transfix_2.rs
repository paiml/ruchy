//! RHLGA-1 transpiler fixes (second batch), each checked end to end:
//! `ruchy transpile`, `ruchy compile` (rustc), run the binary, compare exact
//! stdout.
//!
//! - PRINTSTR-1: a single string-typed argument to print/println (a string
//!   literal binding, `to_string()`, `format!`, a `String` parameter) prints
//!   with `{}`, not `{:?}`, so the compiled program prints no quotes.
//! - FORMATFN-1: the function form `format(...)` transpiles like `format!(...)`.
//! - RETLEN-1: an explicit `return <len()/count()>` in an int-returning
//!   function is cast, including early returns inside `if` branches.
//! - DICTIDX-1: a string key indexes a dict literal as a map (read, index
//!   assignment, in-place method); `as usize` is only for integer indices.
//!
//! PRINTSTR-1 and RETLEN-1 also require `ruchy run` to agree; the FORMATFN-1
//! and DICTIDX-1 interpreter halves are tracked separately.

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

// ---------------------------------------------------------------- PRINTSTR-1

#[test]
fn test_printstr_1_literal_to_string_and_format_bindings() {
    let src = main_with(
        r#"    let s = "hi"
    let t = s.to_string()
    let u = format!("{}-{}", s, 1)
    println(s)
    println(t)
    println(u)"#,
    );
    check(&src, "hi\nhi\nhi-1\n", true);
}

#[test]
fn test_printstr_1_string_parameter() {
    let src = "fun greet(name: String) {\n    println(name)\n}\n\
               fun main() {\n    greet(\"bob\".to_string())\n    println(\"done\")\n}\n";
    check(src, "bob\ndone\n", true);
}

#[test]
fn test_printstr_1_string_method_result() {
    check(
        &main_with(r#"    println("a b".to_uppercase())"#),
        "A B\n",
        true,
    );
}

#[test]
fn test_printstr_1_numbers_and_vecs_unchanged() {
    let src = main_with(
        r#"    let n = 42
    let v = [1, 2]
    println(n)
    println(v)"#,
    );
    check(&src, "42\n[1, 2]\n", true);
}

// ---------------------------------------------------------------- FORMATFN-1

#[test]
fn test_formatfn_1_debug_placeholder() {
    let src = main_with(
        r#"    let f = format("{:?}", [1, 2])
    println(f)"#,
    );
    check(&src, "[1, 2]\n", false);
}

#[test]
fn test_formatfn_1_display_placeholders() {
    let src = main_with(
        r#"    let g = format("{}-{}", "a", 3)
    println(g)"#,
    );
    check(&src, "a-3\n", false);
}

// ---------------------------------------------------------------- RETLEN-1

#[test]
fn test_retlen_1_explicit_return_len() {
    // A String argument: an array literal passed to a `Vec` parameter is a
    // separate defect (no `.to_vec()`), kept out of this case.
    let src = "fun n(s: String) -> int {\n    return s.len()\n}\n\
               fun main() {\n    println(n(\"abc\".to_string()))\n}\n";
    check(src, "3\n", true);
}

#[test]
fn test_retlen_1_early_return_in_if_branch() {
    let src = "fun m(s: String) -> int {\n    if s.len() > 2 {\n        return s.len()\n    }\n    return 0\n}\n\
               fun main() {\n    println(m(\"abc\".to_string()))\n    println(m(\"a\".to_string()))\n}\n";
    check(src, "3\n0\n", true);
}

// ---------------------------------------------------------------- DICTIDX-1

#[test]
fn test_dictidx_1_push_through_string_key() {
    let src = main_with(
        r#"    let mut d = {"k": [1]}
    d["k"].push(2)
    println("{:?}", d["k"])"#,
    );
    check(&src, "[1, 2]\n", false);
}

#[test]
fn test_dictidx_1_read_string_key() {
    let src = main_with(
        r#"    let d = {"a": [1, 2]}
    println("{:?}", d["a"])"#,
    );
    check(&src, "[1, 2]\n", false);
}

#[test]
fn test_dictidx_1_assign_string_key() {
    let src = main_with(
        r#"    let mut d = {"k": [1]}
    d["k"] = [7, 8]
    println("{:?}", d["k"])"#,
    );
    check(&src, "[7, 8]\n", false);
}

#[test]
fn test_dictidx_1_debug_print_map() {
    let src = main_with(
        r#"    let d = {"a": [1]}
    println("{:?}", d)"#,
    );
    check(&src, "{\"a\": [1]}\n", false);
}

#[test]
fn test_dictidx_1_integer_index_unchanged() {
    let src = main_with(
        r#"    let v = [1, 2, 3]
    let i = 1
    println(v[i])"#,
    );
    check(&src, "2\n", true);
}
