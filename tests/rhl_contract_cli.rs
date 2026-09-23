//! RHL-5b: `ruchy transpile x.rhl --emit contract [-o out]` and the files
//! `ruchy compile x.rhl -o out` writes beside the binary.

use assert_cmd::Command;
use std::path::{Path, PathBuf};

const GX10: &str = "docs/rhl/breaks/v2/valid/01-gx10-disk-watch.rhl";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

/// A scratch root holding `vocab/`, `contracts/` and `j.rhl` = GX10.
fn scratch() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    for sub in ["vocab", "contracts"] {
        copy_dir(&root().join(sub), &dir.path().join(sub));
    }
    let job = dir.path().join("j.rhl");
    std::fs::copy(root().join(GX10), &job).expect("copy GX10");
    (dir, job)
}

fn ruchy(args: &[&str]) -> (String, String, i32) {
    let out = Command::cargo_bin("ruchy")
        .expect("ruchy binary")
        .args(args)
        .output()
        .expect("ruchy runs");
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.code().unwrap_or(-1),
    )
}

#[test]
fn test_rhl5b_cli_emit_contract_prints_the_unit_contract() {
    let (_d, job) = scratch();
    let (out, err, exit) = ruchy(&[
        "transpile",
        job.to_str().expect("utf8"),
        "--emit",
        "contract",
    ]);
    assert_eq!(exit, 0, "{err}");
    assert!(out.contains("  expect_0:\n"), "{out}");
    assert!(out.contains("\"plan_ticket_count(plan) ≤ 1\""), "{out}");
    assert!(out.contains("\"may write tickets\""), "{out}");
}

#[test]
fn test_rhl5b_cli_emit_contract_to_a_file_is_deterministic() {
    let (d, job) = scratch();
    let j = job.to_str().expect("utf8");
    let a = d.path().join("a.yaml");
    let b = d.path().join("b.yaml");
    for o in [&a, &b] {
        let (_, err, exit) = ruchy(&[
            "transpile",
            j,
            "--emit",
            "contract",
            "-o",
            o.to_str().expect("utf8"),
        ]);
        assert_eq!(exit, 0, "{err}");
    }
    let (ta, tb) = (std::fs::read(&a).expect("a"), std::fs::read(&b).expect("b"));
    assert!(!ta.is_empty());
    assert_eq!(ta, tb, "same inputs ⇒ byte-identical contract");
}

#[test]
fn test_rhl5b_cli_unknown_emit_names_contract() {
    let (_d, job) = scratch();
    let (_, err, exit) = ruchy(&["transpile", job.to_str().expect("utf8"), "--emit", "yaml"]);
    assert_eq!(exit, 2, "{err}");
    assert!(err.contains("`contract`"), "{err}");
}

#[test]
fn test_rhl5b_cli_compile_writes_binary_contract_and_receipt() {
    let (d, job) = scratch();
    let out = d.path().join("gx10");
    let (_, err, exit) = ruchy(&[
        "compile",
        job.to_str().expect("utf8"),
        "-o",
        out.to_str().expect("utf8"),
    ]);
    assert_eq!(exit, 0, "{err}");
    assert!(out.exists(), "binary");
    let contract = std::fs::read_to_string(d.path().join("gx10.contract.yaml")).expect("contract");
    assert!(contract.contains("expect_0"), "{contract}");
    let receipt = std::fs::read_to_string(d.path().join("gx10.receipt.json")).expect("receipt");
    let r: serde_json::Value = serde_json::from_str(&receipt).expect("JSON");
    assert_eq!(r["verdict"], "Pass", "{receipt}");
    for key in ["inputs", "ruchy_version", "rust_sha256", "contract_sha256"] {
        assert!(r.get(key).is_some(), "{key}: {receipt}");
    }
}
