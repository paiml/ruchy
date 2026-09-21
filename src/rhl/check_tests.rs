//! RHL-1 checker tests: the planted-break gate, the valid-corpus measurement
//! (plan D2 — a report, not a clean gate), the differential report, and unit
//! tests for every code the checker emits.

use super::{check, Report, Verdict};
use crate::rhl::codes;
use crate::rhl::diag::{Diagnostic, Edit};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------- helpers

fn root() -> PathBuf {
    crate::rhl::repo_root()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn has(report: &Report, code: &str) -> bool {
    report.diagnostics.iter().any(|d| d.code == code)
}

fn codes_of(report: &Report) -> Vec<String> {
    let mut v: Vec<String> = report.diagnostics.iter().map(|d| d.code.clone()).collect();
    v.sort();
    v
}

/// The source line `line` (1-based) of `text`.
fn line_of(text: &str, line: usize) -> &str {
    text.lines().nth(line - 1).unwrap_or("")
}

/// The text of `line` from `d`'s column on.
fn text_at(source: &str, d: &Diagnostic) -> String {
    line_of(source, d.span.line)
        .chars()
        .skip(d.span.col - 1)
        .collect()
}

/// The first 1-based line on which `a` and `b` differ.
fn diff_line(a: &str, b: &str) -> usize {
    let (la, lb): (Vec<&str>, Vec<&str>) = (a.lines().collect(), b.lines().collect());
    (0..la.len().max(lb.len()))
        .find(|&i| la.get(i) != lb.get(i))
        .map_or(0, |i| i + 1)
}

/// `line` with `edit` applied (columns and lengths are in characters).
fn apply(line: &str, edit: &Edit) -> String {
    let chars: Vec<char> = line.chars().collect();
    let start = edit.span.col - 1;
    let end = start + edit.span.len;
    let mut out: String = chars[..start].iter().collect();
    out.push_str(&edit.text);
    out.extend(&chars[end..]);
    out
}

// ---------------------------------------------------------------- corpus

#[derive(Debug, Deserialize)]
struct BreakSpec {
    base: String,
    class: String,
    expected_code: String,
    #[serde(default)]
    candidates: Vec<String>,
}

struct Break {
    name: String,
    spec: BreakSpec,
    base: String,
    broken: String,
    report: Report,
}

fn dirs(path: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(path)
        .unwrap_or_else(|e| panic!("cannot list {}: {e}", path.display()))
        .map(|e| e.expect("dir entry").path())
        .collect();
    v.sort();
    v
}

fn load_break(dir: &Path) -> Break {
    let spec: BreakSpec =
        serde_yaml_ng::from_str(&read(&dir.join("break.yaml"))).expect("break.yaml parses");
    let name = dir
        .strip_prefix(root())
        .expect("under root")
        .display()
        .to_string();
    let broken = read(&dir.join("broken.rhl"));
    let report = check(&name, &broken, Some(&root()));
    let base = read(&root().join(&spec.base));
    Break {
        name,
        spec,
        base,
        broken,
        report,
    }
}

fn all_breaks() -> Vec<Break> {
    let planted = root().join("docs/rhl/breaks/planted");
    dirs(&planted)
        .iter()
        .flat_map(|class| dirs(class))
        .map(|d| load_break(&d))
        .collect()
}

fn is_substitution(class: &str) -> bool {
    matches!(
        class,
        "typo'd term" | "two-candidate typo" | "wrong type" | "wrong unit"
    )
}

fn valid_programs() -> Vec<(String, String, Report)> {
    dirs(&root().join("docs/rhl/breaks/valid"))
        .iter()
        .map(|p| {
            let name = p
                .file_name()
                .expect("file name")
                .to_string_lossy()
                .to_string();
            let source = read(p);
            let report = check(&name, &source, Some(&root()));
            (name, source, report)
        })
        .collect()
}

// ---------------------------------------------------------------- a. planted-break gate

/// The diagnostics with the break's expected code, on the mutated line for a
/// substitution class, anywhere for a deletion class.
fn hits(b: &Break) -> Vec<&Diagnostic> {
    let line = diff_line(&b.base, &b.broken);
    let substitution = is_substitution(&b.spec.class);
    b.report
        .diagnostics
        .iter()
        .filter(|d| d.code == b.spec.expected_code)
        .filter(|d| !substitution || d.span.line == line)
        .collect()
}

fn v002_repairs(b: &Break, d: &Diagnostic) -> bool {
    let safe: Vec<_> = d.fixes.iter().filter(|f| f.safe).collect();
    let [fix] = safe.as_slice() else { return false };
    let [edit] = fix.edits.as_slice() else {
        return false;
    };
    let repaired = apply(line_of(&b.broken, edit.span.line), edit);
    d.candidates.len() == 1 && d.fixes.len() == 1 && repaired == line_of(&b.base, edit.span.line)
}

fn v003_matches(b: &Break, d: &Diagnostic) -> bool {
    let got: BTreeSet<&str> = d.candidates.iter().map(|c| c.term.as_str()).collect();
    let want: BTreeSet<&str> = b.spec.candidates.iter().map(String::as_str).collect();
    got == want && d.fixes.iter().all(|f| !f.safe)
}

/// `None` when `b` yields its expected code as the gate requires, else why not.
fn break_failure(b: &Break) -> Option<String> {
    let found = hits(b);
    let ok = match b.spec.expected_code.as_str() {
        codes::V002 => found.iter().any(|d| v002_repairs(b, d)),
        codes::V003 => found.iter().any(|d| v003_matches(b, d)),
        _ => !found.is_empty(),
    };
    let got: Vec<String> = b
        .report
        .diagnostics
        .iter()
        .map(|d| format!("{}@{}", d.code, d.span.line))
        .collect();
    (!ok).then(|| {
        format!(
            "{}: wants {} ({}), got {got:?}",
            b.name, b.spec.expected_code, b.spec.class
        )
    })
}

/// Planted breaks whose pre-registered expectation contradicts the
/// pre-registered vocabulary. Operator ruling on RHL-1 (2026-09-21), option
/// "Named exception + RHL-16": gate the other 70, name these two here, and
/// repair the corpus as v2 files under RHL-16.
///
/// Both change line 13 to `when free is below 1 hour` and expect RHL-T001
/// ("wrong unit"). But their base binds `free` with `pin status of`, which
/// gives Text (vocab/fleet-v1.yaml), so the line compares Text with a
/// Duration: RHL-T002. The base already has RHL-T002 on the same line (plan
/// M4). The list is exact in both directions: another failing break turns the
/// gate red, and so does one of these two starting to pass.
const KNOWN_CORPUS_DEFECTS: [(&str, &str); 2] = [
    (
        "docs/rhl/breaks/planted/wrong-unit/04-gx13-pin-check",
        codes::T002,
    ),
    (
        "docs/rhl/breaks/planted/wrong-unit/09-gx18-pin-check-b",
        codes::T002,
    ),
];

/// The code a known defect yields on its mutated line instead of its expected one.
fn known_defect_code(b: &Break) -> Option<&'static str> {
    KNOWN_CORPUS_DEFECTS
        .iter()
        .find(|(name, _)| b.name.ends_with(name))
        .map(|(_, code)| *code)
}

