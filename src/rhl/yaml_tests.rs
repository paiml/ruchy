//! RHL-3 tests: the YAML surface and `ruchy convert` (spec RHL-001 §2 YAML
//! row, §5 `convert`, §7 F3). Lib tests, not `tests/`: the required CI check
//! runs `cargo test --lib` only.
//!
//! Groups: F3 over every corpus program the grammar parses (count asserted
//! against the directory listing), both directions, byte for byte; F3's
//! positive control; the shape of the YAML and the RHL-3 doc; refusals (a
//! YAML that does not deserialize, and a tree the grammar could not produce);
//! the verbs (`convert`, and `check`/`fmt` extended by extension); property
//! tests over generated trees and over arbitrary text.

use super::{diagnostic, format_yaml, from_yaml, is_rhl_yaml, to_yaml, YamlError};
use crate::rhl::check::{check, check_yaml, Verdict};
use crate::rhl::cli::{
    check_files, convert_file, convert_source, format_file_source, is_rhl, OutputFormat, Target,
};
use crate::rhl::fmt::fmt_tests::arb_program;
use crate::rhl::fmt::{format, format_source};
use crate::rhl::parse;
use proptest::prelude::*;
use std::path::{Path, PathBuf};

const SPEC: &str = "docs/specifications/ruchy-high-level-language-interface.md";
const DOC: &str = "docs/rhl/rhl-3.md";
const BREAKS: &str = "docs/rhl/breaks";
const TYPO: &str = "docs/rhl/breaks/v2/planted/typo-term/03-gx12-runner-load-watch/broken.rhl";

fn root() -> PathBuf {
    crate::rhl::repo_root()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("RHL-3: cannot read {}: {e}", path.display()))
}

fn listing(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("RHL-3: cannot list {}: {e}", dir.display()))
        .map(|e| e.expect("RHL-3: readable dir entry").path())
        .collect();
    v.sort();
    v
}

fn rhl_files(dir: &Path) -> Vec<PathBuf> {
    listing(dir)
        .into_iter()
        .filter(|p| p.extension().is_some_and(|x| x == "rhl"))
        .collect()
}

/// `<planted>/<class>/<case>/broken.rhl` for every class and case.
fn broken_files(planted: &Path) -> Vec<PathBuf> {
    listing(planted)
        .iter()
        .flat_map(|class| listing(class))
        .map(|case| case.join("broken.rhl"))
        .filter(|p| p.is_file())
        .collect()
}

/// Every corpus program the brief names, with the listing it came from.
fn corpus() -> Vec<PathBuf> {
    let b = root().join(BREAKS);
    let mut all = rhl_files(&b.join("v2").join("valid"));
    all.extend(rhl_files(&b.join("valid")));
    all.extend(broken_files(&b.join("planted")));
    all.extend(broken_files(&b.join("v2").join("planted")));
    all
}

/// The corpus programs that parse, with the proof that exactly the
/// `missing-end` class is left out.
fn parseable_corpus() -> Vec<(PathBuf, String)> {
    let all = corpus();
    let (ok, bad): (Vec<_>, Vec<_>) = all
        .iter()
        .map(|p| (p.clone(), read(p)))
        .partition(|(_, src)| parse(src).is_ok());
    for (p, _) in &bad {
        let s = p.display().to_string();
        assert!(s.contains("/missing-end/"), "RHL-3: {s} does not parse");
    }
    let missing_end = all
        .iter()
        .filter(|p| p.display().to_string().contains("/missing-end/"))
        .count();
    assert!(ok.len() > 30, "RHL-3: only {} corpus programs", ok.len());
    assert_eq!(ok.len() + missing_end, all.len(), "count vs listing");
    assert_eq!(
        bad.len(),
        missing_end,
        "every missing-end case fails to parse"
    );
    ok
}

fn yaml_of(rhl: &str) -> String {
    to_yaml(&parse(rhl).unwrap_or_else(|e| panic!("does not parse: {e:?}\n{rhl}")))
}

fn rhl_of(yaml: &str) -> String {
    format(&from_yaml(yaml).unwrap_or_else(|e| panic!("does not read back: {e:?}\n{yaml}")))
}

