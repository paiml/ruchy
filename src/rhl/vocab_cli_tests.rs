//! RHL-2 `ruchy vocab` tests: list, show, validate, and one test per rule of
//! the vocabulary shape (`contracts/rhl-vocabulary-shape-v1.yaml`).

use super::*;
use crate::rhl::check::check;
use std::path::Path;

fn root() -> PathBuf {
    crate::rhl::repo_root()
}

/// A root holding one vocabulary `demo v1` with `terms` (YAML list items)
/// and a contract file `contracts/c.yaml`.
fn shaped(terms: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("vocab")).expect("mkdir");
    std::fs::create_dir_all(tmp.path().join("contracts")).expect("mkdir");
    std::fs::write(tmp.path().join("contracts/c.yaml"), "x: 1\n").expect("write");
    let yaml = format!("vocabulary: demo\nversion: 1\nterms:\n{terms}");
    std::fs::write(tmp.path().join("vocab/demo-v1.yaml"), yaml).expect("write");
    tmp
}

const GOOD: &str = "  - {term: disk free of, kind: measure, takes: [{name: path, type: Text}], gives: Size, effect: read disk, contract: contracts/c.yaml}\n  - {term: GB, kind: unit, gives: Size, contract: contracts/c.yaml}\n  - {term: host, kind: entity, instances_from: machines/*.yaml, contract: contracts/c.yaml}\n  - {term: file ticket, kind: action, attributes: [{name: title, type: Text}], contract: contracts/c.yaml}\n";

fn codes_of(tmp: &tempfile::TempDir) -> Vec<String> {
    let v = validate(&tmp.path().join("vocab/demo-v1.yaml"), tmp.path());
    v.diagnostics.iter().map(|f| f.code.clone()).collect()
}

// ---------------------------------------------------------------- the shape, one test per rule

#[test]
fn test_rhl2_shape_a_well_formed_vocabulary_passes() {
    let tmp = shaped(GOOD);
    let v = validate(&tmp.path().join("vocab/demo-v1.yaml"), tmp.path());
    assert_eq!(v.diagnostics, Vec::<Finding>::new());
    assert_eq!((v.verdict, v.exit_code()), (Verdict::Pass, 0));
    assert_eq!(v.vocabulary.as_deref(), Some("demo v1"));
}

#[test]
fn test_rhl2_shape_every_pre_registered_vocabulary_passes() {
    for (name, version) in [("fleet", 1), ("fleet", 2), ("tickets", 1), ("tickets", 2)] {
        let v = validate(&vocab::vocab_path(&root(), name, version), &root());
        assert_eq!(v.diagnostics, Vec::<Finding>::new(), "{name} v{version}");
    }
}

#[test]
fn test_rhl2_shape_missing_field_is_v004() {
    let tmp = shaped("  - {kind: noun, gives: Text, contract: contracts/c.yaml}\n");
    assert_eq!(codes_of(&tmp), vec![codes::V004]);
}

#[test]
fn test_rhl2_shape_unknown_kind_is_v004() {
    let tmp = shaped("  - {term: x, kind: verb, contract: contracts/c.yaml}\n");
    assert_eq!(codes_of(&tmp), vec![codes::V004]);
}

#[test]
fn test_rhl2_shape_term_without_existing_contract_is_c001() {
    let missing = shaped("  - {term: x, kind: noun, gives: Text, contract: contracts/gone.yaml}\n");
    assert_eq!(codes_of(&missing), vec![codes::C001]);
    let omitted = shaped("  - {term: x, kind: noun, gives: Text}\n");
    let v = validate(&omitted.path().join("vocab/demo-v1.yaml"), omitted.path());
    assert_eq!(v.diagnostics[0].code, codes::C001);
    assert_eq!(v.diagnostics[0].term.as_deref(), Some("x"));
    assert_eq!((v.verdict, v.exit_code()), (Verdict::Refused, 2));
}