fn yields_on_mutated_line(b: &Break, code: &str) -> bool {
    let line = diff_line(&b.base, &b.broken);
    b.report
        .diagnostics
        .iter()
        .any(|d| d.code == code && d.span.line == line)
}

#[test]
fn test_rhl_1_check_planted_breaks_all_yield_their_expected_code() {
    let breaks = all_breaks();
    let failures: Vec<&Break> = breaks
        .iter()
        .filter(|b| break_failure(b).is_some())
        .collect();
    let unexpected: Vec<String> = failures
        .iter()
        .filter(|b| known_defect_code(b).is_none())
        .filter_map(|b| break_failure(b))
        .collect();
    assert!(
        unexpected.is_empty(),
        "{} planted break(s) fail the gate:\n{}",
        unexpected.len(),
        unexpected.join("\n")
    );
    assert_eq!(breaks.len(), 72, "the planted corpus has 6 classes x 12");
}

#[test]
fn test_rhl_1_check_known_corpus_defects_fail_exactly_as_named() {
    let breaks = all_breaks();
    for (name, code) in KNOWN_CORPUS_DEFECTS {
        let b = breaks
            .iter()
            .find(|b| b.name.ends_with(name))
            .unwrap_or_else(|| panic!("{name}: the named defect is not in the corpus"));
        assert!(
            break_failure(b).is_some(),
            "{name} now yields its expected code: remove it from KNOWN_CORPUS_DEFECTS"
        );
        assert!(
            yields_on_mutated_line(b, code),
            "{name}: expected {code} on the mutated line, as the exception records"
        );
    }
}

