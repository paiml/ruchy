//! RHL-2: `ruchy fix --safe` and `ruchy vocab`, and RHL-3: `ruchy convert`
//! and `check`/`fmt` on `.rhl.yaml`, through the built binary (spec RHL-001
//! §5). Every decision is in the library and gated by
//! `cargo test --lib` (`src/rhl/fix.rs`, `src/rhl/vocab_cli.rs`); this file
//! shows what only a process can: the exit code, the file written in place,
//! and the root found from the working directory.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::{Path, PathBuf};

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("mkdir");
    for entry in std::fs::read_dir(from)
        .expect("read_dir")
        .filter_map(Result::ok)
    {
        if entry.path().is_file() {
            std::fs::copy(entry.path(), to.join(entry.file_name())).expect("copy");
        }
    }
}

/// A temporary vocabulary root (vocab/ and contracts/) holding a copy of `rel` as `x.rhl`.
fn rooted(rel: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("temp dir");
    for sub in ["vocab", "contracts"] {
        copy_dir(&repo().join(sub), &dir.path().join(sub));
    }
    let file = dir.path().join("x.rhl");
    std::fs::copy(repo().join(rel), &file).expect("copy");
    (dir, file)
}

const TYPO: &str = "docs/rhl/breaks/v2/planted/typo-term/03-gx12-runner-load-watch/broken.rhl";
const TYPO_BASE: &str = "docs/rhl/breaks/v2/valid/03-gx12-runner-load-watch.rhl";
const TWO: &str = "docs/rhl/breaks/v2/planted/two-candidate-typo/01-gx10-disk-watch/broken.rhl";

#[test]
fn test_rhl2_cli_fix_safe_rewrites_a_typo_file_and_exits_0() {
    let (_dir, file) = rooted(TYPO);
    ruchy_cmd()
        .arg("fix")
        .arg("--safe")
        .arg(&file)
        .assert()
        .code(0)
        .stdout(predicate::str::contains("fixed[RHL-V002]"));
    let fixed = std::fs::read_to_string(&file).expect("read");
    assert_eq!(
        fixed,
        std::fs::read_to_string(repo().join(TYPO_BASE)).expect("base")
    );
    ruchy_cmd().arg("check").arg(&file).assert().code(0);
    // Idempotent: the second run applies nothing and changes nothing.
    ruchy_cmd()
        .args(["fix", "--safe", "--format", "json"])
        .arg(&file)
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"applied\": []"));
    assert_eq!(std::fs::read_to_string(&file).expect("read"), fixed);
}

#[test]
fn test_rhl2_cli_fix_safe_leaves_a_two_candidate_file_unchanged() {
    let (_dir, file) = rooted(TWO);
    let before = std::fs::read_to_string(&file).expect("read");
    // V003 is the `Ambiguous` refusal: exit 2, as `ruchy check` gives it.
    ruchy_cmd()
        .args(["fix", "--safe"])
        .arg(&file)
        .assert()
        .code(2)
        .stdout(predicate::str::contains("error[RHL-V003]"));
    assert_eq!(std::fs::read_to_string(&file).expect("read"), before);
}

#[test]
fn test_rhl2_cli_fix_without_safe_is_refused() {
    let (_dir, file) = rooted(TYPO);
    let before = std::fs::read_to_string(&file).expect("read");
    ruchy_cmd()
        .arg("fix")
        .arg(&file)
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--safe"));
    assert_eq!(std::fs::read_to_string(&file).expect("read"), before);
}

#[test]
fn test_rhl2_cli_vocab_list_json_lists_the_four_vocabularies() {
    let out = ruchy_cmd()
        .current_dir(repo().join("docs/rhl"))
        .args(["vocab", "list", "--format", "json"])
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("JSON");
    let names: Vec<String> = v
        .as_array()
        .expect("array")
        .iter()
        .map(|r| format!("{} v{}", r["name"].as_str().unwrap_or(""), r["version"]))
        .collect();
    assert_eq!(names, ["fleet v1", "fleet v2", "tickets v1", "tickets v2"]);
}

