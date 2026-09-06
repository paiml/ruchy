//! PMAT-093 / PMAT-135 — the release workflow must build and gate, never publish.
//!
//! PMAT-093 pinned a publish job that could only go green by publishing. PMAT-135
//! deletes the job outright: "CI held a publish credential" is the mechanism behind
//! the 403 that stopped the 5.0.0-beta.2 release, and "producer is never the gate" —
//! a workflow that gates AND publishes is one program attesting to itself. Manual
//! publish restores the separation: CI is the gate (build, clean-room, dogfood, tag,
//! prerelease); the operator is the publisher, from their machine, with a scoped
//! token. Consequence: no token in the repo, no crate publishing in any workflow.

use serde_yaml::Value;
use std::fs;
use std::path::PathBuf;

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

/// The `needs:` list of a job, as job ids.
fn job_needs(job: &Value) -> Vec<String> {
    job["needs"]
        .as_sequence()
        .expect("job needs is a list")
        .iter()
        .map(|v| v.as_str().expect("needs entry is a string").to_string())
        .collect()
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

/// PMAT-135: the release workflow holds no publish job, no publish command, and no
/// registry credential. The operator publishes; CI does not.
#[test]
fn test_pmat_135_release_workflow_has_no_publish_job() {
    let yml = release_yml();
    assert!(
        yml["jobs"]["publish-crates"].is_null(),
        "the publish-crates job must be gone: CI is the gate, the operator is the publisher"
    );
    let offenders: Vec<String> = release_yml_text()
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let trimmed = line.trim_start();
            !trimmed.starts_with('#') && trimmed.contains("cargo publish")
        })
        .map(|(i, line)| format!("release.yml:{}: {}", i + 1, line.trim()))
        .collect();
    assert!(
        offenders.is_empty(),
        "no workflow may push a crate to the registry: {offenders:?}"
    );
    assert!(
        !release_yml_text().contains("CARGO_REGISTRY_TOKEN"),
        "no registry credential may be referenced by the release workflow"
    );
}

/// PMAT-135: the policy gate runs last, after every artifact job, and runs the script.
#[test]
fn test_pmat_135_release_policy_job_gates_after_the_builds() {
    let yml = release_yml();
    let job = yml["jobs"]["release-policy"].clone();
    assert!(
        !job.is_null(),
        "release.yml must define a release-policy job that enforces the no-publish policy"
    );
    let needs = job_needs(&job);
    for required in ["create-release", "build-binaries", "build-wasm"] {
        assert!(
            needs.iter().any(|n| n == required),
            "release-policy must need `{required}`; needs = {needs:?}"
        );
    }
    let scripts = job_run_scripts(&job);
    assert!(
        scripts.contains("scripts/release-policy.sh --only no-publish-in-ci"),
        "release-policy must run only the gate whose subject is this tree (--only no-publish-in-ci); run scripts were:\n{scripts}"
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

/// PMAT-129: on wasm32, getrandom 0.3 (pulled by aprender-core's rand 0.9)
/// needs the `wasm_js` backend selected through a cfg flag; without it the
/// Build WASM job fails before `wasm-pack` gets to link anything.
#[test]
fn test_pmat_129_wasm_build_step_selects_the_getrandom_wasm_js_backend() {
    let workflow = std::fs::read_to_string(".github/workflows/release.yml")
        .expect("failed to read .github/workflows/release.yml");
    let lines: Vec<&str> = workflow.lines().collect();
    let step = lines
        .iter()
        .position(|line| line.contains("name: Build WASM package"))
        .expect("release.yml must have a Build WASM package step");
    let window = lines[step..lines.len().min(step + 8)].join("\n");
    assert!(
        window.contains("RUSTFLAGS") && window.contains("getrandom_backend=\"wasm_js\""),
        "the Build WASM package step must set RUSTFLAGS: --cfg getrandom_backend=\"wasm_js\"; got:\n{window}"
    );
}