// ---------------------------------------------------------------- b. valid-corpus measurement

fn program_number(name: &str) -> &str {
    name.get(..2).unwrap_or("")
}

fn has_unknown_word_at(source: &str, report: &Report, word: &str) -> bool {
    let unknown = [codes::V001, codes::V002, codes::V003];
    report
        .diagnostics
        .iter()
        .any(|d| unknown.contains(&d.code.as_str()) && text_at(source, d).starts_with(word))
}

fn effect_message(report: &Report, effect: &str) -> bool {
    report
        .diagnostics
        .iter()
        .any(|d| d.code == codes::E001 && d.message.contains(effect))
}

fn t_and_e(report: &Report) -> Vec<String> {
    codes_of(report)
        .into_iter()
        .filter(|c| c.starts_with("RHL-T") || c.starts_with("RHL-E"))
        .collect()
}

/// Plan M4: the `(T code, E001 effect)` each mistyped program should show.
fn m4_prediction(number: &str) -> Option<(&'static str, &'static str)> {
    match number {
        "03" | "08" => Some((codes::T001, "read runner")),
        "04" | "09" => Some((codes::T002, "read host")),
        "05" | "10" => Some((codes::T001, "read tickets")),
        _ => None,
    }
}

fn assert_m4(name: &str, report: &Report) {
    match m4_prediction(program_number(name)) {
        Some((t, effect)) => {
            assert!(has(report, t), "{name}: M4 predicts {t}");
            assert!(
                effect_message(report, effect),
                "{name}: M4 predicts E001 `{effect}`"
            );
        }
        None => assert!(
            t_and_e(report).is_empty(),
            "{name}: M4 predicts no T/E code"
        ),
    }
}

#[test]
fn test_rhl_1_check_valid_corpus_measurement_matches_plan_m3_m4() {
    let programs = valid_programs();
    eprintln!("RHL-1 valid-corpus measurement (plan D2; report, not a gate):");
    for (name, _, report) in &programs {
        eprintln!("  {name:36} {:?}", codes_of(report));
    }
    for (name, source, report) in &programs {
        for word in ["ticket count", "title", "label"] {
            assert!(
                has_unknown_word_at(source, report, word),
                "{name}: M3 predicts V00x on `{word}`"
            );
        }
        assert_m4(name, report);
    }
    assert_eq!(programs.len(), 12);
}

// ---------------------------------------------------------------- c. differential report

/// Whether `b`'s expected code is new relative to its base: for a
/// substitution, absent from the base on the same line; for a deletion,
/// more frequent than in the base.
fn is_new(b: &Break) -> bool {
    let code = b.spec.expected_code.as_str();
    let base = check(&b.spec.base, &b.base, Some(&root()));
    if is_substitution(&b.spec.class) {
        let line = diff_line(&b.base, &b.broken);
        let on_line = |r: &Report| {
            r.diagnostics
                .iter()
                .any(|d| d.code == code && d.span.line == line)
        };
        return !on_line(&base);
    }
    let count = |r: &Report| r.diagnostics.iter().filter(|d| d.code == code).count();
    count(&b.report) > count(&base)
}

