//! RHL-0 vocab gate (spec `RHL-001` §3.4, falsifiers F4 and F6).
//!
//! Holds `vocab/fleet-v1.yaml` and `vocab/tickets-v1.yaml` to the shape
//! RHL-001 §3.4 pre-registers, and enforces "no term without a contract
//! template" itself — binding B3 (`docs/rhl/phase0-bindings.md`) measured
//! that `pv` does not gate `contracts/` in this repo's required check, so a
//! contract file that nobody reads back is theatre.

use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// RHL-001 §3.4's closed `kind:` list. No other value is a term kind.
const CLOSED_KINDS: [&str; 5] = ["noun", "measure", "action", "unit", "entity"];

/// The marker a term's `lowers_to` must carry until RHL-3 binds it.
const UNBOUND_MARKER: &str = "[U]";

#[derive(Debug, Deserialize)]
struct VocabFile {
    vocabulary: String,
    version: u32,
    terms: Vec<Term>,
}

#[derive(Debug, Deserialize)]
struct Term {
    term: String,
    kind: String,
    #[serde(default)]
    instances_from: Option<String>,
    lowers_to: String,
    contract: String,
    /// A term's signature. Deserialised, not ignored: a term whose `takes`,
    /// `gives` or `effect` is absent or malformed would otherwise sail through
    /// every other check, and RHL-1's checker would have nothing to type
    /// against. Found by pre-PR review, 2026-09-20.
    #[serde(default)]
    takes: Option<Vec<Param>>,
    #[serde(default)]
    gives: Option<String>,
    #[serde(default)]
    effect: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Param {
    name: String,
    #[serde(rename = "type")]
    ty: String,
}

/// The two vocabularies RHL-0 pre-registers, relative to `repo_root()`.
fn vocab_relative_paths() -> [&'static str; 2] {
    ["vocab/fleet-v1.yaml", "vocab/tickets-v1.yaml"]
}

fn load_vocab(path: &Path) -> VocabFile {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("RHL-0 vocab gate: cannot read {path:?}: {e}"));
    serde_yaml::from_str(&text)
        .unwrap_or_else(|e| panic!("RHL-0 vocab gate: {path:?} is not valid vocabulary YAML: {e}"))
}

/// Every vocabulary file, parsed, paired with the path it came from.
fn all_vocabs() -> Vec<(PathBuf, VocabFile)> {
    let root = crate::rhl_pre_registration::repo_root();
    vocab_relative_paths()
        .iter()
        .map(|rel| {
            let path = root.join(rel);
            let vocab = load_vocab(&path);
            (path, vocab)
        })
        .collect()
}

/// (a) Both vocab files parse, and each declares `vocabulary`, `version`
/// and a non-empty `terms` list.
#[test]
fn test_rhl_vocab_0_files_parse() {
    for (path, vocab) in all_vocabs() {
        assert!(
            !vocab.vocabulary.is_empty(),
            "{path:?}: `vocabulary` is empty"
        );
        assert_eq!(vocab.version, 1, "{path:?}: `version` must be 1");
        assert!(
            !vocab.terms.is_empty(),
            "{path:?}: `terms` must be non-empty"
        );
    }
}

/// (b) F6: every term's `contract:` path exists on disk and is non-empty —
/// "no term without a contract template", enforced here since CI does not.
#[test]
fn test_rhl_vocab_0_all_terms_have_existing_contract() {
    let root = crate::rhl_pre_registration::repo_root();
    for (path, vocab) in all_vocabs() {
        for term in &vocab.terms {
            assert!(
                !term.contract.trim().is_empty(),
                "{path:?}: term `{}` has an empty contract path (F6)",
                term.term
            );
            let contract_path = root.join(&term.contract);
            assert!(
                contract_path.is_file(),
                "{path:?}: term `{}` names contract `{}`, which does not exist (F6) — \
                 the vocabulary is refused",
                term.term,
                term.contract
            );
        }
    }
}

/// (c) Every term's `kind` is in the closed list; an unknown kind fails.
#[test]
fn test_rhl_vocab_0_kind_is_closed_list() {
    for (path, vocab) in all_vocabs() {
        for term in &vocab.terms {
            assert!(
                CLOSED_KINDS.contains(&term.kind.as_str()),
                "{path:?}: term `{}` has kind `{}`, not one of {CLOSED_KINDS:?}",
                term.term,
                term.kind
            );
        }
    }
}

