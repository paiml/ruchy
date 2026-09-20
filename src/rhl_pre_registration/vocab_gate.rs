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

/// A `discharged_by` that names no real test is a citation nothing resolves.
///
/// RHL-0 answered `pv validate`'s demand for a `kani_harnesses` block — on
/// invariants Kani cannot reach, because they are properties of bytes on disk —
/// by marking each harness `status: declared_not_implemented` and adding
/// `discharged_by:` naming the test that really discharges it. The apex session,
/// which has the nearest equivalent (`implemented: false` + a `note:`), pointed
/// out that this is strictly more informative than theirs ONLY while the named
/// test exists and can fail. Otherwise it is a worse lie than saying nothing,
/// because it reads as a discharge. Nothing checked it until now.
#[test]
fn test_rhl_vocab_0_every_discharged_by_names_a_real_test() {
    let defined = all_test_fn_source();
    let dangling: Vec<String> = rhl_and_sec_contracts()
        .into_iter()
        .flat_map(|(name, text)| {
            cited_tests(&text)
                .into_iter()
                .map(move |c| (name.clone(), c))
        })
        .filter(|(_, cited)| !defined.contains(&format!("fn {cited}")))
        .map(|(name, cited)| format!("{name}: discharged_by `{cited}` — no such test"))
        .collect();
    assert!(
        dangling.is_empty(),
        "RHL-13: {} contract(s) cite a discharging test that does not exist: \
         {dangling:#?}. A `discharged_by` naming nothing reads as a discharge and \
         is worse than an honest `not built`.",
        dangling.len()
    );
}

/// Every `.rs` source in the crate, concatenated — the haystack for `fn <name>`.
fn all_test_fn_source() -> String {
    let root = crate::rhl_pre_registration::repo_root();
    walkdir::WalkDir::new(root.join("src"))
        .into_iter()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "rs"))
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .collect()
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
            (n, std::fs::read_to_string(&p).unwrap_or_default())
        })
        .collect()
}

/// Every `discharged_by:` value in a contract, unquoted.
fn cited_tests(contract: &str) -> Vec<String> {
    contract
        .lines()
        .filter_map(|l| l.trim().strip_prefix("discharged_by:"))
        .map(|v| v.trim().trim_matches('"').to_string())
        .filter(|v| !v.is_empty())
        .collect()
}
