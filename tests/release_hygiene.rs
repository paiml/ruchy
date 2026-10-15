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

/// `git status --porcelain` lines (tracked changes and untracked, non-ignored files).
fn dirty_paths() -> Vec<String> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .expect("failed to run git status");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(std::string::ToString::to_string)
        .collect()
}

/// The failure message for a failed `cargo package --list`. `cargo package`
/// refuses any dirty tree, so a dirty tree is named as the cause explicitly
/// instead of surfacing only cargo's raw error (G2B-S2). `--allow-dirty` is
/// deliberately not used: the check is about what a clean release would ship.
fn package_failure_message(dirty: &[String], cargo_stderr: &str) -> String {
    if dirty.is_empty() {
        return format!("cargo package --list failed: {cargo_stderr}");
    }
    format!(
        "working tree is dirty: cargo package refuses to run until these are committed \
         or ignored:\n{}\ncargo said: {cargo_stderr}",
        dirty.join("\n")
    )
}

/// `cargo package --list -p ruchy` stdout; fails naming a dirty tree when that is the cause.
fn cargo_package_list() -> String {
    let output = Command::new(env!("CARGO"))
        .args(["package", "--list", "-p", "ruchy"])
        .output()
        .expect("failed to run cargo package --list");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "{}",
        package_failure_message(&dirty_paths(), &stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn test_g2b_s2_package_failure_names_a_dirty_tree() {
    let msg = package_failure_message(&["?? src/.pmat/tdg-cold.db".to_string()], "error: x");
    assert!(msg.starts_with("working tree is dirty"), "{msg}");
    assert!(msg.contains("src/.pmat/tdg-cold.db"), "{msg}");
    assert!(msg.contains("error: x"), "{msg}");
    let clean = package_failure_message(&[], "error: y");
    assert!(!clean.contains("dirty"), "{clean}");
}

/// Paths pmat writes while tests and hooks run must be ignored by the ROOT
/// `.gitignore`, or `cargo package` sees a dirty tree (G2B-S2). pmat also drops
/// a `*` `.gitignore` inside each `.pmat/` it creates, but that file is itself
/// untracked and absent in a fresh clone, so it is checked in isolation here: a
/// scratch repository holding only the root `.gitignore`.
#[test]
fn test_g2b_s2_tool_written_paths_are_ignored() {
    let scratch = tempfile::TempDir::new().expect("tempdir");
    std::fs::copy(".gitignore", scratch.path().join(".gitignore")).expect("copy .gitignore");
    let git = |args: &[&str]| {
        Command::new("git")
            .current_dir(scratch.path())
            .args(args)
            .status()
            .expect("failed to run git")
    };
    assert!(git(&["init", "-q"]).success(), "git init failed");
    for path in [
        "src/.pmat/tdg-cold.db",
        "src/.pmat/tdg-warm.db",
        ".pmat/tdg-cold.db",
        ".pmat/jidoka.jsonl",
        ".pmat/clippy-receipt",
        ".pmat/dead-code-cache-eb-d8.json",
    ] {
        let ignored = git(&["check-ignore", "--no-index", "-q", path]).success();
        assert!(ignored, "{path} must be ignored by the root .gitignore");
    }
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
    let files = cargo_package_list();
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
    let files = cargo_package_list();
    assert!(
        files.lines().any(|l| l == "Cargo.lock"),
        "Cargo.lock must be present in the packaged file list"
    );
}

/// PMAT-127: proptest writes `*.proptest-regressions` files next to failing
/// property tests. They are gitignored, five old ones are tracked, and
/// `tests/**` would ship all of them; the allowlist must negate them so the
/// crate never carries test-run artifacts.
#[test]
fn test_pmat_127_include_negates_proptest_regressions() {
    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("failed to read Cargo.toml");
    let value: toml::Table = cargo_toml
        .parse::<toml::Table>()
        .expect("failed to parse Cargo.toml");
    let include = value
        .get("package")
        .and_then(|p| p.get("include"))
        .and_then(|i| i.as_array())
        .expect("[package] include allowlist");
    let negates = include
        .iter()
        .filter_map(|v| v.as_str())
        .any(|pattern| pattern == "!**/*.proptest-regressions");
    assert!(
        negates,
        "[package] include must contain \"!**/*.proptest-regressions\"; got {include:?}"
    );
}