/// The fenced code block under `### 3.2 Example` in the spec.
fn spec_example() -> String {
    let spec = read(&root().join(SPEC));
    let after = spec.split_once("### 3.2 Example").expect("§3.2").1;
    let body = after.split_once("```").expect("§3.2 fence").1;
    let body = body.split_once('\n').expect("fence line ends").1;
    body.split_once("```").expect("fence closes").0.to_string()
}

// ───────────────────────── a. F3 over the corpus ─────────────────────────

#[test]
fn test_rhl3_f3_corpus_rhl_to_yaml_to_rhl_is_byte_identical_after_fmt() {
    let files = parseable_corpus();
    for (path, src) in &files {
        let normal = format_source(src).expect("parses");
        let back = rhl_of(&yaml_of(&normal));
        assert_eq!(back, normal, "{}", path.display());
    }
}

#[test]
fn test_rhl3_f3_corpus_yaml_to_rhl_to_yaml_is_byte_identical() {
    for (path, src) in &parseable_corpus() {
        let yaml = yaml_of(src);
        assert_eq!(yaml_of(&rhl_of(&yaml)), yaml, "{}", path.display());
        assert_eq!(format_yaml(&yaml).as_deref(), Ok(yaml.as_str()));
    }
}

#[test]
fn test_rhl3_f3_positive_control_one_token_changes_the_yaml() {
    let src = spec_example();
    let changed = src.replacen("below 100 GB", "below 101 GB", 1);
    assert_ne!(changed, src, "the control edits the example");
    assert_ne!(yaml_of(&changed), yaml_of(&src));
    let word = src.replacen("may read disk", "may write disk", 1);
    assert_ne!(yaml_of(&word), yaml_of(&src));
}

#[test]
fn test_rhl3_f3_layout_alone_does_not_change_the_yaml() {
    let src = spec_example();
    let mangled = src.replace("\n  ", "\n        ").replace("\n\n", "\n\n\n");
    assert_eq!(yaml_of(&mangled), yaml_of(&src));
}

// ───────────────────────── b. shape and the doc ─────────────────────────

#[test]
fn test_rhl3_shape_is_human_and_tagless() {
    let y = yaml_of(&spec_example());
    assert!(y.starts_with("rhl: 1\ndecls:\n"), "{y}");
    for want in [
        "- use:\n    vocabulary: fleet v1\n",
        "kind: job",
        "name: gx10 disk watch",
        "phrase: disk free of",
        "op: is_below",
        "unit: GB",
    ] {
        assert!(y.contains(want), "missing {want:?} in\n{y}");
    }
    for never in ["span", "words", "digits", "!"] {
        assert!(!y.contains(never), "{never:?} in\n{y}");
    }
}

#[test]
fn test_rhl3_doc_shows_the_spec_example_in_both_forms() {
    let doc = read(&root().join(DOC));
    let example = spec_example();
    assert!(
        doc.contains(&format!("```rhl\n{example}```")),
        "{DOC} lacks the §3.2 RHL"
    );
    let yaml = yaml_of(&example);
    assert!(
        doc.contains(&format!("```yaml\n{yaml}```")),
        "{DOC} lacks the §3.2 YAML, which is:\n{yaml}"
    );
}

// ───────────────────────── c. refusals ─────────────────────────

fn refused(yaml: &str) -> YamlError {
    match from_yaml(yaml) {
        Ok(p) => panic!("RHL-3: accepted:\n{yaml}\nas\n{}", format(&p)),
        Err(e) => e,
    }
}

const ONE: &str =
    "rhl: 1\ndecls:\n- unit:\n    kind: job\n    name: j\n    body:\n    - expect:\n        app:\n          call:\n            phrase: a\n";

#[test]
fn test_rhl3_the_fixture_is_accepted() {
    assert_eq!(rhl_of(ONE), "job \"j\"\n  expect a\nend\n");
    assert_eq!(to_yaml(&from_yaml(ONE).expect("reads")), ONE);
}

