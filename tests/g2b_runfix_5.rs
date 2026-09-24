//! RHLGA-1 G2b run fixes, part 5.
//!
//! - FORLEAK-1: a `for` loop bound its variable with an assignment, so a
//!   same-named outer binding was overwritten, and a destructuring pattern
//!   (`for (k, v) in ...`) bound the whole element to its first name. Each
//!   iteration now binds its pattern in a scope of its own.
//! - ARRREPEAT-1: arrays had no `repeat` method (Rust `slice::repeat`).
//! - FORMATVAL-1: `format`/`println` called through a variable or parameter
//!   treated a non-literal first argument as a format template; only a
//!   string literal is one (FMTTEMPLATE-1).
//!
//! Where the transpiler can build the program, `ruchy run` is compared with
//! the binary `ruchy compile` builds from the same source.

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

// -------------------------------------------------------------- FORLEAK-1

#[test]
fn test_forleak_1_outer_binding_survives_same_named_loop_var() {
    assert_run_matches_compiled(
        "let x = \"a\"\nfor x in [[1]] { }\nprintln(\"{}\", x)\n",
        "a\n",
    );
}

#[test]
fn test_forleak_1_outer_binding_survives_range_loop_var() {
    assert_run_matches_compiled(
        "let i = 10\nfor i in 0..3 { println(\"{}\", i) }\nprintln(\"{}\", i)\n",
        "0\n1\n2\n10\n",
    );
}

#[test]
fn test_forleak_1_outer_mutation_in_body_persists() {
    assert_run_matches_compiled(
        "let mut s = 0\nfor i in [1, 2] { s += i }\nprintln(\"{}\", s)\n",
        "3\n",
    );
}

#[test]
fn test_forleak_1_break_and_continue_still_work() {
    assert_run_matches_compiled(
        "let mut t = 0\nfor i in 0..10 {\n    if i == 1 { continue }\n    if i == 4 { break }\n    t += i\n}\nprintln(\"{}\", t)\n",
        "5\n",
    );
}

#[test]
fn test_forleak_1_tuple_pattern_binds_each_name() {
    assert_run_prints(
        "let k = 7\nfor (k, v) in [(1, 2), (3, 4)] { println(\"{} {}\", k, v) }\nprintln(\"{}\", k)\n",
        "1 2\n3 4\n7\n",
    );
}

#[test]
fn test_forleak_1_pattern_binding_leaves_outer_second_name() {
    assert_run_prints(
        "let v = \"outer\"\nfor (k, v) in [(1, 2)] { println(\"{}\", k + v) }\nprintln(\"{}\", v)\n",
        "3\nouter\n",
    );
}

#[test]
fn test_forleak_1_if_let_while_let_match_do_not_leak() {
    assert_run_prints(
        "let y = 1\nif let Some(y) = Some(3) { println(\"{}\", y) }\nprintln(\"{}\", y)\n\
         let mut w = Some(4)\nlet q = 1\nwhile let Some(q) = w { println(\"{}\", q)\n w = None }\nprintln(\"{}\", q)\n\
         let z = 1\nmatch 5 { z => println(\"{}\", z) }\nprintln(\"{}\", z)\n",
        "3\n1\n4\n1\n5\n1\n",
    );
}
