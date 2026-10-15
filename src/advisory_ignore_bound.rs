//! The RUSTSEC-2024-0449 exemption is BOUNDED (`SEC-4`, PMAT-260; removal is PMAT-263).
//!
//! `unsafe-libyaml` is unmaintained and has no fixed release. SEC-2 removed every
//! ruchy runtime edge to it, but it stays in `Cargo.lock` through published
//! crates ruchy cannot change: `serde_yaml_ng` is a non-optional build-dependency
//! of `aprender-compute` and `aprender-core` (through 0.71.4), and `serde_yaml`
//! arrives through `provable-contracts`. `deny.toml` therefore ignores the id,
//! and `docs/audits/advisory-ignores.yaml` dates it (`advisory_ignore_gate`).
//!
//! A date bounds the ignore in TIME. This bounds it in REACH, so the id cannot
//! quietly cover anything nobody reviewed:
//!
//!   1. the ignore exists only while `unsafe-libyaml` is in `Cargo.lock` — the
//!      day the crate leaves, this goes red and names the line to delete;
//!   2. only `serde_yaml` and `serde_yaml_ng` link `unsafe-libyaml`;
//!   3. only the reviewed crates below depend on those two — a new consumer is
//!      a new review, not a silent widening;
//!   4. ruchy itself reaches them through `[dev-dependencies]` only (the SEC-2
//!      differential-test oracle), never a normal, build or target dependency.
//!
//! `#[cfg(test)]` in `src/` for the same reason as `advisory_ignore_gate`: the
//! required check runs `cargo test --lib`.

use std::collections::BTreeSet;
use std::path::PathBuf;

const ADVISORY: &str = "RUSTSEC-2024-0449";
const CRATE: &str = "unsafe-libyaml";
const LINKERS: &[&str] = &["serde_yaml", "serde_yaml_ng"];

/// Every crate in `Cargo.lock` that depends on `serde_yaml` or `serde_yaml_ng`,
/// as reviewed for PMAT-260 on 2026-10-15. The set is read from the lock, which
/// includes optional-feature edges; widening it is a reviewed edit.
const REVIEWED_CONSUMERS: &[&str] = &[
    "alimentar",
    "aprender-compute",
    "aprender-core",
    "aprender-present-core",
    "aprender-rag",
    "aprender-simulate",
    "aprender-train",
    "bashrs",
    "forjar",
    "forjar-contracts",
    "jugar-probar",
    "presentar-yaml",
    "provable-contracts",
    "ruchy",
];

fn read_toml(rel: &str) -> toml::Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("SEC-4: cannot read {}: {e}", path.display()));
    toml::from_str(&text).unwrap_or_else(|e| panic!("SEC-4: {rel} is not valid TOML: {e}"))
}

