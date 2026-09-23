//! RHL-5b tests: the unit contract of a job — its `expect` ensures, its effect
//! invariants, its terms and their contracts, `pv validate` on every v2 valid
//! program, and deterministic bytes.

use super::{unit_contract, unit_contract_yaml};
use crate::rhl::lower::LowerFailure;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    crate::rhl::repo_root()
}

const GX10: &str = "docs/rhl/breaks/v2/valid/01-gx10-disk-watch.rhl";

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn v2_valid() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(root().join("docs/rhl/breaks/v2/valid"))
        .expect("RHL-5b: v2 valid corpus exists")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "rhl"))
        .collect();
    files.sort();
    files
}

fn contract_of(source: &str) -> String {
    unit_contract("j.rhl", source, Some(&root()))
        .unwrap_or_else(|f| panic!("RHL-5b: expected a contract, got {f:?}"))
}

fn gx10() -> String {
    contract_of(&read(&root().join(GX10)))
}

fn gx10_with(from: &str, to: &str) -> String {
    let src = read(&root().join(GX10));
    assert!(src.contains(from), "GX10 has `{from}`");
    src.replace(from, to)
}

fn pv_available() -> bool {
    let ok = Command::new("pv").arg("--version").output().is_ok_and(|o| {
        o.status.success() && String::from_utf8_lossy(&o.stdout).contains("contracts")
    });
    // A host without `pv` skips these checks, printed. The release gate sets
    // RHL_REQUIRE_PV=1, and then a missing `pv` fails instead of skipping, so
    // the release never ships on a vacuous pass.
    assert!(
        ok || std::env::var_os("RHL_REQUIRE_PV").is_none(),
        "RHL_REQUIRE_PV is set but no provable-contracts `pv` is on PATH"
    );
    if !ok {
        println!("RHL-5b: skipped: no provable-contracts `pv` on PATH");
    }
    ok
}

/// `pv validate` and `pv status` on `contract`: validate's exit and status's stdout.
fn pv(contract: &str) -> (i32, String, String) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("j.contract.yaml");
    std::fs::write(&path, contract).expect("write contract");
    let validate = Command::new("pv")
        .arg("validate")
        .arg(&path)
        .output()
        .expect("pv validate runs");
    let status = Command::new("pv")
        .arg("status")
        .arg(&path)
        .output()
        .expect("pv status runs");
    let text = |b: &[u8]| String::from_utf8_lossy(b).to_string();
    (
        validate.status.code().unwrap_or(-1),
        text(&validate.stdout) + &text(&validate.stderr),
        text(&status.stdout),
    )
}

/// The `Proof obligations: N` count `pv status` prints.
fn obligations(status: &str) -> usize {
    status
        .lines()
        .find_map(|l| l.strip_prefix("Proof obligations: "))
        .and_then(|n| n.trim().parse().ok())
        .unwrap_or(0)
}

#[test]
fn test_rhl5b_contract_gx10_ensures_ticket_count_at_most_1() {
    let c = gx10();
    assert!(c.contains("  expect_0:\n"), "{c}");
    assert!(
        c.contains("postconditions:\n      - \"plan_ticket_count(plan) ≤ 1\"\n"),
        "{c}"
    );
    assert!(c.contains("type: postcondition"), "{c}");
    assert!(c.contains("expect ticket count is at most 1"), "{c}");
}

#[test]
fn test_rhl5b_contract_gx10_has_the_effect_invariants() {
    let c = gx10();
    assert!(c.contains("  effect_surface:\n"), "{c}");
    assert!(
        c.contains("effects(apply(plan)) ⊆ {read disk, write tickets}"),
        "{c}"
    );
    assert!(
        c.contains("      - \"may read disk\"\n      - \"may write tickets\"\n"),
        "{c}"
    );
}

#[test]
fn test_rhl5b_contract_gx10_metadata_names_unit_schedule_vocabularies_and_source() {
    let c = gx10();
    let sha = crate::rhl::receipt::sha256_hex(read(&root().join(GX10)).as_bytes());
    assert!(c.contains("    name: \"gx10 disk watch\"\n"), "{c}");
    assert!(c.contains(&format!("source_sha256: \"{sha}\"")), "{c}");
    assert!(c.contains("runs_on: \"host gx10\""), "{c}");
    assert!(c.contains("every: \"1 hour\""), "{c}");
    assert!(
        c.contains("    - name: \"fleet\"\n      version: 2\n"),
        "{c}"
    );
    assert!(
        c.contains("    - name: \"tickets\"\n      version: 2\n"),
        "{c}"
    );
}

