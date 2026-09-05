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
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name != "feature_gate_lint.rs")
        })
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

/// Index of the first line of the statement that ends at `idx`: the line after
/// the nearest earlier line that closes a statement or block, or is blank.
fn statement_start(lines: &[&str], idx: usize) -> usize {
    (0..idx)
        .rev()
        .find(|&i| {
            let code = lines[i].split("//").next().unwrap_or_default().trim();
            code.is_empty() || code.ends_with(';') || code.ends_with('{') || code.ends_with('}')
        })
        .map_or(0, |i| i + 1)
}

/// A statement that ends in `.assert();` and neither binds nor otherwise
/// consumes the `assert_cmd::assert::Assert` it produces discards a
/// `#[must_use]` value: a clippy error under `-D warnings` once the target's
/// feature is on. Only `assert_cmd` chains count (mockito's `Mock::assert`
/// returns unit).
fn is_bare_assert_statement(lines: &[&str], idx: usize) -> bool {
    let last = lines[idx].split("//").next().unwrap_or_default().trim();
    if !last.ends_with(".assert();") {
        return false;
    }
    let statement = lines[statement_start(lines, idx)..=idx].join("\n");
    let from_assert_cmd = statement.contains("ruchy_cmd(")
        || statement.contains("cargo_bin")
        || statement.contains("Command::");
    from_assert_cmd && !statement.contains("let ") && !statement.contains(" = ")
}

fn bare_assert_statements(path: &Path, src: &str) -> Vec<String> {
    let lines: Vec<&str> = src.lines().collect();
    (0..lines.len())
        .filter(|&idx| is_bare_assert_statement(&lines, idx))
        .map(|idx| format!("{}:{}: {}", path.display(), idx + 1, lines[idx].trim()))
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
    let bare = [
        "fn t() {",
        "    ruchy_cmd()",
        "        .arg(\"x\")",
        "        .assert(); // times out",
        "}",
    ];
    assert!(is_bare_assert_statement(&bare, 3));
    let bound = [
        "fn t() {",
        "    let out = ruchy_cmd()",
        "        .arg(\"x\")",
        "        .assert();",
        "}",
    ];
    assert!(!is_bare_assert_statement(&bound, 3));
    let wrapped = [
        "    assert_args_accepted(",
        "        ruchy_cmd()",
        "            .assert(),",
        "    );",
    ];
    assert!(!is_bare_assert_statement(&wrapped, 2));
    let mockito = [
        "    let mock = server.mock(\"GET\", \"/\");",
        "    mock.assert();",
    ];
    assert!(!is_bare_assert_statement(&mockito, 1));
}

/// The `[[bin]]` block of the ruchy binary in the crate manifest.
fn ruchy_bin_block() -> String {
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("Cargo.toml is readable");
    manifest
        .split("[[bin]]")
        .skip(1)
        .map(|block| block.split("\n[").next().unwrap_or_default())
        .find(|block| block.contains("name = \"ruchy\""))
        .expect("Cargo.toml declares [[bin]] ruchy")
        .to_string()
}

/// `minimal` is the core library only; the binary is a REPL-driven CLI, so it
/// must say so instead of failing to compile under `--features minimal`.
#[test]
fn test_pmat_113_ruchy_binary_requires_the_repl_feature() {
    let block = ruchy_bin_block();
    let required = block
        .lines()
        .find(|line| line.trim_start().starts_with("required-features"))
        .unwrap_or_default();
    assert!(
        required.contains("\"repl\""),
        "[[bin]] ruchy must declare required-features = [\"repl\"]; found: {required:?}"
    );
}

/// And the default build must still produce the binary.
#[test]
fn test_pmat_113_default_features_enable_repl() {
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("Cargo.toml is readable");
    let batteries = manifest
        .lines()
        .find(|line| line.starts_with("batteries-included"))
        .expect("batteries-included feature");
    let default = manifest
        .lines()
        .find(|line| line.starts_with("default"))
        .expect("default feature");
    assert!(default.contains("\"batteries-included\""), "{default}");
    assert!(batteries.contains("\"repl\""), "{batteries}");
}
