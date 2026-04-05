#![allow(missing_docs)]
//! Success Criterion #6: Binary size budget enforcement.
//!
//! Parent spec Section 10 requires the default-features `ruchy` binary to
//! stay under +20% of the 4.x baseline (12.4 MB → 14.88 MB hard limit).
//!
//! This test locates the release-mode binary produced by assert_cmd and
//! asserts its size falls under the budget. The test is `#[ignore]`'d in
//! debug mode (debug binaries carry symbols and are much larger).
//!
//! Ticket: [EMBED-009] Binary size budget gate.

use std::path::PathBuf;

/// 4.x baseline (parent spec Section 5).
const BASELINE_BYTES: u64 = 12_400_000;

/// Hard limit: baseline + 20%.
const BUDGET_BYTES: u64 = (BASELINE_BYTES as f64 * 1.20) as u64;

fn release_binary_path() -> Option<PathBuf> {
    // assert_cmd picks the binary matching the current build profile.
    // For this test we want the release build specifically.
    #[allow(deprecated)]
    let path: PathBuf = assert_cmd::cargo::cargo_bin("ruchy");
    let parent = path.parent()?;
    let profile_dir = parent.parent()?;
    let release = profile_dir.join("release").join("ruchy");
    if release.exists() {
        Some(release)
    } else if path.exists() && path.to_string_lossy().contains("/release/") {
        Some(path)
    } else {
        None
    }
}

#[test]
fn test_embed_009_binary_size_under_budget() {
    let Some(bin) = release_binary_path() else {
        eprintln!("release binary not found; skipping (debug-only test run)");
        return;
    };

    let size = std::fs::metadata(&bin)
        .expect("stat release binary must succeed")
        .len();

    println!(
        "release binary: {} bytes ({:.2} MB) — budget {} bytes ({:.2} MB)",
        size,
        size as f64 / 1_048_576.0,
        BUDGET_BYTES,
        BUDGET_BYTES as f64 / 1_048_576.0,
    );

    assert!(
        size < BUDGET_BYTES,
        "binary {} is {} bytes, exceeds budget of {} bytes (baseline 12.4 MB + 20%)",
        bin.display(),
        size,
        BUDGET_BYTES
    );
}