#[test]
fn test_rhl5b_contract_gx10_lists_each_term_used_with_its_contract() {
    let c = gx10();
    for (term, path) in [
        ("disk free of", "contracts/rhl-fleet-disk-free-of-v2.yaml"),
        ("file ticket", "contracts/rhl-tickets-file-ticket-v2.yaml"),
        ("ticket count", "contracts/rhl-tickets-ticket-count-v2.yaml"),
        ("host", "contracts/rhl-fleet-host-v2.yaml"),
        ("GB", "contracts/rhl-fleet-gb-v2.yaml"),
        ("hour", "contracts/rhl-fleet-hour-v2.yaml"),
    ] {
        assert!(
            c.contains(&format!("    - term: \"{term}\"\n"))
                && c.contains(&format!("contract: \"{path}\"")),
            "`{term}` → {path}\n{c}"
        );
    }
    assert!(!c.contains("term: \"free\""), "a let name is no term\n{c}");
}

#[test]
fn test_rhl5b_contract_gx10_falsification_names_example_and_guard() {
    let c = gx10();
    assert!(
        c.contains("example_0 \\\"gx10 disk watch example\\\""),
        "{c}"
    );
    assert!(c.contains("test: \"expect_0"), "{c}");
}

#[test]
fn test_rhl5b_contract_gx10_validates_with_pv_and_has_obligations() {
    if !pv_available() {
        return;
    }
    let (exit, out, status) = pv(&gx10());
    assert_eq!(exit, 0, "pv validate:\n{out}");
    assert!(obligations(&status) > 0, "{status}");
}

#[test]
fn test_rhl5b_contract_every_v2_valid_program_validates_with_pv() {
    let pv_ok = pv_available();
    let files = v2_valid();
    assert_eq!(
        files.len(),
        12,
        "RHL-5b: the v2 valid corpus has 12 programs"
    );
    for f in files {
        let c = contract_of(&read(&f));
        if pv_ok {
            let (exit, out, status) = pv(&c);
            assert_eq!(exit, 0, "{}: pv validate:\n{out}\n{c}", f.display());
            assert!(obligations(&status) > 0, "{}: {status}", f.display());
        }
    }
}

#[test]
fn test_rhl5b_contract_without_expect_or_example_is_still_valid() {
    let src = gx10_with("  expect ticket count is at most 1\n", "");
    let src = src.replace(
        "  example \"gx10 disk watch example\"\n    given free is 90 GB\n    then ticket count is 1\n  end\n",
        "",
    );
    assert!(!src.contains("expect") && !src.contains("example"), "{src}");
    let c = contract_of(&src);
    assert!(!c.contains("expect_0"), "{c}");
    assert!(c.contains("effect_surface"), "{c}");
    if pv_available() {
        let (exit, out, status) = pv(&c);
        assert_eq!(exit, 0, "pv validate:\n{out}\n{c}");
        assert!(obligations(&status) > 0, "{status}");
    }
}

#[test]
fn test_rhl5b_contract_is_deterministic_and_tracks_the_source() {
    let a = gx10();
    assert_eq!(a, gx10(), "same inputs ⇒ byte-identical contract");
    assert!(!a.contains("created"), "no wall-clock field\n{a}");
    let other = contract_of(&gx10_with("is at most 1", "is at most 2"));
    assert_ne!(
        a, other,
        "positive control: another expect, another contract"
    );
    assert!(other.contains("plan_ticket_count(plan) ≤ 2"), "{other}");
}

#[test]
fn test_rhl5b_contract_of_yaml_surface_matches_the_rhl_tree() {
    let src = read(&root().join(GX10));
    let program = crate::rhl::parse(&src).expect("parses");
    let yaml = crate::rhl::yaml::to_yaml(&program);
    let from_yaml = unit_contract_yaml("j.rhl.yaml", &yaml, Some(&root())).expect("contract");
    let strip = |c: &str| {
        c.lines()
            .filter(|l| !l.contains("source_sha256"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(strip(&from_yaml), strip(&gx10()));
}

#[test]
fn test_rhl5b_contract_refuses_what_lowering_refuses() {
    let src = gx10_with("then ticket count is 1", "then free is 1 GB");
    match unit_contract("j.rhl", &src, Some(&root())) {
        Err(LowerFailure::Refused(d)) => assert_eq!(d.code, crate::rhl::codes::L001),
        other => panic!("RHL-5b: expected RHL-L001, got {other:?}"),
    }
}
