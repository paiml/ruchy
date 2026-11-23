//! PMAT-135 — the release policy gate must be falsifiable, not decorative.
//!
//! "CI held a publish credential" is the mechanism behind the 403 that stopped the
//! 5.0.0-beta.2 release. "Producer is never the gate": a workflow that gates AND
//! publishes is one program attesting to itself. `scripts/release-policy.sh` is that
//! separation made checkable — CI is the gate, the operator is the publisher.
//!
//! These tests drive the script through its fixtures so the gate itself is proven to
//! turn RED. A gate that only ever prints PASS measures nothing.

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Run `bash scripts/release-policy.sh <args>` from the repo root.
/// Returns (exit code, stdout followed by stderr).
fn run_policy(args: &[&str]) -> (i32, String) {
    let out = Command::new("bash")
        .arg("scripts/release-policy.sh")
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("run bash scripts/release-policy.sh");
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code().unwrap_or(-1), text)
}

/// The script's own falsifiers must all turn RED against the fixtures.
#[test]
fn test_pmat_135_release_policy_self_test_turns_red_on_fixtures() {
    let (code, text) = run_policy(&["--self-test"]);
    assert_eq!(
        code, 0,
        "scripts/release-policy.sh --self-test must exit 0; got {code}:\n{text}"
    );
}

#[test]
fn test_pmat_135_no_publish_gate_fails_on_a_workflow_that_publishes() {
    let (code, text) = run_policy(&[
        "--only",
        "no-publish-in-ci",
        "--gates-dir",
        "tests/fixtures/release-policy/with-publish",
    ]);
    assert_eq!(
        code, 1,
        "a workflow directory that publishes a crate must exit 1; got {code}:\n{text}"
    );
    assert!(
        text.contains("FAIL no-publish-in-ci"),
        "the gate must name itself in the failure line:\n{text}"
    );
}

#[test]
fn test_pmat_135_no_publish_gate_passes_on_a_workflow_that_does_not_publish() {
    let (code, text) = run_policy(&[
        "--only",
        "no-publish-in-ci",
        "--gates-dir",
        "tests/fixtures/release-policy/without-publish",
    ]);
    assert_eq!(
        code, 0,
        "a workflow directory with no publish step must exit 0; got {code}:\n{text}"
    );
    assert!(
        text.contains("PASS no-publish-in-ci"),
        "the gate must report PASS by name:\n{text}"
    );
}

/// The gate applied to the real thing: `.github/workflows/` must be publish-free.
#[test]
fn test_pmat_135_no_publish_gate_passes_on_the_repository_workflows() {
    let (code, text) = run_policy(&["--only", "no-publish-in-ci"]);
    assert_eq!(
        code, 0,
        "no workflow in .github/workflows/ may publish a crate; got {code}:\n{text}"
    );
}
