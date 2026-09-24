//! RHLGA-1 G2b run fixes, part 6.
//!
//! - ARRCLONE-1: arrays had no `clone` method ("Unknown array method:
//!   clone"), nor `to_vec`/`to_owned`; tuples, plain objects and strings had
//!   no `clone` either, and strings and tuples no `to_owned`. A clone is independent of
//!   its source: mutating it leaves the original unchanged.
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

// ------------------------------------------------------------- ARRCLONE-1

#[test]
fn test_arrclone_1_array_clone_repro() {
    assert_run_matches_compiled(
        "let a = [1]\nlet b = a.clone()\nprintln(\"{:?}\", b)\n",
        "[1]\n",
    );
}

// The transpiler builds `[1, 2]` as a fixed array, which has no `push`.
#[test]
fn test_arrclone_1_array_clone_is_independent() {
    assert_run_prints(
        "let a = [1, 2]\nlet mut b = a.clone()\nb.push(3)\nb[0] = 9\nprintln(\"{:?}\", a)\nprintln(\"{:?}\", b)\n",
        "[1, 2]\n[9, 2, 3]\n",
    );
}

#[test]
fn test_arrclone_1_nested_array_clone_is_independent() {
    assert_run_prints(
        "let a = [[1], [2]]\nlet mut b = a.clone()\nb[0] = [7]\nprintln(\"{:?}\", a)\nprintln(\"{:?}\", b)\n",
        "[[1], [2]]\n[[7], [2]]\n",
    );
}

#[test]
fn test_arrclone_1_array_to_vec_and_to_owned() {
    assert_run_matches_compiled(
        "let a = [1, 2]\nprintln(\"{:?}\", a.to_vec())\nprintln(\"{:?}\", a.to_owned())\n",
        "[1, 2]\n[1, 2]\n",
    );
}

#[test]
fn test_arrclone_1_tuple_clone() {
    assert_run_matches_compiled(
        "let t = (1, \"x\")\nlet u = t.clone()\nprintln(\"{:?}\", u)\nprintln(\"{:?}\", t.to_owned())\n",
        "(1, \"x\")\n(1, \"x\")\n",
    );
}

#[test]
fn test_arrclone_1_string_clone_and_to_owned() {
    assert_run_matches_compiled(
        "let s = \"hi\"\nlet r = s.clone()\nprintln(\"{}\", r)\nprintln(\"{}\", s.to_owned())\n",
        "hi\nhi\n",
    );
}

#[test]
fn test_arrclone_1_object_clone_is_independent() {
    assert_run_prints(
        "let o = {x: 1}\nlet mut p = o.clone()\np.x = 5\nprintln(\"{:?}\", o.x)\nprintln(\"{:?}\", p.x)\n",
        "1\n5\n",
    );
}