/// (d) Every term's `lowers_to` is exactly the unbound marker `[U]` — RHL-0
/// must not smuggle in a binding that belongs to RHL-3.
#[test]
fn test_rhl_vocab_0_lowers_to_is_unbound() {
    for (path, vocab) in all_vocabs() {
        for term in &vocab.terms {
            assert_eq!(
                term.lowers_to, UNBOUND_MARKER,
                "{path:?}: term `{}` has `lowers_to: {}`, expected the unbound marker `{UNBOUND_MARKER}` \
                 (binding lowers_to is row RHL-3, not RHL-0)",
                term.term, term.lowers_to
            );
        }
    }
}

/// (e) Term surface forms are unique within a vocabulary, and no term is
/// the empty string.
#[test]
fn test_rhl_vocab_0_terms_unique_and_nonempty() {
    for (path, vocab) in all_vocabs() {
        let mut seen: HashSet<&str> = HashSet::new();
        for term in &vocab.terms {
            assert!(
                !term.term.trim().is_empty(),
                "{path:?}: a term has an empty surface form"
            );
            assert!(
                seen.insert(term.term.as_str()),
                "{path:?}: term `{}` is declared more than once",
                term.term
            );
        }
    }
}

/// (f) Every `entity`-kind term declares `instances_from` (closed world,
/// RHL-001 §3.4).
#[test]
fn test_rhl_vocab_0_entity_terms_declare_instances_from() {
    for (path, vocab) in all_vocabs() {
        for term in &vocab.terms {
            if term.kind != "entity" {
                continue;
            }
            let source = term.instances_from.as_deref().unwrap_or("");
            assert!(
                !source.trim().is_empty(),
                "{path:?}: entity term `{}` has no `instances_from` closed-world source",
                term.term
            );
        }
    }
}

/// (g) Every contract file named by a term parses as YAML and carries a
/// `metadata` block.
#[test]
fn test_rhl_vocab_0_contract_files_parse_with_metadata() {
    let root = crate::rhl_pre_registration::repo_root();
    for (path, vocab) in all_vocabs() {
        for term in &vocab.terms {
            let contract_path = root.join(&term.contract);
            let text = fs::read_to_string(&contract_path).unwrap_or_else(|e| {
                panic!(
                    "{path:?}: term `{}`: cannot read `{}`: {e}",
                    term.term, term.contract
                )
            });
            let value: serde_yaml::Value = serde_yaml::from_str(&text).unwrap_or_else(|e| {
                panic!(
                    "{path:?}: term `{}`: contract `{}` is not valid YAML: {e}",
                    term.term, term.contract
                )
            });
            assert!(
                value.get("metadata").is_some(),
                "{path:?}: term `{}`: contract `{}` has no `metadata` block",
                term.term,
                term.contract
            );
        }
    }
}

/// The signature problems of one term, if any. Split out so the test that calls
/// it stays a single assertion rather than a nest of conditions.
fn signature_problems(path: &str, t: &Term) -> Vec<String> {
    let mut out = Vec::new();
    if t.gives.as_deref().unwrap_or("").trim().is_empty() {
        out.push(format!("{path}: `{}` declares no `gives` type", t.term));
    }
    if t.effect.as_deref().is_some_and(|e| e.trim().is_empty()) {
        out.push(format!("{path}: `{}` declares an empty `effect`", t.term));
    }
    out.extend(empty_params(path, t));
    out
}

fn empty_params(path: &str, t: &Term) -> Vec<String> {
    t.takes
        .iter()
        .flatten()
        .filter(|p| p.name.trim().is_empty() || p.ty.trim().is_empty())
        .map(|_| {
            format!(
                "{path}: `{}` has a parameter with an empty name or type",
                t.term
            )
        })
        .collect()
}

/// Every non-entity term must carry a usable signature. `entity` terms are the
/// exception by design: they name a closed-world set via `instances_from` and
/// take no arguments (spec §3.4).
#[test]
fn test_rhl_vocab_0_terms_declare_a_usable_signature() {
    let mut bad = Vec::new();
    for (path, vocab) in all_vocabs() {
        let path = path.display().to_string();
        for t in vocab.terms.iter().filter(|t| t.kind != "entity") {
            bad.extend(signature_problems(&path, t));
        }
    }
    assert!(
        bad.is_empty(),
        "RHL-0: {} term(s) carry no usable signature: {bad:#?}. §3.4 gives every \
         term a type and an effect; a term without one cannot be type-checked at \
         RHL-1 and cannot have its effect surface enforced at all.",
        bad.len()
    );
}