#[test]
fn test_rhl_1_check_differential_report_names_breaks_whose_code_is_not_new() {
    let not_new: Vec<String> = all_breaks()
        .iter()
        .filter(|b| !is_new(b))
        .map(|b| format!("{} ({})", b.name, b.spec.expected_code))
        .collect();
    eprintln!(
        "RHL-1 differential (plan D2/M5): {} break(s) whose code is NOT new:",
        not_new.len()
    );
    for n in &not_new {
        eprintln!("  {n}");
    }
}

// ---------------------------------------------------------------- d. unit tests

const CLEAN: &str = "use vocabulary fleet v1

job \"clean\"
  every 1 hour
  may read disk

  let free be disk free of \"/\"

  expect free is at least 10 GB

  example \"low\"
    given free is 5 GB
    then free is below 10 GB
  end
end
";

fn check_src(src: &str) -> Report {
    check("t.rhl", src, Some(&root()))
}

/// A job with `body` (already indented) using both vocabularies.
fn job(body: &str) -> String {
    format!("use vocabulary fleet v1\nuse vocabulary tickets v1\n\njob \"t\"\n{body}end\n")
}

#[test]
fn test_rhl_1_check_clean_program_passes_so_the_checker_does_not_refuse_everything() {
    let report = check_src(CLEAN);
    assert_eq!(report.diagnostics, Vec::<Diagnostic>::new());
    assert_eq!(report.verdict, Verdict::Pass);
    assert_eq!(report.exit_code(), 0);
}

#[test]
fn test_rhl_1_check_exit_codes_are_0_pass_1_error_2_refusal() {
    assert_eq!(check_src(CLEAN).exit_code(), 0);
    let wrong_unit = CLEAN.replace("at least 10 GB", "at least 10 hour");
    let report = check_src(&wrong_unit);
    assert_eq!(codes_of(&report), vec![codes::T001]);
    assert_eq!((report.verdict, report.exit_code()), (Verdict::Fail, 1));
    let typo = CLEAN.replace("disk free of", "disk fre of");
    let report = check_src(&typo);
    assert_eq!(codes_of(&report), vec![codes::V002]);
    assert_eq!((report.verdict, report.exit_code()), (Verdict::Refused, 2));
}

#[test]
fn test_rhl_1_check_parse_failure_is_one_p_code_and_nothing_else() {
    let report = check_src("use vocabulary fleet v1\n\njob \"x\"\n  every 1 hour\n");
    assert_eq!(codes_of(&report), vec![codes::P001]);
    assert_eq!(report.exit_code(), 1);
}

#[test]
fn test_rhl_1_check_report_json_has_the_plan_d8_shape() {
    let report = check_src(&CLEAN.replace("disk free of", "disk fre of"));
    let v = serde_json::to_value(&report).expect("serializable");
    let keys: BTreeSet<&str> = v
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        BTreeSet::from(["file", "verdict", "diagnostics", "unverified"])
    );
    assert_eq!(v["verdict"], "refused");
    assert_eq!(v["diagnostics"][0]["code"], codes::V002);
    assert_eq!(v["diagnostics"][0]["candidates"][0]["term"], "disk free of");
    assert_eq!(
        v["diagnostics"][0]["candidates"][0]["vocabulary"],
        "fleet v1"
    );
    assert!(v["unverified"].is_array());
}

#[test]
fn test_rhl_1_check_unknown_vocabulary_is_v004() {
    let report = check_src("use vocabulary nosuch v1\n");
    assert_eq!(codes_of(&report), vec![codes::V004]);
    assert_eq!(report.exit_code(), 2);
    assert_eq!(
        codes_of(&check_src("use vocabulary fleet\n")),
        vec![codes::V004]
    );
    let rootless = check("t.rhl", "use vocabulary fleet v1\n", None);
    assert_eq!(codes_of(&rootless), vec![codes::V004]);
}

