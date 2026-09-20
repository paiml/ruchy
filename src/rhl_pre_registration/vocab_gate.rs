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
