#![allow(missing_docs)]
//! Feature-gate lint for the integration test tree (PMAT-112, PMAT-113).
//!
//! The gate is `cargo check --tests --no-default-features --features minimal`
//! together with `cargo clippy --all-targets --all-features -- -D warnings`.
//! These tests are the fast discriminator: they name the offending file and
//! line in milliseconds instead of after a full feature-matrix build.

use std::fs;
use std::path::{Path, PathBuf};

const REPL_GATE: &str = "#[cfg(feature = \"repl\")]";
const FILE_GATE_PREFIX: &str = "#![cfg(feature = ";

fn test_sources() -> Vec<(PathBuf, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("tests/ directory is readable")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|path| {
            let src = fs::read_to_string(&path).expect("test source is readable");
            (path, src)
        })
        .collect()
}

/// A line that pulls the REPL into the test target.
fn line_references_repl(line: &str) -> bool {
    line.contains("runtime::Repl")
        || line.contains("runtime::repl")
        || (line.contains("runtime::{") && line.contains("Repl"))
}

/// A crate-level `#![cfg(feature = "...")]` keeps the whole target out of
/// the `minimal` build, whichever feature it names.
fn has_file_level_feature_gate(src: &str) -> bool {
    src.lines().any(|line| line.starts_with(FILE_GATE_PREFIX))
}

/// An item-level `#[cfg(feature = "repl")]` within the four lines above the
/// use (attribute, `#[test]`, `fn` header, then the `use` inside the body).
fn item_gated_above(lines: &[&str], idx: usize) -> bool {
    let start = idx.saturating_sub(4);
    lines[start..idx]
        .iter()
        .any(|line| line.trim() == REPL_GATE)
}

fn ungated_repl_uses(path: &Path, src: &str) -> Vec<String> {
    if has_file_level_feature_gate(src) {
        return Vec::new();
    }
    let lines: Vec<&str> = src.lines().collect();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line_references_repl(line))
        .filter(|(idx, _)| !item_gated_above(&lines, *idx))
        .map(|(idx, line)| format!("{}:{}: {}", path.display(), idx + 1, line.trim()))
        .collect()
}

/// `....assert();` as a bare statement discards a `#[must_use]` value, which
/// is a clippy error under `-D warnings` once the target's feature is on.
fn is_bare_assert_statement(line: &str) -> bool {
    let code = line.split("//").next().unwrap_or_default().trim();
    code.ends_with(".assert();") && !code.contains("let ") && !code.contains('=')
}

fn bare_assert_statements(path: &Path, src: &str) -> Vec<String> {
    src.lines()
        .enumerate()
        .filter(|(_, line)| is_bare_assert_statement(line))
        .map(|(idx, line)| format!("{}:{}: {}", path.display(), idx + 1, line.trim()))
        .collect()
}

#[test]
fn test_pmat_113_repl_tests_are_feature_gated() {
    let violations: Vec<String> = test_sources()
        .iter()
        .flat_map(|(path, src)| ungated_repl_uses(path, src))
        .collect();
    assert!(
        violations.is_empty(),
        "REPL-dependent tests must carry `{REPL_GATE}` (crate-level `#!` form or on the item) \
         so that `--no-default-features --features minimal` compiles:\n{}",
        violations.join("\n")
    );
}

#[test]
fn test_pmat_112_no_bare_assert_statements_in_tests() {
    let violations: Vec<String> = test_sources()
        .iter()
        .flat_map(|(path, src)| bare_assert_statements(path, src))
        .collect();
    assert!(
        violations.is_empty(),
        "bare `.assert();` statements discard a must_use value; wrap them in \
         `support::assert_args_accepted(..)` or assert on the result:\n{}",
        violations.join("\n")
    );
}

#[test]
fn test_pmat_113_lint_discriminates_gated_from_ungated_repl_uses() {
    let path = Path::new("x.rs");
    let ungated = "use ruchy::runtime::Repl;\n#[test]\nfn t() {}\n";
    assert_eq!(ungated_repl_uses(path, ungated).len(), 1);
    let file_gated = format!("#![cfg(feature = \"repl\")]\n{ungated}");
    assert!(ungated_repl_uses(path, &file_gated).is_empty());
    let item_gated =
        "#[cfg(feature = \"repl\")]\n#[test]\nfn t() {\n    use ruchy::runtime::repl::state::ReplState;\n}\n";
    assert!(ungated_repl_uses(path, item_gated).is_empty());
    let too_far = "#[cfg(feature = \"repl\")]\n\n\n\n\nuse ruchy::runtime::Repl;\n";
    assert_eq!(ungated_repl_uses(path, too_far).len(), 1);
}

#[test]
fn test_pmat_112_lint_discriminates_bare_from_consumed_asserts() {
    assert!(is_bare_assert_statement("        .assert(); // times out"));
    assert!(is_bare_assert_statement("    cmd.assert();"));
    assert!(!is_bare_assert_statement("            .assert(),"));
    assert!(!is_bare_assert_statement("    let out = cmd.assert();"));
    assert!(!is_bare_assert_statement("        .assert()"));
}