#[test]
fn test_rhl_1_check_bare_number_for_a_duration_is_t003() {
    let report = check_src(&CLEAN.replace("every 1 hour", "every 5"));
    assert_eq!(codes_of(&report), vec![codes::T003]);
    let report = check_src(&CLEAN.replace("every 1 hour", "every 5 GB"));
    assert_eq!(codes_of(&report), vec![codes::T001]);
    let report = check_src(&CLEAN.replace("at least 10 GB", "at least 10"));
    assert_eq!(codes_of(&report), vec![codes::T003]);
}

#[test]
fn test_rhl_1_check_given_then_outside_example_is_x001() {
    let report = check_src(&CLEAN.replace("  expect free", "  then free"));
    assert_eq!(codes_of(&report), vec![codes::X001]);
    let nested = CLEAN.replace(
        "    given free is 5 GB\n",
        "    when free is below 1 GB\n      given free is 5 GB\n    end\n",
    );
    assert_eq!(codes_of(&check_src(&nested)), vec![codes::X001]);
}

#[test]
fn test_rhl_1_check_for_each_is_b001() {
    let body = "  may read disk\n  let free be disk free of \"/\"\n  for each part in free\n    expect part is 1 GB\n  end\n";
    let report = check_src(&job(body));
    assert_eq!(codes_of(&report), vec![codes::B001]);
    assert_eq!(report.exit_code(), 2);
}

#[test]
fn test_rhl_1_check_undeclared_effect_is_e001_with_an_unsafe_fix() {
    let report = check_src(&CLEAN.replace("  may read disk\n", ""));
    assert_eq!(codes_of(&report), vec![codes::E001]);
    let d = &report.diagnostics[0];
    assert!(d.message.contains("may read disk"), "{}", d.message);
    assert_eq!(d.fixes.len(), 1);
    assert!(!d.fixes[0].safe);
    assert_eq!(d.fixes[0].edits[0].text, "  may read disk\n");
    assert_eq!(d.fixes[0].edits[0].span.line, 4);
    let call = CLEAN.replace("may read disk", "may call disk");
    assert_eq!(codes_of(&check_src(&call)), vec![codes::E001]);
}

#[test]
fn test_rhl_1_check_effect_target_and_unit_have_their_own_namespaces() {
    let report = check_src(&CLEAN.replace("may read disk", "may read dsk"));
    let d = report
        .diagnostics
        .iter()
        .find(|d| d.code == codes::V002)
        .expect("V002");
    assert_eq!(d.candidates[0].term, "disk");
    assert_eq!(
        d.expected.as_ref().and_then(|e| e.kind.as_deref()),
        Some("effect target")
    );
    let report = check_src(&CLEAN.replace("10 GB", "10 GBs"));
    let d = report
        .diagnostics
        .iter()
        .find(|d| d.code == codes::V002)
        .expect("V002");
    assert_eq!(d.candidates[0].term, "GB");
    assert_eq!(
        check_src(&CLEAN.replace("10 GB", "10 %")).diagnostics[0].code,
        codes::T001
    );
}

#[test]
fn test_rhl_1_check_tickets_filed_in_and_actions_with_targets() {
    let body = "  may read tickets\n  may write tickets\n  let n be tickets filed in repo \"paiml/infra\"\n  expect n is at most 3\n  file ticket in repo \"paiml/infra\"\n";
    let report = check_src(&job(body));
    assert_eq!(report.diagnostics, Vec::<Diagnostic>::new());
    let repo = report
        .unverified
        .iter()
        .find(|u| u.entity == "repo")
        .expect("repo unverified");
    assert_eq!(
        (repo.instance.as_str(), repo.source_glob.as_str()),
        ("paiml/infra", "org/repos/*.yaml")
    );
    let typo = check_src(&job(&body.replace("tickets filed in", "ticket filed in")));
    let d = &typo.diagnostics[0];
    assert_eq!(
        (d.code.as_str(), d.candidates[0].term.as_str()),
        (codes::V002, "tickets filed in")
    );
    let bad_target = check_src(&job(
        &body.replace("file ticket in repo", "file ticket in host")
    ));
    assert_eq!(codes_of(&bad_target), vec![codes::T002]);
}

