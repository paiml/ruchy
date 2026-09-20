//! RHL-0 grammar gate — falsifier F1, "one meaning" (spec RHL-001 §7).
//!
//! F1's floor is that the grammar is "proven conflict-free by the generator's
//! report", and its positive control is that a deliberately ambiguous production
//! turns this check RED. Both are asserted here, by running LALRPOP itself. The
//! judge is the generator, never a checker we wrote.

use super::repo_root;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const GRAMMAR: &str = "grammar/rhl.lalrpop";
const FIXTURE: &str = "grammar/fixtures/ambiguous.lalrpop";
const REPORT: &str = "grammar/rhl.conflict-report.txt";

/// The one line `grammar/fixtures/ambiguous.lalrpop` marks as its planted
/// defect. Deleting it, and nothing else, must make the fixture process
/// cleanly — that is what attributes the rejection to the ambiguity.
const PLANTED_MARKER: &str = "UNMARKED attribute block";

fn read(rel: &str) -> String {
    let p = repo_root().join(rel);
    std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("RHL-0: cannot read {}: {e}", p.display()))
}

fn sha256_of(text: &str) -> String {
    let mut h = Sha256::new();
    h.update(text.as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// A fresh, unshared output directory, so two gate tests running in parallel
/// never race on LALRPOP's generated files.
fn fresh_out_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("rhl0-grammar-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("RHL-0: cannot create a temp out dir");
    d
}

/// Run LALRPOP over one grammar file and report only whether it accepted it.
///
/// The error value is deliberately discarded: LALRPOP renders both a genuine
/// conflict and a plain typo as the string "invalid data" (measured, lalrpop
/// 0.23), so the message cannot discriminate. The differential in
/// [`test_rhl_grammar_0_ambiguous_fixture_is_rejected_for_being_ambiguous`]
/// does that job instead.
fn lalrpop_accepts(path: &Path, tag: &str) -> bool {
    lalrpop::Configuration::new()
        .set_out_dir(fresh_out_dir(tag))
        .emit_rerun_directives(false)
        .process_file(path)
        .is_ok()
}

#[test]
fn test_rhl_grammar_0_is_conflict_free_by_the_generators_own_report() {
    let path = repo_root().join(GRAMMAR);
    assert!(
        lalrpop_accepts(&path, "subject"),
        "F1 FLOOR BROKEN: LALRPOP rejected {GRAMMAR}. The grammar is no longer \
         proven conflict-free, so RHL's claim that every valid program has \
         exactly one parse has nothing behind it. Re-run the generator and read \
         its conflict report; do NOT reach for #[precedence] to silence it."
    );
}

#[test]
fn test_rhl_grammar_0_ambiguous_fixture_is_rejected_for_being_ambiguous() {
    let fixture = read(FIXTURE);
    let planted: Vec<&str> = fixture
        .lines()
        .filter(|l| l.contains(PLANTED_MARKER) && !l.trim_start().starts_with("//"))
        .collect();
    assert_eq!(
        planted.len(),
        1,
        "F1 POSITIVE CONTROL BROKEN: {FIXTURE} must contain exactly one \
         production marked `{PLANTED_MARKER}`, found {}. The differential below \
         is meaningless without it.",
        planted.len()
    );

    assert!(
        !lalrpop_accepts(&repo_root().join(FIXTURE), "fixture"),
        "F1 POSITIVE CONTROL BROKEN: LALRPOP ACCEPTED {FIXTURE}. This check can \
         no longer detect an ambiguous production, so every conflict-free claim \
         RHL makes is worthless."
    );

    // The differential: delete the one planted production and nothing else.
    // If the trimmed grammar is accepted, the rejection above is attributable
    // to the planted ambiguity — not to a typo, a rename or a bad edit, each of
    // which would fail here too.
    let trimmed: String = fixture
        .lines()
        .filter(|l| !l.contains(PLANTED_MARKER) || l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let control = fresh_out_dir("control-src").join("control.lalrpop");
    std::fs::write(&control, trimmed).expect("RHL-0: cannot write the control grammar");
    assert!(
        lalrpop_accepts(&control, "control"),
        "F1 POSITIVE CONTROL IS NOT DISCRIMINATING: {FIXTURE} still fails once \
         its planted production is removed, so its rejection proves nothing \
         about ambiguity. Something else in the fixture is broken — fix that \
         before trusting this gate."
    );
}

#[test]
fn test_rhl_grammar_0_committed_conflict_report_matches_the_grammar() {
    let report = read(REPORT);
    let actual = sha256_of(&read(GRAMMAR));
    assert!(
        report.contains(&actual),
        "RHL-0 done_when names the conflict-free report as a committed \
         deliverable, and {REPORT} no longer records the sha256 of {GRAMMAR} \
         ({actual}). Either the grammar changed without the report being \
         regenerated, or the report was edited. Re-run the generator and rewrite \
         the report; a report that does not match its subject is decoration."
    );
}

#[test]
fn test_rhl_grammar_0_conflict_freedom_is_not_bought_with_precedence() {
    let grammar = read(GRAMMAR);
    assert!(
        !grammar.contains("#[precedence"),
        "{GRAMMAR} uses a #[precedence] annotation. Precedence RESOLVES an \
         ambiguity silently instead of reporting it, which turns the \
         conflict-free report into a false positive. Rewrite the productions."
    );
    // `else` is legitimate exactly once per level in LALRPOP's lexer `match`
    // block. Anywhere else it is precedence chaining between alternatives,
    // which buys conflict-freedom the same dishonest way.
    let stray: Vec<&str> = grammar
        .lines()
        .filter(|l| {
            l.contains("else") && l.trim() != "} else {" && !l.trim_start().starts_with("//")
        })
        .collect();
    assert!(
        stray.is_empty(),
        "{GRAMMAR} uses `else` outside the lexer `match` block: {stray:?}. \
         Between alternatives that is precedence chaining, and it hides \
         ambiguity rather than removing it."
    );
}

#[test]
fn test_rhl_grammar_0_tokenization_contract_is_stated() {
    let grammar = read(GRAMMAR);
    for required in [
        "TOKENIZATION CONTRACT",
        "NEWLINE IS SIGNIFICANT",
        "KEYWORDS ARE RESERVED AND CLOSED",
        "THE GRAMMAR DOES NOT KNOW ANY VOCABULARY TERM",
        "PHRASE = WORD+",
    ] {
        assert!(
            grammar.contains(required),
            "{GRAMMAR} no longer states its tokenization contract (missing \
             `{required}`). A reader who cannot tell how `disk free of` is \
             tokenized cannot tell whether the grammar means what it says."
        );
    }
}