#[test]
fn test_rhl2_shape_untyped_attribute_is_v004() {
    let no_type = shaped(
        "  - {term: a, kind: action, attributes: [{name: title}], contract: contracts/c.yaml}\n",
    );
    assert_eq!(codes_of(&no_type), vec![codes::V004]);
    let empty = shaped("  - {term: a, kind: action, attributes: [{name: title, type: \"\"}], contract: contracts/c.yaml}\n");
    assert_eq!(codes_of(&empty), vec![codes::V004]);
}

#[test]
fn test_rhl2_shape_entity_without_instances_from_is_v004() {
    let tmp = shaped("  - {term: host, kind: entity, contract: contracts/c.yaml}\n");
    assert_eq!(codes_of(&tmp), vec![codes::V004]);
    let v = validate(&tmp.path().join("vocab/demo-v1.yaml"), tmp.path());
    assert!(v.diagnostics[0].message.contains("instances_from"), "{v:?}");
}

#[test]
fn test_rhl2_shape_unit_without_gives_is_v004() {
    let tmp = shaped("  - {term: GB, kind: unit, contract: contracts/c.yaml}\n");
    assert_eq!(codes_of(&tmp), vec![codes::V004]);
    let v = validate(&tmp.path().join("vocab/demo-v1.yaml"), tmp.path());
    assert!(v.diagnostics[0].message.contains("gives"), "{v:?}");
}

#[test]
fn test_rhl2_shape_is_the_checkers_loader() {
    // One implementation: a vocabulary `vocab validate` refuses for its shape
    // is one `ruchy check` cannot load (RHL-V004).
    let tmp = shaped("  - {term: host, kind: entity, contract: contracts/c.yaml}\n");
    assert!(vocab::load(tmp.path(), "demo", 1).is_err());
    let src = "use vocabulary demo v1\n\njob \"j\"\n  every 1 hour\nend\n";
    let report = check("t.rhl", src, Some(tmp.path()));
    let v004 = report
        .diagnostics
        .iter()
        .find(|d| d.code == codes::V004)
        .unwrap_or_else(|| panic!("the vocabulary is refused: {report:#?}"));
    assert!(v004.message.contains("instances_from"), "{v004:?}");
}

// ---------------------------------------------------------------- list, show, validate

#[test]
fn test_rhl2_vocab_list_names_every_vocabulary_with_its_term_count() {
    let rows = list(&root());
    let names: Vec<String> = rows
        .iter()
        .map(|r| format!("{} v{}", r.name, r.version))
        .collect();
    assert_eq!(
        names,
        vec!["fleet v1", "fleet v2", "tickets v1", "tickets v2"]
    );
    for r in &rows {
        let loaded = vocab::load(&root(), &r.name, r.version).expect("loads");
        assert_eq!(r.terms, loaded.terms.len());
        assert_eq!(r.error, None);
    }
    let out = list_outcome(&root(), OutputFormat::Json);
    assert_eq!(out.exit, 0);
    let v: serde_json::Value = serde_json::from_str(&out.stdout).expect("JSON");
    assert_eq!(v[3]["name"], "tickets");
    assert_eq!(v[3]["version"], 2);
}

#[test]
fn test_rhl2_vocab_list_reports_a_file_that_does_not_load_exit_1() {
    let tmp = shaped("  - {term: GB, kind: unit, contract: contracts/c.yaml}\n");
    let out = list_outcome(tmp.path(), OutputFormat::Text);
    assert_eq!(out.exit, 1);
    assert!(
        out.stdout.contains("demo v1  does not load"),
        "{}",
        out.stdout
    );
}