#[test]
fn test_rhl_1_check_misused_terms_are_t002() {
    const EXPECT: &str = "  expect free is at least 10 GB\n";
    const LET: &str = "disk free of \"/\"";
    let cases = [
        (EXPECT, "  expect free\n"),
        (EXPECT, "  expect free contains 10 GB\n"),
        (EXPECT, "  expect free is one of 10 GB\n"),
        (EXPECT, "  runs on free\n"),
        (EXPECT, "  expect hour is 1 hour\n"),
        (EXPECT, "  disk free of \"/\"\n"),
        (EXPECT, "  expect free in \"x\"\n"),
        (LET, "disk free of"),
        (LET, "disk free of 3 GB"),
    ];
    for (from, to) in cases {
        let report = check_src(&CLEAN.replace(from, to));
        let got = codes_of(&report);
        assert_eq!(got, vec![codes::T002], "`{to}`: {:#?}", report.diagnostics);
    }
}

#[test]
fn test_rhl_1_check_let_and_set_scope() {
    let set_ok = CLEAN.replace("  expect free", "  set free to 3 GB\n  expect free");
    assert_eq!(codes_of(&check_src(&set_ok)), Vec::<String>::new());
    let set_bad = CLEAN.replace("  expect free", "  set free to 3 hour\n  expect free");
    assert_eq!(codes_of(&check_src(&set_bad)), vec![codes::T001]);
    let set_unknown = CLEAN.replace("  expect free", "  set fre to 3 GB\n  expect free");
    let report = check_src(&set_unknown);
    assert_eq!(report.diagnostics[0].candidates[0].vocabulary, "let");
    let before = CLEAN.replace("  expect free is", "  expect fre is");
    assert_eq!(check_src(&before).diagnostics[0].candidates[0].term, "free");
}

// ---------------------------------------------------------------- temp vocabulary roots

/// A temp root with `vocab/fleet-v1.yaml` and the fleet contracts, except
/// `skip`, and an optional `machines/<host>/forjar.yaml`.
fn temp_root(skip: Option<&str>, host: Option<&str>) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    let t = tmp.path();
    std::fs::create_dir_all(t.join("vocab")).expect("mkdir vocab");
    std::fs::create_dir_all(t.join("contracts")).expect("mkdir contracts");
    std::fs::copy(
        root().join("vocab/fleet-v1.yaml"),
        t.join("vocab/fleet-v1.yaml"),
    )
    .expect("copy vocab");
    let vocab = crate::rhl::vocab::load(t, "fleet", 1).expect("fleet v1 loads");
    for term in vocab.terms.iter().filter(|x| Some(x.term.as_str()) != skip) {
        std::fs::copy(root().join(&term.contract), t.join(&term.contract)).expect("copy contract");
    }
    if let Some(h) = host {
        std::fs::create_dir_all(t.join("machines").join(h)).expect("mkdir machine");
        std::fs::write(t.join("machines").join(h).join("forjar.yaml"), "host: x\n").expect("write");
    }
    tmp
}

fn with_host(host: &str) -> String {
    CLEAN.replace(
        "  every 1 hour\n",
        &format!("  runs on host {host}\n  every 1 hour\n"),
    )
}

#[test]
fn test_rhl_1_check_term_without_contract_refuses_the_vocabulary_c001() {
    let tmp = temp_root(Some("hour"), None);
    let report = check("t.rhl", CLEAN, Some(tmp.path()));
    assert_eq!(codes_of(&report), vec![codes::C001]);
    let d = &report.diagnostics[0];
    assert_eq!(d.span.line, 1);
    assert!(
        d.message.contains("`hour`") && d.message.contains("`fleet v1`"),
        "{}",
        d.message
    );
    assert_eq!((report.verdict, report.exit_code()), (Verdict::Refused, 2));
    let full = temp_root(None, None);
    assert_eq!(check("t.rhl", CLEAN, Some(full.path())).exit_code(), 0);
}