/// `(name, dependency names)` for every `[[package]]` in `Cargo.lock`.
fn lock_packages() -> Vec<(String, Vec<String>)> {
    let lock = read_toml("Cargo.lock");
    let pkgs = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .expect("SEC-4: Cargo.lock has no [[package]] array");
    assert!(!pkgs.is_empty(), "SEC-4: Cargo.lock lists zero packages");
    pkgs.iter()
        .map(|p| {
            let name = p
                .get("name")
                .and_then(toml::Value::as_str)
                .unwrap_or_default();
            let deps = p
                .get("dependencies")
                .and_then(toml::Value::as_array)
                .map(|d| {
                    d.iter()
                        .filter_map(toml::Value::as_str)
                        // "name", "name version" or "name version (source)"
                        .filter_map(|s| s.split_whitespace().next())
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default();
            (name.to_string(), deps)
        })
        .collect()
}

fn dependents_of(pkgs: &[(String, Vec<String>)], targets: &[&str]) -> BTreeSet<String> {
    pkgs.iter()
        .filter(|(_, deps)| deps.iter().any(|d| targets.contains(&d.as_str())))
        .map(|(n, _)| n.clone())
        .collect()
}

fn deny_ignores_advisory() -> bool {
    read_toml("deny.toml")
        .get("advisories")
        .and_then(|a| a.get("ignore"))
        .and_then(toml::Value::as_array)
        .is_some_and(|rows| {
            rows.iter()
                .filter_map(|r| r.as_str().or_else(|| r.get("id")?.as_str()))
                .any(|id| id == ADVISORY)
        })
}

#[test]
fn test_sec_4_ignore_lives_exactly_while_the_crate_is_locked() {
    let locked = lock_packages().iter().any(|(n, _)| n == CRATE);
    let ignored = deny_ignores_advisory();
    assert!(
        !(ignored && !locked),
        "SEC-4: deny.toml ignores {ADVISORY} but {CRATE} is no longer in Cargo.lock. \
         Delete the ignore and its docs/audits/advisory-ignores.yaml entry (PMAT-263)."
    );
}

#[test]
fn test_sec_4_only_the_two_yaml_crates_link_unsafe_libyaml() {
    if !deny_ignores_advisory() {
        return;
    }
    let pkgs = lock_packages();
    let linkers = dependents_of(&pkgs, &[CRATE]);
    let unexpected: Vec<&String> = linkers
        .iter()
        .filter(|n| !LINKERS.contains(&n.as_str()))
        .collect();
    assert!(
        unexpected.is_empty(),
        "SEC-4: {ADVISORY} is ignored, and {unexpected:?} now link {CRATE} directly. \
         The ignore was reviewed for {LINKERS:?} only."
    );
}

#[test]
fn test_sec_4_no_unreviewed_crate_reaches_the_yaml_crates() {
    if !deny_ignores_advisory() {
        return;
    }
    let pkgs = lock_packages();
    let consumers = dependents_of(&pkgs, LINKERS);
    assert!(
        !consumers.is_empty(),
        "SEC-4: no consumer of {LINKERS:?} found: the lock reader read nothing"
    );
    let unreviewed: Vec<&String> = consumers
        .iter()
        .filter(|n| !REVIEWED_CONSUMERS.contains(&n.as_str()))
        .collect();
    assert!(
        unreviewed.is_empty(),
        "SEC-4: {ADVISORY} is ignored, and {unreviewed:?} newly depend on {LINKERS:?}. \
         A new path to {CRATE} is a new review: add it to REVIEWED_CONSUMERS only after one."
    );
}

/// Every dependency table of ruchy's own manifest except `[dev-dependencies]`.
fn non_dev_dep_names(manifest: &toml::Value) -> Vec<String> {
    let mut tables: Vec<&toml::Value> = ["dependencies", "build-dependencies"]
        .iter()
        .filter_map(|k| manifest.get(*k))
        .collect();
    if let Some(targets) = manifest.get("target").and_then(toml::Value::as_table) {
        for t in targets.values() {
            for k in ["dependencies", "build-dependencies"] {
                if let Some(v) = t.get(k) {
                    tables.push(v);
                }
            }
        }
    }
    tables
        .iter()
        .filter_map(|t| t.as_table())
        .flat_map(|t| {
            t.iter().map(|(key, spec)| {
                // `alias = { package = "real-name", .. }` renames: judge the real name.
                spec.get("package")
                    .and_then(toml::Value::as_str)
                    .unwrap_or(key)
                    .to_string()
            })
        })
        .collect()
}

#[test]
fn test_sec_4_ruchy_reaches_the_yaml_crates_through_dev_dependencies_only() {
    let manifest = read_toml("Cargo.toml");
    let names = non_dev_dep_names(&manifest);
    assert!(
        !names.is_empty(),
        "SEC-4: read zero non-dev dependencies from Cargo.toml"
    );
    let bad: Vec<&String> = names
        .iter()
        .filter(|n| LINKERS.contains(&n.as_str()) || n.as_str() == CRATE)
        .collect();
    assert!(
        bad.is_empty(),
        "SEC-4: Cargo.toml has a non-dev dependency on {bad:?}. SEC-2 moved every \
         YAML read and write to serde-saphyr (src/yaml_io.rs); the {ADVISORY} ignore \
         assumes ruchy ships no unsafe-libyaml edge of its own."
    );
}
