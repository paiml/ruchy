//! PMAT-095: EXAMPLES-MIGRATE-4TO5
//!
//! Ruchy 5.0 reserves 7 new keywords (requires ensures invariant decreases
//! infra signal yield). This test suite proves every file listed in
//! tests/fixtures/examples_manifest.txt parses successfully (`ruchy check`)
//! against the crate's own binary. The manifest is derived from a
//! differential `ruchy check` run of the 4.2.1 baseline vs HEAD
//! (docs/specifications/evidence/2026-09-05-release-gather/differential-check-4.2.1-vs-head.csv):
//! every `examples/**/*.ruchy` file where `head_exit == 0`, plus
//! `examples/24_math_science.ruchy` (which regressed at HEAD because it uses
//! a now-reserved keyword as an identifier and must be migrated).

use assert_cmd::Command;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::Duration;

const MANIFEST_PATH: &str = "tests/fixtures/examples_manifest.txt";
const CHECK_TIMEOUT_SECS: u64 = 10;

/// Helper to get the crate's own `ruchy` binary command.
fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// Read and parse the manifest file into a list of non-empty, trimmed lines.
fn read_manifest() -> Vec<String> {
    let contents = fs::read_to_string(MANIFEST_PATH)
        .unwrap_or_else(|e| panic!("failed to read {MANIFEST_PATH}: {e}"));
    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

/// Run `ruchy check <file>` under a timeout and assert success.
fn assert_check_succeeds(file_path: &str) {
    assert!(
        Path::new(file_path).exists(),
        "manifest entry not found on disk: {file_path}"
    );

    ruchy_cmd()
        .arg("check")
        .arg(file_path)
        .timeout(Duration::from_secs(CHECK_TIMEOUT_SECS))
        .assert()
        .success();
}

#[test]
fn test_pmat_095_manifest_no_duplicates_and_files_exist() {
    let manifest = read_manifest();
    assert!(!manifest.is_empty(), "manifest must not be empty");

    let unique: HashSet<&String> = manifest.iter().collect();
    assert_eq!(
        unique.len(),
        manifest.len(),
        "manifest contains duplicate entries"
    );

    for file_path in &manifest {
        assert!(
            Path::new(file_path).exists(),
            "manifest entry not found on disk: {file_path}"
        );
    }
}

#[test]
fn test_pmat_095_all_manifest_examples_parse() {
    let manifest = read_manifest();
    let mut failures = Vec::new();

    for file_path in &manifest {
        let result = std::panic::catch_unwind(|| assert_check_succeeds(file_path));
        if result.is_err() {
            failures.push(file_path.clone());
        }
    }

    assert!(
        failures.is_empty(),
        "the following manifest examples failed `ruchy check`: {failures:?}"
    );
}

#[test]
fn test_pmat_095_math_science_parses_post_migration() {
    assert_check_succeeds("examples/24_math_science.ruchy");
}