#[test]
fn test_rhl2_cli_vocab_show_json_has_the_terms() {
    ruchy_cmd()
        .current_dir(repo())
        .args(["vocab", "show", "tickets", "v2", "--format", "json"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"term\": \"ticket count\""));
}

#[test]
fn test_rhl2_cli_vocab_validate_with_a_deleted_contract_exits_2_c001() {
    let (dir, _) = rooted(TYPO);
    std::fs::remove_file(dir.path().join("contracts/rhl-fleet-host-v2.yaml")).expect("rm");
    ruchy_cmd()
        .current_dir(dir.path())
        .args(["vocab", "validate", "fleet", "v2"])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("error[RHL-C001]"));
    ruchy_cmd()
        .current_dir(dir.path())
        .args(["vocab", "validate", "vocab/tickets-v2.yaml"])
        .assert()
        .code(0);
}

/// Convert `file` with the CLI into `out`, which must succeed.
fn convert(file: &Path, out: &Path) {
    ruchy_cmd()
        .arg("convert")
        .arg(file)
        .arg("-o")
        .arg(out)
        .assert()
        .code(0)
        .stdout("");
}

#[test]
fn test_rhl3_cli_convert_round_trips_both_directions() {
    let (dir, file) = rooted(TYPO_BASE);
    let yaml = dir.path().join("x.rhl.yaml");
    convert(&file, &yaml);
    let text = std::fs::read_to_string(&yaml).expect("yaml written");
    assert!(text.starts_with("rhl: 1\n"), "{text}");
    ruchy_cmd()
        .arg("convert")
        .arg(&yaml)
        .assert()
        .code(0)
        .stdout(std::fs::read_to_string(&file).expect("base"));
    ruchy_cmd()
        .args(["convert", "--to", "yaml"])
        .arg(&file)
        .assert()
        .code(0)
        .stdout(text);
}

#[test]
fn test_rhl3_cli_convert_malformed_yaml_is_p001_exit_2() {
    let dir = tempfile::tempdir().expect("temp dir");
    let bad = dir.path().join("x.rhl.yaml");
    std::fs::write(&bad, "rhl: 1\ndecls: [\n").expect("write");
    ruchy_cmd()
        .arg("convert")
        .arg(&bad)
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains("error[RHL-P004]"));
}

fn check_json(file: &Path) -> (serde_json::Value, Option<i32>) {
    let out = ruchy_cmd()
        .args(["check", "--format", "json"])
        .arg(file)
        .output()
        .expect("runs");
    let v = serde_json::from_slice(&out.stdout).expect("JSON report");
    (v, out.status.code())
}

#[test]
fn test_rhl3_cli_check_on_yaml_reports_the_typo_like_the_rhl_form() {
    let (dir, file) = rooted(TYPO);
    let yaml = dir.path().join("x.rhl.yaml");
    convert(&file, &yaml);
    let (from_rhl, rhl_exit) = check_json(&file);
    let (from_yaml, yaml_exit) = check_json(&yaml);
    assert_eq!(from_yaml["diagnostics"][0]["code"], "RHL-V002");
    assert_eq!(
        from_yaml["diagnostics"][0]["code"],
        from_rhl["diagnostics"][0]["code"]
    );
    assert_eq!(
        from_yaml["diagnostics"][0]["candidates"],
        from_rhl["diagnostics"][0]["candidates"]
    );
    assert_eq!(from_yaml["verdict"], "refused");
    assert_eq!((yaml_exit, rhl_exit), (Some(2), Some(2)));
}

#[test]
fn test_rhl3_cli_fmt_on_yaml_prints_canonical_yaml() {
    let (dir, file) = rooted(TYPO_BASE);
    let yaml = dir.path().join("x.rhl.yaml");
    convert(&file, &yaml);
    let canonical = std::fs::read_to_string(&yaml).expect("yaml");
    std::fs::write(&yaml, canonical.replace("rhl: 1\n", "") + "rhl: 1\n").expect("reorder");
    ruchy_cmd()
        .args(["fmt", "--stdout"])
        .arg(&yaml)
        .assert()
        .code(0)
        .stdout(predicate::str::contains(canonical));
}