#[test]
fn test_rhl3_malformed_yaml_is_p004_with_the_yaml_line_and_column() {
    let e = refused("rhl: 1\ndecls: [\n  - {unit: \n");
    assert!(e.line >= 2, "{e:?}");
    let d = diagnostic("x.rhl.yaml", &e);
    assert_eq!(d.code, "RHL-P004");
    assert_eq!((d.span.line, d.span.col), (e.line, e.col));
    assert_eq!(d.file, "x.rhl.yaml");
}

#[test]
fn test_rhl3_schema_marker_is_required_and_is_1() {
    refused(&ONE.replacen("rhl: 1", "rhl: 2", 1));
    refused(&ONE.replacen("rhl: 1\n", "", 1));
}

#[test]
fn test_rhl3_unknown_keys_are_refused_not_dropped() {
    refused(&ONE.replacen("    name: j\n", "    name: j\n    nmae: k\n", 1));
    refused(&ONE.replacen("    name: j\n", "    name: j\n    span: 3\n", 1));
    refused(&format!("{ONE}extra: 1\n"));
}

#[test]
fn test_rhl3_empty_phrases_and_names_are_refused() {
    refused(&ONE.replacen("phrase: a", "phrase: ''", 1));
    refused(&ONE.replacen("phrase: a", "phrase: a  b", 1));
    refused(&ONE.replacen("phrase: a", "phrase: ' a'", 1));
    let let_empty = ONE.replacen(
        "    - expect:\n        app:\n          call:\n            phrase: a\n",
        "    - let:\n        name: ''\n        value:\n          app:\n            text: v\n",
        1,
    );
    refused(&let_empty);
}

#[test]
fn test_rhl3_a_tree_the_grammar_cannot_produce_is_refused() {
    // A keyword as a word: `expect end` does not parse.
    refused(&ONE.replacen("phrase: a", "phrase: end", 1));
    // A quote inside a string: `job "j"k"` does not parse.
    refused(&ONE.replacen("name: j", "name: j\"k", 1));
    // A word the lexer does not make: `A` is no RHL word.
    refused(&ONE.replacen("phrase: a", "phrase: A", 1));
    // Leading zeros: `007` reads back as `7`, a different tree.
    let q = ONE.replacen(
        "call:\n            phrase: a\n",
        "quantity:\n            value: '007'\n",
        1,
    );
    refused(&q);
    // `or` under `and` has no RHL spelling (there are no parentheses).
    let and = "rhl: 1\ndecls:\n- unit:\n    kind: job\n    name: j\n    body:\n    - expect:\n        and:\n        - or:\n          - app:\n              text: a\n          - app:\n              text: b\n        - app:\n            text: c\n";
    refused(and);
}

#[test]
fn test_rhl3_yaml_is_data_aliases_and_local_tags_are_refused() {
    let mut bomb = String::from("a: &a [x, x, x, x, x, x, x, x, x, x]\n");
    let mut prev = 'a';
    for name in ['b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'] {
        let refs = vec![format!("*{prev}"); 10].join(", ");
        bomb.push_str(&format!("{name}: &{name} [{refs}]\n"));
        prev = name;
    }
    bomb.push_str("rhl: 1\ndecls: []\n");
    refused(&bomb);
    refused(&ONE.replacen("phrase: a", "phrase: !custom a", 1));
    refused(&ONE.replacen("    name: j\n", "    name: !python/object j\n", 1));
    refused(&ONE.replacen("        app:\n", "        app: !m\n", 1));
}

// ───────────────────────── d. the verbs ─────────────────────────

#[test]
fn test_rhl3_extension_rhl_yaml_is_rhl_and_is_rhl_yaml() {
    assert!(is_rhl_yaml(Path::new("a/job.rhl.yaml")));
    assert!(!is_rhl_yaml(Path::new("a/job.yaml")));
    assert!(!is_rhl_yaml(Path::new("a/job.rhl")));
    assert!(is_rhl(Path::new("a/job.rhl.yaml")));
    assert!(!is_rhl(Path::new("a/job.yaml")));
}

#[test]
fn test_rhl3_target_flag_accepts_rhl_and_yaml_only() {
    assert_eq!(Target::parse("rhl"), Ok(Target::Rhl));
    assert_eq!(Target::parse("yaml"), Ok(Target::Yaml));
    assert!(Target::parse("json").is_err());
}