/// Every `discharged_by` either resolves to a real test, or carries a declared
/// sentinel from a CLOSED set.
///
/// RHL-0 answered `pv validate`'s demand for a `kani_harnesses` block — on
/// invariants Kani cannot reach, because they are properties of bytes on disk —
/// by marking each harness `status: declared_not_implemented` and adding
/// `discharged_by:` naming the test that really discharges it.
///
/// The apex session, which closed the identical defect class one field over
/// (`scripts/ci/theorem-bind.sh`, apex#57), pointed out two things. First, a
/// `discharged_by` naming nothing reads as a discharge and is worse than an
/// honest "not built" — so it must resolve. Second, and this is the one I had
/// wrong: a gate that ONLY refuses a dangling name pushes the next author toward
/// naming *some* test to go green, which is exactly how apex's original defect
/// happened (`lean_theorem: Theorems.RowMerged` copied into every contract,
/// resolving to nothing, the validator checking presence only, the gate green).
///
/// **Forcing a citation where none can honestly exist is the failure mode.** So
/// there is a closed set of sentinels giving "there is legitimately nothing here"
/// a way to be said and still be checked, and a sentinel OUTSIDE the set is as
/// red as a dangling name — otherwise the set is advisory.
const SENTINELS: &[&str] = &[
    // The obligation is a fact about THIS REPOSITORY — bytes on disk, a file
    // count, a generator's verdict — so no proof or property test can exist and
    // inventing one would be the defect this gate catches. Every RHL/SEC Kani
    // harness is this shape.
    "none_repository_fact",
    // Discharged outside this repository. Reserved, zero uses today; kept so the
    // vocabulary exists the day an external discharge does.
    "declared_external",
];

#[test]
fn test_rhl_vocab_0_every_discharged_by_resolves_or_declares() {
    let declared = declared_test_fns();
    let mut bad = Vec::new();
    let mut resolved = 0usize;
    let mut sentinels = 0usize;
    for (name, text) in rhl_and_sec_contracts() {
        for cited in cited_tests(&text) {
            match classify(&cited, &declared) {
                Citation::Resolved => resolved += 1,
                Citation::Sentinel => sentinels += 1,
                Citation::Dangling if cited.is_empty() => bad.push(format!(
                    "{name}: a `discharged_by:` with NO VALUE — a citation that cites nothing"
                )),
                Citation::Dangling => bad.push(format!(
                    "{name}: discharged_by `{cited}` — names no #[test] fn under src/ or tests/, \
                     and is not one of {SENTINELS:?}"
                )),
            }
        }
    }
    assert!(
        bad.is_empty(),
        "RHL-13: {} citation(s) neither resolve nor declare: {bad:#?}. A \
         `discharged_by` naming nothing reads as a discharge. If nothing can \
         discharge it, say so with a declared sentinel — do not invent a test name \
         to go green.",
        bad.len()
    );
    // apex's third arm: all-sentinel is a vocabulary, not a binding.
    assert!(
        resolved >= 1,
        "RHL-13: {sentinels} citation(s) and not one resolves to a real test. An \
         all-sentinel binding records that nothing is discharged anywhere, which \
         is not a binding at all."
    );
}

enum Citation {
    Resolved,
    Sentinel,
    Dangling,
}

fn classify(cited: &str, declared: &std::collections::HashSet<String>) -> Citation {
    if SENTINELS.contains(&cited) {
        Citation::Sentinel
    } else if declared.contains(cited) {
        Citation::Resolved
    } else {
        Citation::Dangling
    }
}

/// Every test function DECLARED in the crate, by name.
///
/// An earlier revision concatenated every `.rs` file and asked whether the text
/// contained `fn <cited>`. A two-family review measured what that admits:
///
///   `test_rhl_vocab_0`          resolves — a strict PREFIX of a real test
///   `test_function`             resolves — it exists only inside a string literal
///   `is_palindrome`             resolves — it exists only inside a doc comment
///   `main`, `new`               resolve — real functions, but not tests
///
/// 86 names appear as `fn X` in non-declaration positions in this crate, and 33
/// appear in comments with no definition anywhere. apex's `theorem-bind.sh` —
/// which this gate's design was taken from — does not have that hole: it resolves
/// with `grep -qxF` against a CURATED LEDGER of proved theorems. The port turned
/// exact-match-against-a-registry into substring-match-against-raw-text, which is
/// the same class of defect the gate exists to catch.
///
/// So: parse declaration positions only, and require the citation to be a
/// `#[test]` function — `main` and `new` are real, and neither discharges
/// anything.
fn declared_test_fns() -> std::collections::HashSet<String> {
    let root = crate::rhl_pre_registration::repo_root();
    let mut out = std::collections::HashSet::new();
    for dir in ["src", "tests"] {
        for e in walkdir::WalkDir::new(root.join(dir)).into_iter().flatten() {
            if e.path().extension().is_some_and(|x| x == "rs") {
                let text = std::fs::read_to_string(e.path()).unwrap_or_default();
                collect_test_fns(&text, &mut out);
            }
        }
    }
    out
}