#[test]
fn test_rhl_1_check_closed_world_host_instances_d6() {
    let tmp = temp_root(None, Some("gx10"));
    let report = check("t.rhl", &with_host("gx11"), Some(tmp.path()));
    assert_eq!(codes_of(&report), vec![codes::V002]);
    let d = &report.diagnostics[0];
    assert_eq!(d.candidates[0].term, "gx10");
    assert!(d.fixes[0].safe);
    assert_eq!(d.fixes[0].edits[0].text, "gx10");
    let ok = check("t.rhl", &with_host("gx10"), Some(tmp.path()));
    assert_eq!(ok.diagnostics, Vec::<Diagnostic>::new());
    assert!(ok.unverified.is_empty());
}

#[test]
fn test_rhl_1_check_open_world_host_is_unverified_not_refused_d6() {
    let tmp = temp_root(None, None);
    let report = check("t.rhl", &with_host("gx10"), Some(tmp.path()));
    assert_eq!(report.diagnostics, Vec::<Diagnostic>::new());
    assert_eq!(report.exit_code(), 0);
    let u = &report.unverified[0];
    let got = (
        u.entity.as_str(),
        u.instance.as_str(),
        u.source_glob.as_str(),
    );
    assert_eq!(got, ("host", "gx10", "machines/*/forjar.yaml"));
    let bare = check(
        "t.rhl",
        &CLEAN.replace("  every 1 hour\n", "  runs on host\n  every 1 hour\n"),
        Some(tmp.path()),
    );
    assert_eq!(codes_of(&bare), vec![codes::T002]);
}

// ---------------------------------------------------------------- pre-PR review findings (ph6 quorum)

/// The one V002 diagnostic of `report`, with its fixes.
fn only_v002(report: &Report) -> &Diagnostic {
    let v: Vec<&Diagnostic> = report
        .diagnostics
        .iter()
        .filter(|d| d.code == codes::V002)
        .collect();
    assert_eq!(v.len(), 1, "{:?}", codes_of(report));
    v[0]
}

#[test]
fn test_rhl_1_check_unique_candidate_that_breaks_the_entity_position_is_not_safe() {
    let src = "use vocabulary fleet v1\n\njob \"a\"\n  runs on hou gx10\n  may read disk\nend\n";
    let report = check_src(src);
    let d = only_v002(&report);
    assert_eq!(d.candidates[0].term, "hour");
    assert!(
        d.fixes.iter().all(|f| !f.safe),
        "`hour` is a unit where `runs on` needs an entity: §3.5 says a fix is safe \
         only when it preserves types"
    );
}

#[test]
fn test_rhl_1_check_unique_candidate_that_changes_the_dimension_is_not_safe() {
    let src = CLEAN.replace("10 GB\n\n", "10 hou\n\n");
    let report = check_src(&src);
    let d = only_v002(&report);
    assert_eq!(d.candidates[0].term, "hour");
    assert!(
        d.fixes.iter().all(|f| !f.safe),
        "`10 hour` is a Duration compared with the Size `free`: not type-preserving"
    );
}

#[test]
fn test_rhl_1_check_unique_candidate_that_keeps_types_stays_safe() {
    let src = CLEAN.replace("disk free of", "disk fre of");
    let report = check_src(&src);
    let d = only_v002(&report);
    assert!(d.fixes.iter().any(|f| f.safe));
}

#[test]
fn test_rhl_1_check_term_with_no_contract_field_is_c001_not_v004() {
    let tmp = temp_root(None, None);
    let path = tmp.path().join("vocab/fleet-v1.yaml");
    let yaml = std::fs::read_to_string(&path).expect("read copy");
    let line = "    contract: contracts/rhl-fleet-hour-v1.yaml\n";
    assert!(yaml.contains(line), "fixture drifted");
    std::fs::write(&path, yaml.replacen(line, "", 1)).expect("write");
    let report = check("t.rhl", CLEAN, Some(tmp.path()));
    assert_eq!(
        codes_of(&report),
        vec![codes::C001],
        "{:?}",
        report.diagnostics
    );
    assert!(report.diagnostics[0].message.contains("`hour`"));
}
