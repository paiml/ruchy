//! RHL-5: `ruchy test` on `.rhl` — examples lowered to hermetic tests, built,
//! run and reported per example; `--format json` for agents.

use assert_cmd::Command;
use std::path::{Path, PathBuf};

const GX10: &str = "docs/rhl/breaks/v2/valid/01-gx10-disk-watch.rhl";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A scratch root holding `vocab/`, `contracts/` and `j.rhl` = GX10 with
/// `from` replaced by `to`.
fn scratch(from: &str, to: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    for sub in ["vocab", "contracts"] {
        copy_dir(&root().join(sub), &dir.path().join(sub));
    }
    let src = std::fs::read_to_string(root().join(GX10)).expect("GX10");
    assert!(src.contains(from), "GX10 has `{from}`");
    let job = dir.path().join("j.rhl");
    std::fs::write(&job, src.replace(from, to)).expect("write j.rhl");
    (dir, job)
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("mkdir");
    for e in std::fs::read_dir(from)
        .expect("read dir")
        .filter_map(Result::ok)
    {
        let target = to.join(e.file_name());
        if e.path().is_dir() {
            copy_dir(&e.path(), &target);
        } else {
            std::fs::copy(e.path(), &target).expect("copy");
        }
    }
}

fn ruchy_test(job: &Path, json: bool) -> (String, i32) {
    let mut cmd = Command::cargo_bin("ruchy").expect("ruchy binary");
    cmd.arg("test");
    if json {
        cmd.args(["--format", "json"]);
    }
    let out = cmd.arg(job).output().expect("ruchy test runs");
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        out.status.code().unwrap_or(-1),
    )
}

#[test]
fn test_rhl5_cli_gx10_example_passes() {
    let (_d, job) = scratch("then", "then");
    let (out, exit) = ruchy_test(&job, false);
    assert_eq!(exit, 0, "{out}");
    assert!(
        out.contains("example \"gx10 disk watch example\" … ok"),
        "{out}"
    );
    assert!(out.contains("1 example, 1 passed, 0 failed"), "{out}");
}

#[test]
fn test_rhl5_cli_a_wrong_then_fails_exit_1_naming_the_example() {
    let (_d, job) = scratch("then ticket count is 1", "then ticket count is 0");
    let (out, exit) = ruchy_test(&job, false);
    assert_eq!(exit, 1, "{out}");
    assert!(
        out.contains("example \"gx10 disk watch example\" … FAILED (then ticket count is 0)"),
        "{out}"
    );
}

#[test]
fn test_rhl5_cli_a_plan_that_breaks_an_expect_fails_naming_the_expect() {
    let (_d, job) = scratch(
        "      label \"fleet\"\n    end\n",
        "      label \"fleet\"\n    end\n    file ticket in repo \"paiml/infra\" with\n      \
         title \"again\"\n    end\n",
    );
    let text = std::fs::read_to_string(&job).expect("read");
    std::fs::write(&job, text.replace("ticket count is 1", "ticket count is 2")).expect("write");
    let (out, exit) = ruchy_test(&job, false);
    assert_eq!(exit, 1, "{out}");
    assert!(
        out.contains("FAILED (expect ticket count is at most 1)"),
        "{out}"
    );
}

#[test]
fn test_rhl5_cli_json_output_parses() {
    let (_d, job) = scratch("then ticket count is 1", "then ticket count is 0");
    let (out, exit) = ruchy_test(&job, true);
    assert_eq!(exit, 1);
    let v: serde_json::Value = serde_json::from_str(&out).expect("JSON");
    assert_eq!(v["failed"], 1);
    assert_eq!(v["passed"], 0);
    assert_eq!(v["examples"][0]["name"], "gx10 disk watch example");
    assert_eq!(v["examples"][0]["ok"], false);
    assert_eq!(v["examples"][0]["failed"], "then ticket count is 0");
}

#[test]
fn test_rhl5_cli_a_job_with_no_examples_reports_zero_and_exits_0() {
    let (_d, job) = scratch(
        "  example \"gx10 disk watch example\"\n    given free is 90 GB\n    then ticket count is 1\n  end\n",
        "",
    );
    let (out, exit) = ruchy_test(&job, false);
    assert_eq!(exit, 0, "{out}");
    assert!(out.contains("0 examples, 0 passed, 0 failed"), "{out}");
}
