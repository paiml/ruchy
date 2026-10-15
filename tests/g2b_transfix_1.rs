//! RHLGA-1 transpiler fixes, each checked end to end: `ruchy transpile`,
//! `ruchy compile` (rustc), run the binary, compare exact stdout.
//!
//! - PRINTLNFMT-1: a string-literal first argument containing any `{…}`
//!   placeholder (`{:?}`, `{:>5}`, `{0}`, …) is the format string. Before the
//!   fix only `{}` was recognised, so `println("{:?}", v)` printed `{:?} [1, 7, 3]`.
//! - INTLEN-1: a `usize` expression (`len()`/`count()`) meeting an `int`
//!   let annotation, return type or known parameter type is cast (`as i64`).
//! - COUNTITER-1: `count()` on a receiver that is already an iterator
//!   (`chars()`, `iter()`, …) is called directly, without another `.iter()`.
//!
//! The interpreter (`ruchy run`) must agree for the PRINTLNFMT `{:?}` cases
//! and the INTLEN cases.

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
    format!("fun main() {{\n    let v = [1, 7, 3]\n    let x = 5\n{body}\n}}\n")
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

// ---------------------------------------------------------------- PRINTLNFMT-1

#[test]
fn test_printlnfmt_1_debug_placeholder_vec() {
    check(&main_with(r#"    println("{:?}", v)"#), "[1, 7, 3]\n", true);
}

#[test]
fn test_printlnfmt_1_debug_placeholder_int() {
    check(&main_with(r#"    println("{:?}", x)"#), "5\n", true);
}

#[test]
fn test_printlnfmt_1_prefixed_debug_placeholder() {
    check(
        &main_with(r#"    println("v={:?}", v)"#),
        "v=[1, 7, 3]\n",
        true,
    );
}

#[test]
fn test_printlnfmt_1_display_placeholders_unchanged() {
    let src = main_with(
        r#"    println("{}", x)
    println("{} and {:?}", x, [1])"#,
    );
    check(&src, "5\n5 and [1]\n", true);
}

#[test]
fn test_printlnfmt_1_spec_placeholders_compiled() {
    let src = main_with(
        r#"    println("{:>5}|", x)
    println("{:.2}", 3.14159)
    println("{0}-{0}", x)
    println("{:#?}", x)"#,
    );
    check(&src, "    5|\n3.14\n5-5\n5\n", false);
}

#[test]
fn test_printlnfmt_1_print_form() {
    let src = main_with(
        r#"    print("{:?};", v)
    println("")"#,
    );
    // The interpreter prints `"{:?};" [1, 7, 3]` for `print` (runtime, not transpiler).
    check(&src, "[1, 7, 3];\n", false);
}

#[test]
fn test_printlnfmt_1_eprintln_form() {
    let src = main_with(
        r#"    eprintln("{:?}", v)
    eprint("{:?}!", x)"#,
    );
    let (stdout, stderr) = compile_and_run(&src);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "[1, 7, 3]\n5!", "{}", transpile(&src));
}

#[test]
fn test_printlnfmt_1_escaped_braces_are_not_placeholders() {
    let rust = transpile(&main_with(r#"    println("{{x}}", v)"#));
    assert!(
        !rust.contains(r#"println!("{{x}}", v)"#),
        "only-escape literal is not a format string:\n{rust}"
    );
}

// ------------------------------------------------------------------- INTLEN-1

#[test]
fn test_intlen_1_let_annotation() {
    let src = main_with(
        r#"    let n: int = v.len()
    println(n + 1)"#,
    );
    check(&src, "4\n", true);
}

#[test]
fn test_intlen_1_let_annotation_string_len() {
    let src = main_with(
        r#"    let s = "hello"
    let n: int = s.len()
    println(n * 2)"#,
    );
    check(&src, "10\n", true);
}

#[test]
fn test_intlen_1_function_return() {
    let src = "fun h(xs: Vec<i32>) -> int { xs.len() }\n\
               fun main() {\n    let mut w = [4, 5]\n    w.push(6)\n    println(h(w) + 1)\n}\n";
    check(src, "4\n", true);
}

#[test]
fn test_intlen_1_known_parameter() {
    let src = "fun g(n: int) -> int { n * 2 }\n\
               fun main() {\n    let v = [1, 7, 3]\n    println(g(v.len()))\n}\n";
    check(src, "6\n", true);
}

// ----------------------------------------------------------------- COUNTITER-1

#[test]
fn test_countiter_1_chars_count() {
    let src = main_with(
        r#"    let s = "hello"
    println(s.chars().count())"#,
    );
    check(&src, "5\n", false);
}

#[test]
fn test_countiter_1_iter_count() {
    check(&main_with("    println(v.iter().count())"), "3\n", false);
}

#[test]
fn test_countiter_1_collection_count_still_iterates() {
    check(&main_with("    println(v.count())"), "3\n", false);
}
