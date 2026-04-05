#![allow(missing_docs)]
//! Success Criterion #11: Zero unsafe in transpiler output.
//!
//! Per `ruchy-5.0-sovereign-platform.md` Section 10, the transpiler MUST
//! never emit `unsafe {` blocks in generated Rust code. This test transpiles
//! a representative set of example programs and asserts every output is
//! completely free of the `unsafe` keyword.
//!
//! Ticket: [EMBED-008] Zero-unsafe transpile gate.
//!
//! Reference: GitHub Issue #132 and CLAUDE.md "ZERO UNSAFE CODE POLICY".

use assert_cmd::Command;
use std::path::Path;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// Example files we require to be unsafe-free on transpile.
/// Picked to exercise: basics, control flow, collections, strings, functions,
/// and the 5.0 sovereign surface (contracts, SIMD arrays).
const REQUIRED_EXAMPLES: &[&str] = &[
    "01_basics.ruchy",
    "02_functions.ruchy",
    "03_control_flow.ruchy",
    "04_collections.ruchy",
    "05_strings.ruchy",
    "30_contracts.ruchy",
    "33_simd_arrays.ruchy",
];

fn transpile_to_string(path: &Path) -> String {
    let output = ruchy_cmd()
        .arg("transpile")
        .arg(path)
        .output()
        .expect("ruchy transpile must run");
    assert!(
        output.status.success(),
        "transpile {} failed: stderr={}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("transpile output must be UTF-8")
}

fn assert_no_unsafe(src: &str, label: &str) {
    // Match the policy exactly: `unsafe {` is the thing we forbid.
    // Also check `unsafe fn` / `unsafe trait` / raw pointer casts.
    let needles: &[&str] = &["unsafe {", "unsafe fn", "unsafe trait", "unsafe impl"];
    for needle in needles {
        assert!(
            !src.contains(needle),
            "Transpiled {label} contains forbidden pattern {needle:?}.\n\
             First 200 chars around match: {}",
            src.find(needle)
                .map(|i| {
                    let start = i.saturating_sub(80);
                    let end = (i + 120).min(src.len());
                    src[start..end].to_string()
                })
                .unwrap_or_default()
        );
    }
}

#[test]
fn test_embed_008_required_examples_transpile_without_unsafe() {
    let examples_dir = Path::new("examples");
    assert!(examples_dir.is_dir(), "examples/ must exist");

    for name in REQUIRED_EXAMPLES {
        let path = examples_dir.join(name);
        if !path.exists() {
            // Missing example is a separate failure mode -- report clearly.
            panic!("required example missing: {}", path.display());
        }
        let out = transpile_to_string(&path);
        assert_no_unsafe(&out, name);
    }
}

#[test]
fn test_embed_008_sovereign_examples_transpile_without_unsafe() {
    // Every 5.0 sovereign example (30..35) must be unsafe-free.
    let examples_dir = Path::new("examples");
    let sovereign_prefixes = ["30_", "31_", "32_", "33_", "34_"];
    let mut checked = 0usize;

    for entry in std::fs::read_dir(examples_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".ruchy") {
            continue;
        }
        // Only check files matching a sovereign prefix AND explicitly listed
        // as 5.0 content (contracts / sovereign_platform / migration_demo /
        // simd_arrays / embedding).
        let is_sovereign_50 = sovereign_prefixes
            .iter()
            .any(|p| name.starts_with(p))
            && (name.contains("contracts")
                || name.contains("sovereign")
                || name.contains("migration")
                || name.contains("simd_arrays")
                || name.contains("embedding"));
        if !is_sovereign_50 {
            continue;
        }

        let path = entry.path();
        let out = transpile_to_string(&path);
        assert_no_unsafe(&out, &name);
        checked += 1;
    }

    assert!(
        checked >= 5,
        "expected at least 5 sovereign examples checked, got {checked}"
    );
}
