#![allow(missing_docs)]
//! COMPILERACE-1: `ruchy compile` must never write its output into the
//! repository working directory.
//!
//! **Root Cause**: `ruchy compile FILE` defaults to `-o a.out` in the CURRENT
//! directory (src/bin/ruchy.rs). Tests that invoke `ruchy compile` via
//! `assert_cmd` without `-o`/`--output` and without `.current_dir(...)` all
//! write to `<repo_root>/a.out`. When several such tests run in parallel,
//! rustc's intermediate codegen objects (`a.*.rcgu.o`) collide next to the
//! shared output path and one test's cleanup/compile step deletes another's
//! objects, producing spurious rust-lld "No such file or directory" failures
//! (measured in tests/transpiler_defect_017_array_to_vec_GREEN.rs during a
//! full workspace run; passes alone).
//!
//! This test documents and enforces the rule: every `ruchy compile`
//! invocation in the test suite must direct its output into a directory the
//! test owns, e.g. via `.current_dir(<TempDir>)` or `-o <TempDir>/<name>`.

use assert_cmd::Command;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

#[test]
fn test_compilerace_1_compile_with_current_dir_stays_out_of_repo() {
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_a_out = repo_root.join("a.out");

    // Baseline: the repo root must not already have an a.out (the
    // acceptance command for this ticket also asserts this after the full
    // suite runs).
    assert!(
        !repo_a_out.exists(),
        "precondition failed: {} already exists before running the test",
        repo_a_out.display()
    );

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");
    std::fs::write(&test_file, "println(\"compilerace-1\")\n").unwrap();

    // Compile without an explicit -o/--output, relying on .current_dir(...)
    // to keep the default `a.out` inside the test's own temp directory.
    ruchy_cmd()
        .arg("compile")
        .arg(&test_file)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let binary_path = temp_dir.path().join("a.out");
    assert!(
        binary_path.exists(),
        "compiled binary should be created inside the temp dir"
    );

    // The rule this test documents: no a.out should ever land in the repo
    // root as a side effect of this invocation.
    assert!(
        !repo_a_out.exists(),
        "ruchy compile must not write its default output ({}) into the repo root",
        repo_a_out.display()
    );
}