#[test]
fn test_rhl3_convert_source_both_directions() {
    let src = spec_example();
    let yaml = convert_source("x.rhl", &src, Target::Yaml).expect("to yaml");
    assert_eq!(yaml, yaml_of(&src));
    let back = convert_source("x.rhl.yaml", &yaml, Target::Rhl).expect("to rhl");
    assert_eq!(back, format_source(&src).expect("parses"));
}

#[test]
fn test_rhl3_convert_source_refuses_what_is_not_a_tree() {
    let err = convert_source("x.rhl.yaml", "rhl: [", Target::Rhl).expect_err("bad yaml");
    assert!(err.contains("x.rhl.yaml:1:"), "{err}");
    assert!(err.contains("error[RHL-P004]"), "{err}");
    let err =
        convert_source("x.rhl", "job \"x\"\n  every 1 hour\n", Target::Yaml).expect_err("bad rhl");
    assert!(err.contains("error[RHL-P001]"), "{err}");
}

#[test]
fn test_rhl3_convert_file_writes_output_and_picks_direction_by_extension() {
    let dir = tempfile::tempdir().expect("temp dir");
    let rhl = dir.path().join("x.rhl");
    std::fs::write(&rhl, spec_example()).expect("write");
    let out = convert_file(&rhl, None, None);
    assert_eq!(out.exit, 0, "{}", out.stderr);
    assert_eq!(out.stdout, yaml_of(&spec_example()));
    let yaml = dir.path().join("x.rhl.yaml");
    let out = convert_file(&rhl, Some(&yaml), None);
    assert_eq!((out.exit, out.stdout.as_str()), (0, ""));
    let back = dir.path().join("y.rhl");
    let out = convert_file(&yaml, Some(&back), None);
    assert_eq!(out.exit, 0, "{}", out.stderr);
    assert_eq!(read(&back), format_source(&spec_example()).expect("parses"));
}

#[test]
fn test_rhl3_convert_file_refusals_exit_2_or_1() {
    let dir = tempfile::tempdir().expect("temp dir");
    let bad = dir.path().join("x.rhl.yaml");
    std::fs::write(&bad, "rhl: 1\ndecls: {\n").expect("write");
    let out = convert_file(&bad, None, None);
    assert_eq!(out.exit, 2);
    assert!(out.stderr.contains("error[RHL-P004]"), "{}", out.stderr);
    assert!(out.stdout.is_empty());
    let other = dir.path().join("x.txt");
    std::fs::write(&other, "job \"j\"\n  expect a\nend\n").expect("write");
    assert_eq!(convert_file(&other, None, None).exit, 2, "no direction");
    let forced = convert_file(&other, None, Some("yaml"));
    assert_eq!(forced.exit, 0, "{}", forced.stderr);
    assert_eq!(convert_file(&other, None, Some("toml")).exit, 2);
    assert_eq!(
        convert_file(&dir.path().join("none.rhl"), None, None).exit,
        1
    );
}

#[test]
fn test_rhl3_check_on_yaml_runs_the_same_checker() {
    let src = read(&root().join(TYPO));
    let yaml = yaml_of(&src);
    let r = Some(root());
    let from_rhl = check("x.rhl", &src, r.as_deref());
    let from_yaml = check_yaml("x.rhl.yaml", &yaml, r.as_deref());
    let codes = |rep: &crate::rhl::check::Report| -> Vec<String> {
        rep.diagnostics.iter().map(|d| d.code.clone()).collect()
    };
    assert!(codes(&from_rhl).contains(&"RHL-V002".to_string()));
    assert_eq!(codes(&from_yaml), codes(&from_rhl));
    assert_eq!(from_yaml.verdict, from_rhl.verdict);
    for (y, t) in from_yaml.diagnostics.iter().zip(&from_rhl.diagnostics) {
        assert_eq!((&y.message, &y.candidates), (&t.message, &t.candidates));
        assert_eq!(
            (y.span.line, y.span.col, y.span.len),
            (0, 0, 0),
            "unknown span"
        );
        assert!(y.fixes.is_empty(), "no text edits into a YAML file");
        assert_eq!(y.file, "x.rhl.yaml");
    }
}

