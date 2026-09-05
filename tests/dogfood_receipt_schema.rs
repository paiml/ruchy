//! PMAT-096 — schema validation for the `make pre-release-gate` dogfood receipt.
//!
//! The receipt shape is fixed by the 5.0.0-beta.2 release plan §4. This test owns the
//! schema: `validate_receipt` is the single implementation, exercised against a
//! checked-in fixture, against mutated copies of it, and — when it exists — against the
//! receipt the gate actually produced.

use serde_json::Value;
use std::path::PathBuf;

const REQUIRED_TOP_LEVEL: [&str; 7] = [
    "schema_version",
    "version",
    "head",
    "baseline",
    "stages",
    "warns",
    "verdict",
];

const REQUIRED_STAGES: [&str; 8] = [
    "tests",
    "features",
    "verbs",
    "differential",
    "transpile",
    "clean_room",
    "package",
    "satd",
];

const VALID_STATUS: [&str; 3] = ["PASS", "WARN", "FAIL"];

/// Validate a dogfood receipt against the §4 schema.
///
/// Returns `Ok(())` when every required key is present, every stage carries a status in
/// {PASS,WARN,FAIL}, and the verdict is consistent (`go` iff no stage is `FAIL`).
fn validate_receipt(receipt: &Value) -> Result<(), String> {
    let root = receipt
        .as_object()
        .ok_or_else(|| "receipt is not a JSON object".to_string())?;

    for key in REQUIRED_TOP_LEVEL {
        if !root.contains_key(key) {
            return Err(format!("missing top-level key: {key}"));
        }
    }

    if root["schema_version"].as_u64() != Some(1) {
        return Err(format!(
            "schema_version must be 1, got {}",
            root["schema_version"]
        ));
    }
    if !root["warns"].is_array() {
        return Err("warns must be an array".to_string());
    }

    let stages = root["stages"]
        .as_object()
        .ok_or_else(|| "stages is not a JSON object".to_string())?;

    let mut any_fail = false;
    for name in REQUIRED_STAGES {
        let stage = stages
            .get(name)
            .ok_or_else(|| format!("missing stage: {name}"))?;
        let status = stage
            .get("status")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("stage {name} has no string status"))?;
        if !VALID_STATUS.contains(&status) {
            return Err(format!("stage {name} has invalid status: {status}"));
        }
        if status == "FAIL" {
            any_fail = true;
        }
    }

    let verdict = root["verdict"]
        .as_str()
        .ok_or_else(|| "verdict is not a string".to_string())?;
    match verdict {
        "go" if any_fail => Err("verdict is go but a stage is FAIL".to_string()),
        "no-go" if !any_fail => Err("verdict is no-go but no stage is FAIL".to_string()),
        "go" | "no-go" => Ok(()),
        other => Err(format!("verdict must be go or no-go, got {other}")),
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_fixture() -> Value {
    let path = repo_root().join("tests/fixtures/dogfood-receipt-example.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read fixture {}: {e}", path.display()));
    serde_json::from_str(&text).expect("fixture is not valid JSON")
}

/// Locate `docs/specifications/evidence/*-dogfood/receipt.json`, if the gate has run.
fn find_generated_receipt() -> Option<PathBuf> {
    let evidence = repo_root().join("docs/specifications/evidence");
    let mut found: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(evidence).ok()? {
        let dir = entry.ok()?.path();
        if dir.is_dir() && dir.file_name()?.to_str()?.ends_with("-dogfood") {
            let receipt = dir.join("receipt.json");
            if receipt.is_file() {
                found.push(receipt);
            }
        }
    }
    found.sort();
    found.pop()
}

#[test]
fn test_pmat_096_schema_accepts_example_fixture() {
    let receipt = load_fixture();
    assert_eq!(
        validate_receipt(&receipt),
        Ok(()),
        "the checked-in example receipt must satisfy the §4 schema"
    );
}

#[test]
fn test_pmat_096_schema_rejects_missing_clean_room_stage() {
    let mut receipt = load_fixture();
    receipt["stages"]
        .as_object_mut()
        .expect("stages object")
        .remove("clean_room")
        .expect("fixture must have a clean_room stage to remove");
    let err = validate_receipt(&receipt).expect_err("a receipt without clean_room is invalid");
    assert_eq!(err, "missing stage: clean_room");
}

#[test]
fn test_pmat_096_schema_rejects_missing_top_level_key() {
    for key in REQUIRED_TOP_LEVEL {
        let mut receipt = load_fixture();
        receipt.as_object_mut().expect("object").remove(key);
        let err = validate_receipt(&receipt)
            .expect_err("removing a required top-level key must invalidate the receipt");
        assert!(
            err.contains(key),
            "error for missing {key} should name it, got: {err}"
        );
    }
}

#[test]
fn test_pmat_096_schema_rejects_invalid_stage_status() {
    let mut receipt = load_fixture();
    receipt["stages"]["satd"]["status"] = Value::from("SKIPPED");
    let err = validate_receipt(&receipt).expect_err("SKIPPED is not a valid status");
    assert_eq!(err, "stage satd has invalid status: SKIPPED");
}

#[test]
fn test_pmat_096_verdict_go_with_a_failing_stage_is_rejected() {
    let mut receipt = load_fixture();
    receipt["stages"]["satd"]["status"] = Value::from("FAIL");
    receipt["stages"]["satd"]["count"] = Value::from(2);
    let err = validate_receipt(&receipt).expect_err("go + FAIL is inconsistent");
    assert_eq!(err, "verdict is go but a stage is FAIL");
}

#[test]
fn test_pmat_096_verdict_no_go_requires_a_failing_stage() {
    let mut receipt = load_fixture();
    receipt["verdict"] = Value::from("no-go");
    let err = validate_receipt(&receipt).expect_err("no-go without a FAIL is inconsistent");
    assert_eq!(err, "verdict is no-go but no stage is FAIL");
}

#[test]
fn test_pmat_096_verdict_no_go_with_a_failing_stage_is_accepted() {
    let mut receipt = load_fixture();
    receipt["stages"]["satd"]["status"] = Value::from("FAIL");
    receipt["stages"]["satd"]["count"] = Value::from(2);
    receipt["verdict"] = Value::from("no-go");
    assert_eq!(validate_receipt(&receipt), Ok(()));
}

#[test]
fn test_pmat_096_schema_accepts_generated_receipt() {
    let Some(path) = find_generated_receipt() else {
        eprintln!(
            "SKIP test_pmat_096_schema_accepts_generated_receipt: no \
             docs/specifications/evidence/*-dogfood/receipt.json present; \
             run `make pre-release-gate` to produce one"
        );
        return;
    };
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let receipt: Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("{} is not valid JSON: {e}", path.display()));
    assert_eq!(
        validate_receipt(&receipt),
        Ok(()),
        "gate-produced receipt {} violates the §4 schema",
        path.display()
    );
}
