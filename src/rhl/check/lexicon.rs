//! The words a program may use: the terms of its `use vocabulary` lines, with
//! each entity's declared instances (plan D6), indexed by spelling.

use crate::rhl::vocab::{self, Term, TermKind, Vocabulary};
use std::path::Path;

/// One term of a used vocabulary.
#[derive(Debug, Clone)]
pub(crate) struct LexTerm {
    /// The term as the vocabulary file declares it.
    pub(crate) term: Term,
    /// The vocabulary it comes from, as a program names it: `fleet v1`.
    pub(crate) vocab: String,
    /// For an entity: its declared instances. Empty means no source of truth
    /// exists, so the entity is unverified rather than closed-world.
    pub(crate) instances: Vec<String>,
}

/// Every term of every vocabulary a program uses.
#[derive(Debug, Clone, Default)]
pub(crate) struct Lexicon {
    terms: Vec<LexTerm>,
    labels: Vec<String>,
}

impl Lexicon {
    /// Whether the vocabulary `label` is already loaded.
    pub(crate) fn has(&self, label: &str) -> bool {
        self.labels.iter().any(|l| l == label)
    }

    /// Add every term of `vocab`, reading entity instances under `root`.
    pub(crate) fn add(&mut self, root: &Path, vocab: &Vocabulary) {
        let label = vocab.label();
        for term in &vocab.terms {
            let instances = entity_instances(root, term);
            self.terms.push(LexTerm {
                term: term.clone(),
                vocab: label.clone(),
                instances,
            });
        }
        self.labels.push(label);
    }

    /// The term spelled exactly `text`, if any. The first vocabulary wins.
    pub(crate) fn get(&self, text: &str) -> Option<&LexTerm> {
        self.terms.iter().find(|t| t.term.term == text)
    }

    /// The unit term spelled exactly `text`, if any.
    pub(crate) fn unit(&self, text: &str) -> Option<&LexTerm> {
        self.units().find(|t| t.term.term == text)
    }

    /// Every unit term.
    pub(crate) fn units(&self) -> impl Iterator<Item = &LexTerm> {
        self.terms.iter().filter(|t| t.term.kind == TermKind::Unit)
    }

    /// `(spelling, vocabulary)` for every term: the generic namespace.
    pub(crate) fn term_pool(&self) -> Vec<(String, String)> {
        self.terms
            .iter()
            .map(|t| (t.term.term.clone(), t.vocab.clone()))
            .collect()
    }

    /// `(spelling, vocabulary)` for every unit term: the unit namespace.
    pub(crate) fn unit_pool(&self) -> Vec<(String, String)> {
        self.units()
            .map(|t| (t.term.term.clone(), t.vocab.clone()))
            .collect()
    }

    /// `(target, vocabulary)` for the `<target>` of every term's effect.
    pub(crate) fn effect_target_pool(&self) -> Vec<(String, String)> {
        let mut pool: Vec<(String, String)> = Vec::new();
        for t in &self.terms {
            if let Some((_, target)) = t.term.effect_parts() {
                if !pool.iter().any(|(p, _)| p == target) {
                    pool.push((target.to_string(), t.vocab.clone()));
                }
            }
        }
        pool
    }
}

fn entity_instances(root: &Path, term: &Term) -> Vec<String> {
    match (&term.kind, &term.instances_from) {
        (TermKind::Entity, Some(glob)) => vocab::instances(root, glob),
        _ => Vec::new(),
    }
}
