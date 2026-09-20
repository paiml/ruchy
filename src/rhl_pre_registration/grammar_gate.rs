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

/// Run LALRPOP over one grammar file, keeping its rendered error.
///
/// An earlier revision discarded the error, on the stated ground that LALRPOP
/// renders a conflict and a typo alike as `"invalid data"`. That was measured to
/// be FALSE (lalrpop 0.23.1, 2026-09-20): `invalid data` is specifically the
/// conflict/ambiguity path, while a typo renders as its own message — an
/// undefined nonterminal gives ``no definition found for `X` ``. The message is
/// the sharpest discriminator available, so the positive control now asserts on
/// it directly, and keeps the differential as a second, independent leg.
fn lalrpop_result(path: &Path, tag: &str) -> Result<(), String> {
    lalrpop::Configuration::new()
        .set_out_dir(fresh_out_dir(tag))
        .emit_rerun_directives(false)
        .process_file(path)
        .map_err(|e| e.to_string())
}

fn lalrpop_accepts(path: &Path, tag: &str) -> bool {
    lalrpop_result(path, tag).is_ok()
}

/// LALRPOP's rendered error for a grammar whose only defect is an LR conflict.
/// Measured, not guessed; see [`lalrpop_result`].
const CONFLICT_ERROR: &str = "invalid data";

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

    let err = lalrpop_result(&repo_root().join(FIXTURE), "fixture").expect_err(
        "F1 POSITIVE CONTROL BROKEN: LALRPOP ACCEPTED the ambiguous fixture. This \
         check can no longer detect an ambiguous production, so every \
         conflict-free claim RHL makes is worthless.",
    );
    assert_eq!(
        err, CONFLICT_ERROR,
        "F1 POSITIVE CONTROL IS NOT DISCRIMINATING: {FIXTURE} was rejected, but \
         for `{err}` rather than for an LR conflict. A renamed nonterminal or a \
         stray brace also fails, and would have passed this control for the wrong \
         reason. Fix the fixture so its ONLY defect is the planted ambiguity."
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
    let silencers: Vec<&str> = grammar
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .filter(|l| has_attr(l, "precedence") || has_attr(l, "assoc"))
        .collect();
    assert!(
        silencers.is_empty(),
        "{GRAMMAR} uses a precedence or associativity annotation: {silencers:?}. \
         These RESOLVE an ambiguity silently instead of reporting it, which turns \
         the conflict-free report into a false positive. Rewrite the productions."
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

/// `#[name` allowing whitespace after the bracket, which LALRPOP's parser accepts
/// in some positions. A bare `contains("#[precedence")` missed `#[ precedence`.
fn has_attr(line: &str, name: &str) -> bool {
    line.split("#[")
        .skip(1)
        .any(|rest| rest.trim_start().starts_with(name))
}

/// The one line that can switch this whole instrument off.
///
/// LALRPOP builds LR states only for nonterminals reachable from a `pub` entry
/// point. Moving `pub` from `Program` to a leaf makes almost nothing reachable,
/// and the grammar then processes cleanly however ambiguous it is. MEASURED: with
/// `pub` on `Quantity` and Amendment A1's ambiguity put back in full, LALRPOP
/// returns `Ok` and all the other checks here stay green. Four characters,
/// plausible as part of an honest RHL-1 refactor, and F1's floor is gone.
#[test]
fn test_rhl_grammar_0_program_is_the_only_entry_point() {
    let grammar = read(GRAMMAR);
    let pubs: Vec<&str> = grammar
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("pub ") && l.contains(':'))
        .collect();
    assert_eq!(
        pubs.len(),
        1,
        "{GRAMMAR} declares {} `pub` entry points: {pubs:?}. LALRPOP only builds \\
         states for what a `pub` nonterminal reaches, so more than one entry point \\
         — or the wrong one — changes what 'conflict-free' even ranges over.",
        pubs.len()
    );
    assert!(
        pubs[0].starts_with("pub Program:"),
        "{GRAMMAR}'s entry point is `{}`, not `pub Program`. Everything this gate \\
         certifies is scoped to what the entry point reaches; move it to a leaf \\
         and the grammar processes cleanly however ambiguous it is.",
        pubs[0]
    );
}

/// A floor on what the grammar must still DESCRIBE.
///
/// "Conflict-free" is a property of the empty language too. MEASURED: a grammar
/// keeping this file's comment header and its entire `match` block, but whose
/// only production is `Phrase = WORD+`, passes every other check here — while
/// being unable to parse a single RHL program. So the keywords of spec §3.3 must
/// appear in the PRODUCTIONS, not merely be declared as tokens.
///
/// This is a floor, not a proof. The real check — that the committed corpus
/// parses, and parses exactly once — needs a generated parser, which is row
/// RHL-1's work and is named as a gap in this row's receipt.
#[test]
fn test_rhl_grammar_0_productions_still_cover_the_keyword_set() {
    let grammar = read(GRAMMAR);
    let body = grammar
        .rsplit_once('}')
        .map(|(_, _)| productions_region(&grammar))
        .unwrap_or_default();
    let missing: Vec<&str> = KEYWORDS
        .iter()
        .filter(|k| !body.contains(&format!("\"{k}\"")))
        .copied()
        .collect();
    assert!(
        missing.is_empty(),
        "{GRAMMAR} declares {} §3.3 keyword(s) as tokens but uses them in no \\
         production: {missing:?}. A grammar that lexes RHL and parses none of it \\
         is still conflict-free, and that is not what F1 claims.",
        missing.len()
    );
}

/// Everything after the lexer `match` block — i.e. the productions.
fn productions_region(grammar: &str) -> String {
    match grammar.find("\npub ") {
        Some(i) => grammar[i..].to_string(),
        None => String::new(),
    }
}

/// Spec §3.3's closed keyword set, plus `with` from Amendment A1.
const KEYWORDS: &[&str] = &[
    "use vocabulary",
    "job",
    "command",
    "check",
    "pipeline",
    "shape",
    "let",
    "be",
    "set",
    "to",
    "when",
    "otherwise",
    "end",
    "for each",
    "in",
    "repeat at most",
    "times",
    "until",
    "wait up to",
    "runs on",
    "every",
    "may read",
    "may write",
    "may call",
    "expect",
    "example",
    "given",
    "then",
    "give back",
    "stop with",
    "with",
    "is",
    "is not",
    "is below",
    "is above",
    "is at least",
    "is at most",
    "is one of",
    "contains",
    "and",
    "or",
    "not",
];
