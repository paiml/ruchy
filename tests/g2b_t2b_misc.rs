#![allow(missing_docs)]
//! RHLGA-1 (G2b T2b): miscellaneous transpile/runtime defects, checked end to end.

use assert_cmd::Command;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// Compile `src` with `ruchy compile` and return the binary's stdout.
fn compile_and_run(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let source = dir.path().join("main.ruchy");
    let binary = dir.path().join("main_bin");
    std::fs::write(&source, src).expect("write source");
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .assert()
        .success();
    let out = std::process::Command::new(&binary)
        .output()
        .expect("run binary");
    assert!(out.status.success(), "binary failed: {out:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// A sync `main` that awaits compiled to `.await` in a non-async fn (rustc E0728).
#[test]
fn test_rhlga_1_sync_main_awaiting_async_fn_compiles_and_runs() {
    let src = "async fun fetch() -> i32 {\n    42\n}\n\nfun main() {\n    let d = await fetch()\n    println(d)\n}\n";
    assert_eq!(compile_and_run(src).trim(), "42");
}

/// `async fun main` compiled to `async fn main` (rustc E0752).
#[test]
fn test_rhlga_1_async_main_compiles_and_runs() {
    let src = "async fun fetch() -> i32 {\n    40\n}\n\nasync fun main() {\n    let d = await fetch()\n    println(d + 2)\n}\n";
    assert_eq!(compile_and_run(src).trim(), "42");
}

/// A non-unit tail of an awaiting main is still printed (BOOK-COMPAT-015 semantics).
#[test]
fn test_rhlga_1_awaiting_main_prints_non_unit_tail() {
    let src = "async fun fetch() -> i32 {\n    7\n}\n\nfun main() {\n    let d = await fetch()\n    d\n}\n";
    assert_eq!(compile_and_run(src).trim(), "7");
}
