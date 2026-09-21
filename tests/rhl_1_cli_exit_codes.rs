//! RHL-1: `ruchy check` and `ruchy fmt` on `.rhl` files, through the built
//! binary (spec RHL-001 §5, plan D8).
//!
//! The library holds every decision and is gated by `cargo test --lib`
//! (src/rhl/cli.rs). What only a process can show is the exit code: exit 2
//! is a `process::exit` in the handler, because `main` maps every `Err` to 1.
//! This file is not in the required check (binding B4); it runs under
//! `cargo test`.

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

const TYPO: &str = "docs/rhl/breaks/planted/typo-term/01-gx10-disk-watch/broken.rhl";
const MISSING_END: &str = "docs/rhl/breaks/planted/missing-end/01-gx10-disk-watch/broken.rhl";
const VALID: &str = "docs/rhl/breaks/valid/01-gx10-disk-watch.rhl";

#[test]
fn test_rhl_1_cli_check_refusal_exits_2_with_json_report() {
    ruchy_cmd()
        .args(["check", TYPO, "--format", "json"])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("\"code\": \"RHL-V002\""))
        .stdout(predicate::str::contains("\"verdict\": \"refused\""));
}

#[test]
fn test_rhl_1_cli_check_parse_error_exits_1() {
    ruchy_cmd()
        .args(["check", MISSING_END])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("error[RHL-P001]"));
}

#[test]
fn test_rhl_1_cli_check_clean_program_exits_0() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = env!("CARGO_MANIFEST_DIR");
    for sub in ["vocab", "contracts"] {
        copy_dir(&std::path::Path::new(root).join(sub), &dir.path().join(sub));
    }
    let file = dir.path().join("clean.rhl");
    std::fs::write(
        &file,
        "use vocabulary fleet v1\n\njob \"d\"\n  every 1 hour\n  may read disk\n\n  let free be disk free of \"/\"\n\n  expect free is at least 10 GB\nend\n",
    )
    .expect("write");
    ruchy_cmd()
        .arg("check")
        .arg(&file)
        .assert()
        .code(0)
        .stdout(predicate::str::contains(": pass"));
}

#[test]
fn test_rhl_1_cli_check_rejects_an_unknown_format() {
    ruchy_cmd()
        .args(["check", VALID, "--format", "yaml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("`text` or `json`"));
}

#[test]
fn test_rhl_1_cli_fmt_check_accepts_the_normal_form() {
    ruchy_cmd()
        .args(["fmt", "--check", VALID])
        .assert()
        .success();
}

#[test]
fn test_rhl_1_cli_fmt_stdout_normalizes_layout() {
    let dir = tempfile::tempdir().expect("temp dir");
    let file = dir.path().join("m.rhl");
    std::fs::write(&file, "job \"x\"\n        every 1 hour\nend").expect("write");
    ruchy_cmd()
        .args(["fmt", "--stdout"])
        .arg(&file)
        .assert()
        .success()
        .stdout("job \"x\"\n  every 1 hour\nend\n");
}

fn copy_dir(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).expect("mkdir");
    for entry in std::fs::read_dir(from).expect("read_dir") {
        let path = entry.expect("entry").path();
        if path.is_file() {
            std::fs::copy(&path, to.join(path.file_name().expect("name"))).expect("copy");
        }
    }
}