#[test]
fn test_rhl3_check_on_malformed_yaml_is_one_p004() {
    let rep = check_yaml("x.rhl.yaml", "rhl: 1\ndecls: {\n", None);
    assert_eq!(rep.diagnostics.len(), 1);
    assert_eq!(rep.diagnostics[0].code, "RHL-P004");
    assert_eq!(rep.verdict, Verdict::Fail);
}

#[test]
fn test_rhl3_check_files_dispatches_rhl_yaml_by_extension() {
    let dir = tempfile::tempdir().expect("temp dir");
    let file = dir.path().join("x.rhl.yaml");
    std::fs::write(&file, ONE).expect("write");
    let out = check_files(&[file.as_path()], OutputFormat::Json);
    let v: serde_json::Value = serde_json::from_str(&out.stdout).expect("JSON");
    assert_eq!(v["diagnostics"][0]["code"], "RHL-V001", "{}", out.stdout);
}

#[test]
fn test_rhl3_fmt_on_yaml_re_emits_canonical_yaml() {
    let loose = "decls:\n  - unit: {kind: job, name: j, body: [{expect: {app: {call: {phrase: a}}}}]}\nrhl: 1\n";
    assert_eq!(format_file_source("x.rhl.yaml", loose).as_deref(), Ok(ONE));
    let err = format_file_source("x.rhl.yaml", "rhl: [").expect_err("bad");
    assert!(err.contains("error[RHL-P004]"), "{err}");
}

#[test]
fn test_rhl3_fix_refuses_yaml_input() {
    let out = crate::rhl::fix::fix_files(&[Path::new("x.rhl.yaml")], true, OutputFormat::Text);
    assert_eq!(out.exit, 1);
    assert!(out.stderr.contains("ruchy convert"), "{}", out.stderr);
}

// ───────────────────────── e. properties ─────────────────────────

/// `PROPTEST_CASES` if set, else 100.
fn cases() -> u32 {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(cases()))]

    #[test]
    fn test_rhl3_prop_tree_to_yaml_to_tree(t in arb_program()) {
        let yaml = to_yaml(&t);
        let back = from_yaml(&yaml).map_err(|e| TestCaseError::fail(format!("{e:?}\n{yaml}")))?;
        prop_assert_eq!(to_yaml(&back), yaml.clone());
        prop_assert_eq!(format(&back), format(&t), "{}", yaml);
    }

    #[test]
    fn test_rhl3_prop_rhl_to_yaml_to_rhl(t in arb_program()) {
        let text = format(&t);
        let yaml = convert_source("p.rhl", &text, Target::Yaml).map_err(TestCaseError::fail)?;
        let back = convert_source("p.rhl.yaml", &yaml, Target::Rhl).map_err(TestCaseError::fail)?;
        prop_assert_eq!(back, text);
    }

    #[test]
    fn test_rhl3_prop_arbitrary_text_never_panics(s in "\\PC{0,200}") {
        if let Err(e) = from_yaml(&s) {
            prop_assert_eq!(diagnostic("f.rhl.yaml", &e).code, "RHL-P004");
        }
        let _ = convert_source("f.rhl.yaml", &s, Target::Rhl);
    }

    #[test]
    fn test_rhl3_prop_damaged_yaml_never_panics(t in arb_program(), at in any::<prop::sample::Index>(), with in "[\\[\\]{}:,'\" a-z0-9\n-]{0,3}") {
        let yaml = to_yaml(&t);
        let mut cut = at.index(yaml.len());
        while !yaml.is_char_boundary(cut) {
            cut -= 1;
        }
        let damaged = format!("{}{with}{}", &yaml[..cut], &yaml[cut..].chars().skip(1).collect::<String>());
        match from_yaml(&damaged) {
            Ok(p) => prop_assert_eq!(to_yaml(&from_yaml(&to_yaml(&p)).expect("canonical reads")), to_yaml(&p)),
            Err(e) => prop_assert_eq!(diagnostic("f.rhl.yaml", &e).code, "RHL-P004"),
        }
    }
}
