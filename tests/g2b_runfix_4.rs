//! RHLGA-1 G2b run fixes, part 4.
//!
//! - IDXEVAL1-1: `ruchy run` evaluated the index expression of a compound
//!   assignment (`v[next()] += 1`) or a nested-index assignment
//!   (`m[f()][1] = 9`) twice: once to read the place and once to write it
//!   back. Compiled code evaluates it once.
//! - FMTTEMPLATE-1: `ruchy run` treated a non-literal first argument of
//!   `println(s, 1)` as a format template; the transpiler treats only a
//!   string literal as one and prints every argument separated by spaces.
//! - FMTEXTRA-1: the format engine ignored an extra positional argument
//!   (`println("{}", 1, 2)`); rustc rejects an argument the template never
//!   uses, so `ruchy run` now reports an error.
//!
//! Every accepted case compares `ruchy run` with the binary `ruchy compile`
//! builds from the same source.

use assert_cmd::Command;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn write_source(dir: &Path, src: &str) -> PathBuf {
    let file = dir.join("prog.ruchy");
    std::fs::write(&file, src).expect("write ruchy source");
    file
}

fn ruchy() -> Command {
    Command::cargo_bin("ruchy").expect("ruchy binary is built for tests")
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

fn run_cmd(dir: &Path, src: &str) -> Command {
    let mut cmd = ruchy();
    cmd.arg("run")
        .arg(write_source(dir, src))
        .timeout(Duration::from_secs(10));
    cmd
}

fn compile_cmd(dir: &Path, src: &str) -> (Command, PathBuf) {
    let bin = dir.join("prog_bin");
    let mut cmd = ruchy();
    cmd.arg("compile")
        .arg(write_source(dir, src))
        .arg("-o")
        .arg(&bin)
        .timeout(Duration::from_secs(180));
    (cmd, bin)
}

fn compiled_run(dir: &Path, src: &str) -> String {
    let (cmd, bin) = compile_cmd(dir, src);
    stdout_of(cmd, "ruchy compile");
    let mut run = Command::new(&bin);
    run.timeout(Duration::from_secs(10));
    stdout_of(run, "compiled binary")
}

/// The compiled binary prints `expected`, and `ruchy run` prints it too.
fn assert_run_matches_compiled(src: &str, expected: &str) {
    let dir = tempfile::tempdir().expect("tempdir");
    assert_eq!(compiled_run(dir.path(), src), expected, "compiled binary");
    assert_eq!(
        stdout_of(run_cmd(dir.path(), src), "ruchy run"),
        expected,
        "ruchy run"
    );
}

/// `ruchy run` prints `expected`: the output Rust semantics give. Used where
/// the transpiler cannot build the program yet.
fn assert_run_prints(src: &str, expected: &str) {
    let dir = tempfile::tempdir().expect("tempdir");
    assert_eq!(
        stdout_of(run_cmd(dir.path(), src), "ruchy run"),
        expected,
        "ruchy run"
    );
}

/// `ruchy compile` rejects `src`, and `ruchy run` fails with `needle` on
/// stderr.
fn assert_both_reject(src: &str, needle: &str) {
    let dir = tempfile::tempdir().expect("tempdir");
    let (mut cmd, _) = compile_cmd(dir.path(), src);
    cmd.assert().failure();
    let out = run_cmd(dir.path(), src).output().expect("spawn ruchy run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "ruchy run accepted: {stderr}");
    assert!(stderr.contains(needle), "stderr lacks {needle:?}: {stderr}");
}

// ------------------------------------------------------------- IDXEVAL1-1

const COUNTER: &str = r#"
let mut calls = 0
fun next() -> i64 {
    calls += 1
    println("index {}", calls)
    0
}
"#;

fn with_counter(main: &str) -> String {
    format!("{COUNTER}{main}")
}

#[test]
fn test_idxeval1_1_compound_index_evaluates_index_once() {
    let src = with_counter(
        r#"
fun main() {
    let mut v = [10, 20]
    v[next()] += 1
    v[next()] *= 3
    println("{:?} {}", v, calls)
}
"#,
    );
    assert_run_matches_compiled(&src, "index 1\nindex 2\n[33, 20] 2\n");
}

#[test]
fn test_idxeval1_1_nested_index_assign_evaluates_index_once() {
    let src = with_counter(
        r#"
fun main() {
    let mut m = [[1, 2], [3, 4]]
    m[next()][1] = 9
    println("{:?} {}", m, calls)
}
"#,
    );
    assert_run_matches_compiled(&src, "index 1\n[[1, 9], [3, 4]] 1\n");
}

#[test]
fn test_idxeval1_1_nested_compound_evaluates_each_index_once() {
    let src = with_counter(
        r#"
fun main() {
    let mut m = [[1, 2], [3, 4]]
    m[next()][next()] += 5
    println("{:?} {}", m, calls)
}
"#,
    );
    assert_run_matches_compiled(&src, "index 1\nindex 2\n[[6, 2], [3, 4]] 2\n");
}

/// The transpiler emits a nested array literal as a fixed-size array, which
/// has no `push`, so this case is checked against `ruchy run` alone.
#[test]
fn test_idxeval1_1_in_place_push_on_indexed_element_evaluates_index_once() {
    let src = with_counter(
        r#"
fun main() {
    let mut m = [[1], [2]]
    m[next()].push(7)
    println("{:?} {}", m, calls)
}
"#,
    );
    assert_run_prints(&src, "index 1\n[[1, 7], [2]] 1\n");
}

// ---------------------------------------------------------- FMTTEMPLATE-1

#[test]
fn test_fmttemplate_1_variable_first_arg_is_not_a_template() {
    assert_run_matches_compiled(
        r#"
fun main() {
    let s = "{}"
    println(s, 1)
    let t = "x={} y={}"
    println(t, 2, 3)
    println("a", 1)
    println("{} and {}", 4, 5)
}
"#,
        "{} 1\nx={} y={} 2 3\na 1\n4 and 5\n",
    );
}

// ------------------------------------------------------------- FMTEXTRA-1

#[test]
fn test_fmtextra_1_extra_positional_argument_is_rejected() {
    assert_both_reject(
        r#"
fun main() {
    println("{} {}", 1, 2, 3)
}
"#,
        "argument never used",
    );
}

#[test]
fn test_fmtextra_1_unreferenced_explicit_index_is_rejected() {
    assert_both_reject(
        r#"
fun main() {
    println("{0} {0}", 1, 2)
}
"#,
        "argument never used",
    );
}

#[test]
fn test_fmtextra_1_explicit_indices_count_as_uses() {
    assert_run_matches_compiled(
        r#"
fun main() {
    println("{1} {0} {}", 1, 2)
    println("{} {0}", 3)
}
"#,
        "2 1 1\n3 3\n",
    );
}
