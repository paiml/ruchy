//! RHL-5b tests: the receipt verdict is three-valued and never a fabricated
//! Pass; the receipt of a compiled job and its determinism.

use super::{verdict_of, Verdict};
use crate::rhl::cli::{compile_file, contract_path, receipt_path, ExampleResult};
use std::path::{Path, PathBuf};

fn result(ok: bool) -> ExampleResult {
    ExampleResult {
        name: "e".to_string(),
        ok,
        failed: (!ok).then(|| "then x".to_string()),
    }
}

#[test]
fn test_rhl5b_verdict_is_pass_only_with_every_example_holding() {
    assert_eq!(verdict_of(Ok(&[result(true), result(true)])), Verdict::Pass);
    assert_eq!(
        verdict_of(Ok(&[result(true), result(false)])),
        Verdict::Fail
    );
    assert_eq!(
        verdict_of(Ok(&[])),
        Verdict::Unknown("no examples".to_string())
    );
    assert_eq!(
        verdict_of(Err("cannot build".to_string())),
        Verdict::Unknown("cannot build".to_string())
    );
}

#[test]
fn test_rhl5b_verdict_serializes_three_valued() {
    let j = |v: &Verdict| serde_json::to_string(v).expect("json");
    assert_eq!(j(&Verdict::Pass), "\"Pass\"");
    assert_eq!(j(&Verdict::Fail), "\"Fail\"");
    assert_eq!(
        j(&Verdict::Unknown("no examples".to_string())),
        "{\"Unknown\":\"no examples\"}"
    );
}

// ---------------------------------------------------------------- compiled

const GX10: &str = "docs/rhl/breaks/v2/valid/01-gx10-disk-watch.rhl";

fn rustc_available() -> bool {
    let ok = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    if !ok {
        println!("RHL-5b: skipped: no `rustc` on PATH to build the job");
    }
    ok
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

/// A scratch root with `vocab/`, `contracts/` and `j.rhl` = GX10 edited.
fn scratch(edit: impl Fn(String) -> String) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = crate::rhl::repo_root();
    for sub in ["vocab", "contracts"] {
        copy_dir(&root.join(sub), &dir.path().join(sub));
    }
    let src = std::fs::read_to_string(root.join(GX10)).expect("GX10");
    let job = dir.path().join("j.rhl");
    std::fs::write(&job, edit(src)).expect("write j.rhl");
    (dir, job)
}

/// Compile `job` to `<dir>/<name>` and read its receipt as JSON.
fn compiled(job: &Path, name: &str) -> (serde_json::Value, String, String) {
    let out = job.with_file_name(name);
    let o = compile_file(job, &out);
    assert_eq!(o.exit, 0, "{}{}", o.stdout, o.stderr);
    let receipt = std::fs::read_to_string(receipt_path(&out)).expect("receipt written");
    let contract = std::fs::read_to_string(contract_path(&out)).expect("contract written");
    let value = serde_json::from_str(&receipt).expect("receipt is JSON");
    (value, receipt, contract)
}

#[test]
fn test_rhl5b_receipt_gx10_passes_and_names_its_inputs() {
    if !rustc_available() {
        return;
    }
    let (_d, job) = scratch(|s| s);
    let (r, bytes, contract) = compiled(&job, "a");
    assert_eq!(r["verdict"], "Pass", "{bytes}");
    let src = std::fs::read(&job).expect("source");
    assert_eq!(r["inputs"]["source_sha256"], super::sha256_hex(&src));
    assert_eq!(r["contract_sha256"], super::sha256_hex(contract.as_bytes()));
    assert_eq!(r["inputs"]["vocabularies"][0]["name"], "fleet");
    assert_eq!(r["inputs"]["vocabularies"][0]["version"], 2);
    assert_eq!(r["ruchy_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(r["rust_sha256"].as_str().map(str::len), Some(64), "{bytes}");
    let files = r["inputs"]["runtime_files"].as_array().expect("array");
    assert!(
        files
            .iter()
            .any(|f| f["path"] == "vocab/runtime/fleet.ruchy"),
        "{bytes}"
    );
    let (_, again, contract2) = compiled(&job, "b");
    assert_eq!(bytes, again, "same inputs ⇒ byte-identical receipt");
    assert_eq!(contract, contract2, "same inputs ⇒ byte-identical contract");
}

#[test]
fn test_rhl5b_receipt_a_wrong_then_is_fail() {
    if !rustc_available() {
        return;
    }
    let (_d, job) = scratch(|s| s.replace("then ticket count is 1", "then ticket count is 0"));
    let (r, bytes, _) = compiled(&job, "a");
    assert_eq!(r["verdict"], "Fail", "{bytes}");
}

#[test]
fn test_rhl5b_receipt_no_examples_is_unknown() {
    if !rustc_available() {
        return;
    }
    let (_d, job) = scratch(|s| {
        s.replace(
            "  example \"gx10 disk watch example\"\n    given free is 90 GB\n    then ticket count is 1\n  end\n",
            "",
        )
    });
    let (r, bytes, _) = compiled(&job, "a");
    assert_eq!(r["verdict"]["Unknown"], "no examples", "{bytes}");
}

#[test]
fn test_rhl5b_compile_refuses_c002_when_the_contract_cannot_be_written() {
    let (_d, job) = scratch(|s| s);
    let out = job.with_file_name("a");
    std::fs::create_dir_all(contract_path(&out)).expect("a directory where the contract goes");
    let o = compile_file(&job, &out);
    assert_eq!(o.exit, 2, "{}", o.stderr);
    assert!(o.stderr.contains(crate::rhl::codes::C002), "{}", o.stderr);
    assert!(!out.exists(), "no binary without its contract");
}
