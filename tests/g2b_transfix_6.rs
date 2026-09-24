//! RHLGA-1 transpiler fixes (sixth batch), each checked end to end:
//! `ruchy transpile`, `ruchy compile` (rustc), run the binary, compare exact
//! stdout, and require `ruchy run` to print the same.
//!
//! - PRINTPARAM-1: an untyped function or closure parameter whose emitted
//!   (or call-site inferred) type is `String` prints with `{}` through
//!   `println(s)`. Before, `p("hi")` and `f("hi")` printed `"hi"` with quotes.
//! - NESTPUSHLIT-1: pushing or inserting an array literal into a binding
//!   whose inner literals became `vec![..]` (NESTARRVEC-1) emits the argument
//!   as `vec![..]` too. Before, rustc refused it (E0308).

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

// ------------------------------------------------------------ NESTPUSHLIT-1

#[test]
fn test_nestpushlit_1_push_literal_after_elem_growth() {
    let src = main_with(
        r#"    let mut m = [[1]]
    m[0].push(2)
    m.push([3])
    println("{:?}", m)"#,
    );
    check(&src, "[[1, 2], [3]]\n", true);
}

#[test]
fn test_nestpushlit_1_insert_literal_after_elem_growth() {
    let src = main_with(
        r#"    let mut m = [[1]]
    m[0].push(2)
    m.insert(0, [5, 6])
    println("{:?}", m)"#,
    );
    check(&src, "[[5, 6], [1, 2]]\n", true);
}

#[test]
fn test_nestpushlit_1_pushed_literal_grown_later() {
    let src = main_with(
        r#"    let mut m = [[1]]
    m.push([3])
    m[1].push(4)
    println("{:?}", m)"#,
    );
    check(&src, "[[1], [3, 4]]\n", true);
}

#[test]
fn test_nestpushlit_1_no_elem_growth_keeps_arrays() {
    let src = main_with(
        r#"    let mut m = [[1]]
    m.push([3])
    println("{:?}", m)"#,
    );
    check(&src, "[[1], [3]]\n", true);
}

// ------------------------------------------------------------ STRRECV-1

// `replace`/`repeat` on a receiver the transpiler cannot prove is a string
// (a for-loop variable, a struct field, `String::from(..)`) print as text.
#[test]
fn test_strrecv_1_for_variable_replace_prints_text() {
    let src = main_with(
        r#"    let text = "ab\ncd"
    for line in text.lines() {
        println(line.replace("a", "b"))
    }"#,
    );
    check(&src, "bb\ncd\n", true);
}

#[test]
fn test_strrecv_1_field_replace_prints_text() {
    let src = format!(
        "struct P {{ name: String }}\n{}",
        main_with(
            r#"    let p = P { name: String::from("xa") }
    println(p.name.replace("x", "y"))"#
        )
    );
    check(&src, "ya\n", true);
}

#[test]
fn test_strrecv_1_call_receiver_repeat_prints_text() {
    let src = main_with(
        r#"    let s = String::from("q")
    println(s.repeat(2))
    println(String::from("z").repeat(3))"#,
    );
    check(&src, "qq\nzzz\n", true);
}

// A list receiver still prints its `repeat` with `{:?}` (PRINTSTRSCOPE-1).
#[test]
fn test_strrecv_1_list_repeat_stays_debug() {
    let src = main_with(
        r#"    let v = [1]
    println(v.repeat(2))
    println([2].repeat(2))"#,
    );
    check(&src, "[1, 1]\n[2, 2]\n", true);
}

// A string rebinding of a list name prints its `repeat` as text again.
#[test]
fn test_strrecv_1_list_name_rebound_to_string() {
    let src = main_with(
        r#"    let v = [1]
    println(v.len())
    let v = String::from("ab")
    println(v.repeat(2))"#,
    );
    check(&src, "1\nabab\n", true);
}

// A loop variable over string items prints as text.
#[test]
fn test_strrecv_1_loop_variable_over_string_items() {
    let src = main_with(
        r#"    let text = "a b\nc"
    for line in text.lines() {
        println(line)
    }
    for w in text.split_whitespace() {
        println(w)
    }
    for p in "x,y".split(",") {
        println(p)
    }
    for s in ["m", "n"] {
        println(s)
    }"#,
    );
    check(&src, "a b\nc\na\nb\nc\nx\ny\nm\nn\n", true);
}

// A for loop, a while loop or a compound assignment as main's last
// statement is unit: the compiled binary prints nothing for it.
#[test]
fn test_strrecv_1_trailing_loops_are_unit() {
    let src = main_with(
        r#"    let mut i = 0
    while i < 2 {
        i += 1
    }
    println(i)
    for k in [1] {
        println(k)
    }"#,
    );
    check(&src, "2\n1\n", true);
    check(&main_with("    let mut j = 1\n    j += 1"), "", true);
}

// A list-typed parameter or annotated binding keeps `repeat` on `{:?}`
// (a `{}` would not build: `Vec` has no Display).
#[test]
fn test_strrecv_1_list_typed_param_repeat_stays_debug() {
    let src = format!(
        "fun f(v: Vec<i32>) {{\n    println(v.repeat(2))\n}}\n{}",
        main_with("    let w: Vec<i32> = vec![3]\n    println(w.repeat(2))\n    f(vec![1])")
    );
    check(&src, "[3, 3]\n[1, 1]\n", true);
}

// `Option::replace` takes one argument and is not a string.
#[test]
fn test_strrecv_1_option_replace_stays_debug() {
    let src = main_with("    let mut o = Some(1)\n    println(o.replace(2))\n    println(o)");
    check(&src, "Some(1)\nSome(2)\n", false);
}
