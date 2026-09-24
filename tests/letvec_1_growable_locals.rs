//! LETVEC-1: an array literal bound by `let` is growable in the interpreter,
//! so the transpiled Rust must be a `Vec` whenever the program needs one.
//!
//! Before the fix `let mut v = [1, 2]; v.push(3)` transpiled to
//! `let mut v = [1, 2]; v.push(3);` and rustc failed with E0599 (no `push`
//! on `[{integer}; 2]`), with or without a `Vec<i32>` annotation, in a
//! `fun main` and in a top-level script.
//!
//! The literal becomes `vec![..]` when the binding is annotated `Vec<..>` or
//! is, later in its scope, the receiver of a Vec-only method. A literal that
//! is never grown stays a fixed-size array (`sort`/`reverse` alone work on
//! Rust arrays and do not force a `Vec`).

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

fn stdout_of(mut cmd: Command, what: &str) -> String {
    let out = cmd.output().expect("spawn command");
    assert!(
        out.status.success(),
        "{what} failed: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

fn interpret(dir: &Path, src: &str) -> String {
    let file = write_source(dir, src);
    let mut cmd = ruchy();
    cmd.arg("run").arg(file);
    stdout_of(cmd, "ruchy run")
}

fn transpile(dir: &Path, src: &str) -> String {
    let file = write_source(dir, src);
    let mut cmd = ruchy();
    cmd.arg("transpile").arg(file);
    stdout_of(cmd, "ruchy transpile")
}

fn transpile_str(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    transpile(dir.path(), src)
}

/// `ruchy compile` (rustc) the source, then run the binary.
fn compile_and_run(dir: &Path, src: &str) -> String {
    let file = write_source(dir, src);
    let bin = dir.join("prog_bin");
    let mut cmd = ruchy();
    cmd.arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(&bin)
        .timeout(Duration::from_secs(60));
    stdout_of(cmd, &format!("ruchy compile of:\n{src}\n"));
    let mut run = Command::new(&bin);
    run.timeout(Duration::from_secs(30));
    stdout_of(run, "compiled binary")
}

/// Strip whitespace so `vec ! [1 , 2]` and `vec![1,2]` compare equal.
fn squash(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// The four binding shapes: (label, program) for `op` applied to
/// `[3, 1, 2, 2]`, printing the array and its length afterwards.
fn forms(op: &str) -> Vec<(&'static str, String)> {
    let body = |decl: &str, indent: &str| {
        format!("{indent}let mut v{decl} = [3, 1, 2, 2]\n{indent}v.{op}\n{indent}println(v)\n{indent}println(v.len())\n")
    };
    vec![
        (
            "fun main",
            format!("fun main() {{\n{}}}\n", body("", "    ")),
        ),
        (
            "fun main, Vec<i32>",
            format!("fun main() {{\n{}}}\n", body(": Vec<i32>", "    ")),
        ),
        ("script", body("", "")),
        ("script, Vec<i32>", body(": Vec<i32>", "")),
    ]
}

/// Every form transpiles to a `vec![..]` binding, compiles with rustc, and
/// prints `expected` (checked against the interpreter when it supports `op`).
fn assert_grows(op: &str, expected: &str) {
    for (label, src) in forms(op) {
        let dir = tempfile::tempdir().expect("tempdir");
        let rust = transpile(dir.path(), &src);
        assert!(
            squash(&rust).contains("vec![3,1,2,2]"),
            "{label}, `{op}`: expected a vec![..] binding in:\n{rust}"
        );
        let actual = compile_and_run(dir.path(), &src);
        assert_eq!(actual, expected, "{label}, `{op}`: rust:\n{rust}");
    }
}

/// As [`assert_grows`], and the interpreter prints the same thing.
fn assert_grows_like_interpreter(op: &str, expected: &str) {
    for (label, src) in forms(op) {
        let dir = tempfile::tempdir().expect("tempdir");
        assert_eq!(
            interpret(dir.path(), &src),
            expected,
            "{label}, `{op}`: interpreter"
        );
    }
    assert_grows(op, expected);
}

#[test]
fn test_letvec_1_01_push() {
    assert_grows_like_interpreter("push(9)", "[3, 1, 2, 2, 9]\n5\n");
}

#[test]
fn test_letvec_1_02_pop() {
    assert_grows_like_interpreter("pop()", "[3, 1, 2]\n3\n");
}

#[test]
fn test_letvec_1_03_extend() {
    assert_grows_like_interpreter("extend([7, 8])", "[3, 1, 2, 2, 7, 8]\n6\n");
}

#[test]
fn test_letvec_1_04_insert() {
    assert_grows_like_interpreter("insert(0, 9)", "[9, 3, 1, 2, 2]\n5\n");
}

#[test]
fn test_letvec_1_05_remove() {
    assert_grows_like_interpreter("remove(1)", "[3, 2, 2]\n3\n");
}

#[test]
fn test_letvec_1_06_clear() {
    assert_grows_like_interpreter("clear()", "[]\n0\n");
}

#[test]
fn test_letvec_1_07_truncate() {
    assert_grows_like_interpreter("truncate(2)", "[3, 1]\n2\n");
}

#[test]
fn test_letvec_1_08_dedup() {
    assert_grows_like_interpreter("dedup()", "[3, 1, 2]\n3\n");
}

#[test]
fn test_letvec_1_09_append_makes_a_vec() {
    // `append` is emitted as `push` by call_transpilation.rs, which rustc
    // rejects on a Vec<i32>; only the binding shape is asserted here.
    for (label, src) in forms("append([7, 8])") {
        let rust = transpile_str(&src);
        assert!(
            squash(&rust).contains("vec![3,1,2,2]"),
            "{label}: expected a vec![..] binding in:\n{rust}"
        );
    }
}

#[test]
fn test_letvec_1_10_retain() {
    assert_grows("retain(|x| *x > 1)", "[3, 2, 2]\n3\n");
}

#[test]
fn test_letvec_1_11_drain() {
    assert_grows("drain(0..2)", "[2, 2]\n2\n");
}

#[test]
fn test_letvec_1_12_resize() {
    assert_grows("resize(6, 0)", "[3, 1, 2, 2, 0, 0]\n6\n");
}

#[test]
fn test_letvec_1_13_extend_from_slice() {
    assert_grows("extend_from_slice(&[7])", "[3, 1, 2, 2, 7]\n5\n");
}

#[test]
fn test_letvec_1_14_never_grown_stays_array() {
    let src = "fun main() {\n    let v = [1, 2, 3]\n    println(v.len())\n}\n";
    let rust = squash(&transpile_str(src));
    assert!(
        rust.contains("letv=[1,2,3];"),
        "expected a fixed array:\n{rust}"
    );
    assert!(!rust.contains("vec!"), "unexpected vec!:\n{rust}");
}

#[test]
fn test_letvec_1_15_sort_and_reverse_alone_stay_array() {
    let src = "fun main() {\n    let mut v = [3, 1, 2]\n    v.sort()\n    v.reverse()\n    println(v)\n}\n";
    let dir = tempfile::tempdir().expect("tempdir");
    let rust = transpile(dir.path(), src);
    assert!(
        squash(&rust).contains("letmutv=[3,1,2];"),
        "expected a fixed array:\n{rust}"
    );
    assert_eq!(compile_and_run(dir.path(), src), "[3, 2, 1]\n");
}

#[test]
fn test_letvec_1_16_only_the_grown_binding_becomes_vec() {
    let src = "fun main() {\n    let a = [1, 2]\n    let mut b = [3]\n    b.push(4)\n    println(a.len() + b.len())\n}\n";
    let dir = tempfile::tempdir().expect("tempdir");
    let rust = squash(&transpile(dir.path(), src));
    assert!(rust.contains("leta=[1,2];"), "a stays an array:\n{rust}");
    assert!(
        rust.contains("letmutb=vec![3];"),
        "b becomes a vec:\n{rust}"
    );
    assert_eq!(compile_and_run(dir.path(), src), "4\n");
}

#[test]
fn test_letvec_1_17_growth_inside_nested_block() {
    let src = "fun main() {\n    let mut v = [1]\n    if true {\n        for i in 0..3 {\n            v.push(i)\n        }\n    }\n    println(v.len())\n}\n";
    let dir = tempfile::tempdir().expect("tempdir");
    let expected = interpret(dir.path(), src);
    assert_eq!(expected, "4\n");
    assert_eq!(compile_and_run(dir.path(), src), expected);
}
