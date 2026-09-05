//! PMAT-093 — the release workflow must be able to publish, not merely look like it.
//!
//! At `main` on 2026-09-05 `.github/workflows/release.yml` published a `ruchy-cli`
//! package that is not a workspace member, wrapped both `cargo publish` steps in
//! `continue-on-error: true`, read a `CRATES_TOKEN` secret that does not exist, and
//! could publish before the binaries had built. Every one of those is a way for the
//! job to go green without a crate reaching crates.io. These tests pin the fixed
//! shape. Each is red on the pre-fix file (see the DoD mutations in the plan).

use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn release_yml_text() -> String {
    fs::read_to_string(repo_root().join(".github/workflows/release.yml"))
        .expect("read .github/workflows/release.yml")
}

fn release_yml() -> Value {
    serde_yaml::from_str(&release_yml_text()).expect("release.yml parses as YAML")
}

/// Package names of every workspace member, read from each member's Cargo.toml.
fn workspace_member_names() -> Vec<String> {
    let root = repo_root();
    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("read Cargo.toml");
    let members_block = manifest
        .split("members = [")
        .nth(1)
        .and_then(|rest| rest.split(']').next())
        .expect("[workspace] members list");
    members_block
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .map(|dir| package_name(&root.join(dir).join("Cargo.toml")))
        .collect()
}

fn package_name(manifest: &Path) -> String {
    let text =
        fs::read_to_string(manifest).unwrap_or_else(|e| panic!("{}: {e}", manifest.display()));
    let mut in_package = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
        } else if in_package && line.starts_with("name") {
            return line
                .split('=')
                .nth(1)
                .unwrap()
                .trim()
                .trim_matches('"')
                .to_string();
        }
    }
    panic!("{}: no [package] name", manifest.display())
}

fn publish_job() -> Value {
    release_yml()["jobs"]["publish-crates"].clone()
}

/// Every `run:` script in a job, concatenated.
fn job_run_scripts(job: &Value) -> String {
    job["steps"]
        .as_sequence()
        .expect("steps")
        .iter()
        .filter_map(|s| s["run"].as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every step of every job, as (job id, step index, step).
fn all_steps() -> Vec<(String, usize, Value)> {
    let yml = release_yml();
    let jobs = yml["jobs"].as_mapping().expect("jobs mapping");
    let mut out = Vec::new();
    for (job_id, job) in jobs {
        for (i, step) in job["steps"].as_sequence().into_iter().flatten().enumerate() {
            out.push((job_id.as_str().unwrap().to_string(), i, step.clone()));
        }
    }
    out
}

#[test]
fn test_pmat_093_no_step_is_masked_with_continue_on_error() {
    let masked: Vec<String> = all_steps()
        .into_iter()
        .filter(|(_, _, step)| !step["continue-on-error"].is_null())
        .map(|(job, i, step)| format!("{job}#{i} {}", step["name"].as_str().unwrap_or("?")))
        .collect();
    assert!(
        masked.is_empty(),
        "steps carrying a continue-on-error key can report green without doing their job: {masked:?}"
    );
}

#[test]
fn test_pmat_093_every_cargo_publish_targets_a_workspace_member() {
    let members = workspace_member_names();
    let scripts = job_run_scripts(&publish_job());
    let mut publishes = 0;
    for line in scripts.lines().filter(|l| l.contains("cargo publish")) {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let pkg = tokens
            .iter()
            .position(|t| *t == "-p" || *t == "--package")
            .and_then(|i| tokens.get(i + 1))
            .unwrap_or_else(|| panic!("cargo publish without -p/--package: {line}"));
        assert!(
            members.iter().any(|m| m == pkg),
            "cargo publish names `{pkg}`, which is not a workspace member {members:?}"
        );
        publishes += 1;
    }
    assert!(
        publishes >= 2,
        "expected to publish ruchy and ruchy-wasm, found {publishes} publish steps"
    );
}

#[test]
fn test_pmat_093_publish_job_needs_the_build_jobs() {
    let needs: Vec<String> = publish_job()["needs"]
        .as_sequence()
        .expect("publish-crates.needs is a list")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    for job in ["create-release", "build-binaries", "build-wasm"] {
        assert!(
            needs.iter().any(|n| n == job),
            "publish-crates must need `{job}`; needs = {needs:?}"
        );
    }
}

#[test]
fn test_pmat_093_publish_uses_only_the_existing_registry_secret() {
    let job = serde_yaml::to_string(&publish_job()).expect("serialize publish job");
    assert!(
        job.contains("secrets.CARGO_REGISTRY_TOKEN"),
        "publish-crates must use secrets.CARGO_REGISTRY_TOKEN (the secret that exists)"
    );
    assert!(
        !job.contains("CRATES_TOKEN"),
        "secrets.CRATES_TOKEN does not exist in this repository"
    );
}

#[test]
fn test_pmat_093_prerelease_flag_is_derived_from_the_tag() {
    let steps = release_yml()["jobs"]["create-release"]["steps"].clone();
    let text = serde_yaml::to_string(&steps).expect("serialize create-release steps");
    assert!(
        text.contains("contains(github.ref_name, '-')"),
        "create-release must mark tags with a pre-release suffix (v5.0.0-beta.2) as prerelease"
    );
}

#[test]
fn test_pmat_093_required_status_checks_file_names_gate() {
    let path = repo_root().join(".github/required-status-checks.txt");
    let text = fs::read_to_string(&path).expect(".github/required-status-checks.txt exists");
    assert!(
        text.lines().any(|l| l.trim() == "gate"),
        "required-status-checks.txt must list the `gate` context the org ruleset requires"
    );
}
