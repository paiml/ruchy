#![allow(missing_docs)]
//! COMPILER-001: `ruchy compile` must honour (by isolating from)
//! `CARGO_TARGET_DIR`.
//!
//! Discovered via Genchi Genbutsu during ruchy-book ch18 validation on
//! 5.0.0-beta.1. When the parent environment sets `CARGO_TARGET_DIR` to
//! share a build cache, cargo writes the compiled binary there instead of
//! into the temp project's local `target/` directory. The compiler's
//! post-build lookup then fails with "Expected binary not found".
//!
//! The fix (src/backend/compiler.rs `compile_with_cargo`) removes
//! `CARGO_TARGET_DIR` from the cargo invocation's environment so the
//! temp project gets its own isolated `target/`.
//!
//! Ticket: [COMPILER-001] Cargo target dir isolation in ruchy compile.

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_compiler_001_compile_succeeds_with_cargo_target_dir_set() {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("hello.ruchy");
    fs::write(&src, "fun main() { println(\"hi\") }\n").unwrap();
    let out = tmp.path().join("hello_bin");

    // Force CARGO_TARGET_DIR to point somewhere the compiler must NOT
    // look for the binary. If the compiler honoured this, it would not
    // find the binary at <temp>/target/release/. The fix removes
    // CARGO_TARGET_DIR from the cargo invocation so the lookup succeeds.
    let shared_target = tmp.path().join("shared_cargo_target");
    fs::create_dir_all(&shared_target).unwrap();

    ruchy_cmd()
        .env("CARGO_TARGET_DIR", &shared_target)
        .arg("compile")
        .arg(&src)
        .arg("-o")
        .arg(&out)
        .assert()
        .success();

    assert!(out.exists(), "compiled binary must exist at {}", out.display());
}