/// `#[test]`-annotated function names in one source file, declaration position
/// only. A line inside a comment or a string is not a declaration.
fn collect_test_fns(text: &str, out: &mut std::collections::HashSet<String>) {
    let mut armed = false;
    for raw in text.lines() {
        armed = advance(raw.trim(), armed, out);
    }
}

/// One line of the walk: returns whether a `#[test]` attribute is still in scope
/// for the next line. Split out to keep the walk itself a single loop.
fn advance(line: &str, armed: bool, out: &mut std::collections::HashSet<String>) -> bool {
    match classify_line(line) {
        Line::Skip => armed,
        Line::TestAttr => true,
        Line::Fn(name) => keep(armed, name, out),
        Line::Other => armed && inert(line),
    }
}

/// Record the name when a `#[test]` was in scope, and disarm either way.
fn keep(armed: bool, name: String, out: &mut std::collections::HashSet<String>) -> bool {
    if armed {
        out.insert(name);
    }
    false
}

/// A line that neither declares nor disarms — blank, or another attribute.
fn inert(line: &str) -> bool {
    line.is_empty() || line.starts_with("#[")
}

enum Line {
    /// A comment, doc comment or bare string — never a declaration.
    Skip,
    /// `#[test]` or `#[tokio::test…]`, arming the next declaration.
    TestAttr,
    /// A function declaration, with its name.
    Fn(String),
    Other,
}

fn classify_line(line: &str) -> Line {
    if line.starts_with("//") || line.starts_with('*') || line.starts_with('"') {
        return Line::Skip;
    }
    if line.starts_with("#[test]") || line.starts_with("#[tokio::test") {
        return Line::TestAttr;
    }
    fn_name(line).map_or(Line::Other, Line::Fn)
}

/// The declared name in `[pub] [async] [const] [unsafe] fn NAME(` — or nothing.
fn fn_name(line: &str) -> Option<String> {
    let rest = line.split_once("fn ")?.1;
    let before = line.split("fn ").next()?;
    let ok = before.split_whitespace().all(|w| {
        matches!(w, "pub" | "async" | "const" | "unsafe" | "extern" | "") || w.starts_with("pub(")
    });
    if !ok {
        return None;
    }
    let name: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    let terminated = rest[name.len()..].starts_with('(') || rest[name.len()..].starts_with('<');
    (!name.is_empty() && terminated).then_some(name)
}

/// The RHL and SEC contracts, as (filename, text).
fn rhl_and_sec_contracts() -> Vec<(String, String)> {
    let root = crate::rhl_pre_registration::repo_root();
    std::fs::read_dir(root.join("contracts"))
        .expect("RHL-13: contracts/ is missing")
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            let n = p
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            n.starts_with("rhl-") || n.starts_with("sec-")
        })
        .map(|p| {
            let n = p
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let text = std::fs::read_to_string(&p).unwrap_or_else(|e| {
                panic!(
                    "RHL-13: cannot read contracts/{n}: {e}. An unreadable contract must \
                        not read as 'no citations here' — that is how a dangling citation \
                        goes unexamined."
                )
            });
            (n, text)
        })
        .collect()
}

/// Every `discharged_by:` value in a contract, unquoted.
///
/// An empty value is RETURNED, not filtered. apex's `theorem-bind.sh` refuses it
/// by name — `fail "(empty) … has a lean_theorem with no value"` — and an earlier
/// revision here dropped it with `.filter(|v| !v.is_empty())`, so a key written
/// with no value, or as a YAML list, vanished from the count entirely: the gate
/// stayed green while examining one fewer citation than the file contains. A
/// citation that is never examined is worse than one that fails.
fn cited_tests(contract: &str) -> Vec<String> {
    contract
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .filter_map(|l| l.split_once("discharged_by"))
        .filter_map(|(_, v)| v.trim_start().strip_prefix(':'))
        .map(|v| v.trim().trim_matches('"').trim_matches('\'').to_string())
        .collect()
}
