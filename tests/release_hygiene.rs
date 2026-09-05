//! PMAT-092: Release package hygiene tests.
//!
//! Guards the crates.io package against regressions that broke Windows
//! checkouts (colon-containing paths) and bloated the published tarball
//! (quarantined test directories, docs/, .pmat-work/ scratch files).
//!
//! See docs/specifications/ruchy-5.0.0-beta.2-release-plan.md Section 2 (Z1).

use std::process::Command;

/// Run `git ls-files` in the repo root and return the tracked paths.
fn git_ls_files() -> Vec<String> {
    let output = Command::new("git")
        .arg("ls-files")
        .output()
        .expect("failed to run git ls-files");
    assert!(output.status.success(), "git ls-files failed");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(std::string::ToString::to_string)
        .collect()
}

#[test]
fn test_pmat_092_git_tracked_paths_no_colon() {
    let files = git_ls_files();
    let offenders: Vec<&String> = files.iter().filter(|p| p.contains(':')).collect();
    assert!(
        offenders.is_empty(),
        "tracked paths must not contain ':' (breaks Windows checkout): {offenders:?}"
    );
}

#[test]
fn test_pmat_092_quarantine_dirs_not_present() {
    let quarantine_dirs = [
        "tests.disabled",
        "tests_disabled_for_mutation",
        "tests_temp_disabled_for_sprint7_mutation",
    ];
    for dir in quarantine_dirs {
        assert!(
            !std::path::Path::new(dir).exists(),
            "quarantined test directory must not exist in worktree: {dir}"
        );
    }
}

#[test]
fn test_pmat_092_quarantine_dirs_not_tracked() {
    let files = git_ls_files();
    let quarantine_prefixes = [
        "tests.disabled/",
        "tests_disabled_for_mutation/",
        "tests_temp_disabled_for_sprint7_mutation/",
    ];
    let offenders: Vec<&String> = files
        .iter()
        .filter(|p| {
            quarantine_prefixes
                .iter()
                .any(|prefix| p.starts_with(prefix))
        })
        .collect();
    assert!(
        offenders.is_empty(),
        "quarantined test files must not be tracked by git: {offenders:?}"
    );
}

#[test]
fn test_pmat_092_cargo_toml_package_declares_include() {
    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("failed to read Cargo.toml");
    let value: toml::Table = cargo_toml
        .parse::<toml::Table>()
        .expect("failed to parse Cargo.toml");
    let package = value
        .get("package")
        .and_then(|p| p.as_table())
        .expect("Cargo.toml must have a [package] table");
    let include = package.get("include");
    assert!(
        include.is_some() && include.unwrap().is_array(),
        "[package] must declare an `include` allowlist (not just `exclude`)"
    );
}

/// Only the colon-containing `.pmat-work/` scratch directories (the ones that
/// break Windows checkout) are required to be untracked. Other `.pmat-work/`
/// paths (e.g. ticket-scoped receipts without a colon) are out of scope here;
/// they are excluded from the crates.io tarball simply by not appearing in
/// `[package] include`.
#[test]
fn test_pmat_092_pmat_work_colon_dirs_not_tracked() {
    let files = git_ls_files();
    let offenders: Vec<&String> = files
        .iter()
        .filter(|p| p.starts_with(".pmat-work/") && p.contains(':'))
        .collect();
    assert!(
        offenders.is_empty(),
        ".pmat-work/ colon-named scratch directories must not be tracked by git: {offenders:?}"
    );
}

#[test]
fn test_pmat_092_pmat_work_not_in_package_list() {
    let output = Command::new(env!("CARGO"))
        .args(["package", "--list", "-p", "ruchy"])
        .output()
        .expect("failed to run cargo package --list");
    assert!(
        output.status.success(),
        "cargo package --list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let files = String::from_utf8_lossy(&output.stdout);
    let offenders: Vec<&str> = files
        .lines()
        .filter(|l| l.starts_with(".pmat-work/"))
        .collect();
    assert!(
        offenders.is_empty(),
        ".pmat-work/ files must not be in the packaged file list: {offenders:?}"
    );
}

/// Cargo.lock is auto-included by cargo for packages that produce a binary
/// (`[[bin]]`), regardless of the `include` allowlist. Guard that this stays
/// true so `cargo install ruchy` remains reproducible.
#[test]
fn test_pmat_092_cargo_lock_present_in_package_list() {
    let output = Command::new(env!("CARGO"))
        .args(["package", "--list", "-p", "ruchy"])
        .output()
        .expect("failed to run cargo package --list");
    assert!(
        output.status.success(),
        "cargo package --list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let files = String::from_utf8_lossy(&output.stdout);
    assert!(
        files.lines().any(|l| l == "Cargo.lock"),
        "Cargo.lock must be present in the packaged file list"
    );
}