#[test]
fn test_rhl2_vocab_show_gives_every_field_of_every_term() {
    let out = show_outcome(&root(), "tickets", "v2", OutputFormat::Json);
    assert_eq!(out.exit, 0, "{}", out.stderr);
    let v: serde_json::Value = serde_json::from_str(&out.stdout).expect("JSON");
    let file = v["terms"]
        .as_array()
        .expect("terms")
        .iter()
        .find(|t| t["term"] == "file ticket")
        .expect("file ticket");
    assert_eq!(file["kind"], "action");
    assert_eq!(file["takes"][0]["type"], "Text");
    assert_eq!(file["gives"], "Ticket");
    assert_eq!(file["effect"], "write tickets");
    assert_eq!(file["attributes"][1]["name"], "label");
    assert_eq!(
        file["contract"],
        "contracts/rhl-tickets-file-ticket-v2.yaml"
    );
    let text = show_outcome(&root(), "fleet", "1", OutputFormat::Text);
    assert!(text.stdout.starts_with("fleet v1 ("), "{}", text.stdout);
    assert!(
        text.stdout.contains("disk free of  measure"),
        "{}",
        text.stdout
    );
}

#[test]
fn test_rhl2_vocab_show_of_an_unknown_vocabulary_is_v004_exit_2() {
    let out = show_outcome(&root(), "fleet", "v9", OutputFormat::Text);
    assert_eq!(out.exit, 2);
    assert!(out.stderr.contains("RHL-V004"), "{}", out.stderr);
}

#[test]
fn test_rhl2_vocab_validate_by_name_or_by_file() {
    let r = root();
    let by_name = validate_outcome(Some(&r), &["fleet".into(), "v2".into()], OutputFormat::Json);
    assert_eq!(by_name.exit, 0, "{}{}", by_name.stdout, by_name.stderr);
    let file = r.join("vocab/tickets-v1.yaml").display().to_string();
    let by_file = validate_outcome(None, &[file], OutputFormat::Text);
    assert_eq!(by_file.exit, 0, "{}{}", by_file.stdout, by_file.stderr);
    assert!(
        by_file.stdout.trim_end().ends_with(": pass"),
        "{}",
        by_file.stdout
    );
    let bad = validate_outcome(
        Some(&r),
        &["a".into(), "b".into(), "c".into()],
        OutputFormat::Text,
    );
    assert_eq!(bad.exit, 1);
}

#[test]
fn test_rhl2_vocab_validate_of_a_deleted_contract_is_c001_exit_2() {
    let tmp = tempfile::tempdir().expect("tempdir");
    for sub in ["vocab", "contracts"] {
        let to = tmp.path().join(sub);
        std::fs::create_dir_all(&to).expect("mkdir");
        for e in std::fs::read_dir(root().join(sub))
            .expect("dir")
            .filter_map(Result::ok)
        {
            if e.path().is_file() {
                std::fs::copy(e.path(), to.join(e.file_name())).expect("copy");
            }
        }
    }
    std::fs::remove_file(tmp.path().join("contracts/rhl-fleet-gb-v2.yaml")).expect("rm");
    let out = validate_outcome(
        Some(tmp.path()),
        &["fleet".into(), "2".into()],
        OutputFormat::Json,
    );
    assert_eq!(out.exit, 2);
    let v: serde_json::Value = serde_json::from_str(&out.stdout).expect("JSON");
    assert_eq!(v["verdict"], "refused");
    assert_eq!(v["diagnostics"][0]["code"], "RHL-C001");
    assert_eq!(v["diagnostics"][0]["term"], "GB");
}

#[test]
fn test_rhl2_vocab_root_is_explicit_or_found_from_the_cwd() {
    let r = root();
    assert_eq!(resolve_root(None, &r), Some(r.clone()));
    assert_eq!(resolve_root(None, &r.join("docs/rhl")), Some(r.clone()));
    let tmp = tempfile::tempdir().expect("tempdir");
    assert_eq!(resolve_root(None, tmp.path()), None);
    assert_eq!(
        resolve_root(Some(Path::new("/x")), tmp.path()),
        Some(PathBuf::from("/x"))
    );
    assert_eq!(parse_version("v2"), Some(2));
    assert_eq!(parse_version("2"), Some(2));
    assert_eq!(parse_version("two"), None);
}
