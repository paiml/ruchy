//! The checker's state and the diagnostics it builds: unknown words with
//! candidates (plan D4) and undeclared effects (plan D5).

use super::lexicon::{LexTerm, Lexicon};
use super::near;
use super::types::Val;
use super::Unverified;
use crate::rhl::codes;
use crate::rhl::diag::{Candidate, Diagnostic, Edit, Expected, Fix, LineSpan};
use crate::rhl::tree::{App, Atom, Phrase, Span, Unit, Word};

/// One use of a term that has an effect.
#[derive(Debug, Clone)]
struct EffectUse {
    term: String,
    effect: (String, String),
    span: Span,
}

/// Walks one program and collects its diagnostics.
pub(crate) struct Checker<'a> {
    pub(crate) file: &'a str,
    pub(crate) source: &'a str,
    pub(crate) lex: &'a Lexicon,
    pub(crate) diags: Vec<Diagnostic>,
    pub(crate) unverified: Vec<Unverified>,
    scope: Vec<(String, Val)>,
    uses: Vec<EffectUse>,
    declared: Vec<(String, String)>,
}

impl<'a> Checker<'a> {
    /// A checker for `source` (named `file`) over the terms of `lex`.
    pub(crate) fn new(file: &'a str, source: &'a str, lex: &'a Lexicon) -> Self {
        Self {
            file,
            source,
            lex,
            diags: Vec::new(),
            unverified: Vec::new(),
            scope: Vec::new(),
            uses: Vec::new(),
            declared: Vec::new(),
        }
    }

    /// Check one unit. `let` names and `may` lines are unit-scoped.
    pub(crate) fn unit(&mut self, unit: &Unit) {
        self.scope.clear();
        self.uses.clear();
        self.declared.clear();
        self.body(&unit.body, false);
        self.effects(unit);
    }

    /// Push an error with `code` at `span`.
    pub(crate) fn error(&mut self, code: &str, span: Span, message: String) {
        let d = Diagnostic::error(code, self.file, self.source, span, message);
        self.diags.push(d);
    }

    /// The value bound to the `let` name `name`, if one is in scope.
    pub(crate) fn lookup_let(&self, name: &str) -> Option<Val> {
        self.scope
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.clone())
    }

    /// Bind the `let` name `name` for the rest of the unit.
    pub(crate) fn bind(&mut self, name: String, value: Val) {
        self.scope.push((name, value));
    }

    /// Record `may <verb> <target>`.
    pub(crate) fn declare(&mut self, verb: &str, target: String) {
        self.declared.push((verb.to_string(), target));
    }

    /// The generic namespace: every term, then every `let` name in scope.
    pub(crate) fn generic_pool(&self) -> Vec<(String, String)> {
        let mut pool = self.lex.term_pool();
        pool.extend(
            self.scope
                .iter()
                .map(|(n, _)| (n.clone(), "let".to_string())),
        );
        pool
    }

    /// Record a use of `term` at `span`, for the effect check.
    pub(crate) fn use_term(&mut self, term: &LexTerm, span: Span) {
        if let Some((verb, target)) = term.term.effect_parts() {
            self.uses.push(EffectUse {
                term: term.term.term.clone(),
                effect: (verb.to_string(), target.to_string()),
                span,
            });
        }
    }

    /// Record an instance of an entity with no source of truth (plan D6).
    pub(crate) fn note_unverified(&mut self, term: &LexTerm, instance: &str) {
        let entry = Unverified {
            entity: term.term.term.clone(),
            instance: instance.to_string(),
            source_glob: term.term.instances_from.clone().unwrap_or_default(),
        };
        if !self.unverified.contains(&entry) {
            self.unverified.push(entry);
        }
    }

    /// Report `words` as unknown in the namespace `pool` (named `kind`):
    /// RHL-V001, V002 or V003 by the number of candidates (plan D4).
    pub(crate) fn unknown(&mut self, words: &[Word], pool: &[(String, String)], kind: &str) -> Val {
        let texts: Vec<String> = words.iter().map(|w| w.text.clone()).collect();
        let d = match near::search_prefixes(&texts, pool) {
            None => self.unknown_diag(codes::V001, words, Vec::new(), kind),
            Some((k, found)) if found.iter().any(|c| c.distance == 0) => {
                self.extra_words(&words[..k], &words[k..])
            }
            Some((k, found)) => {
                let code = if found.len() == 1 {
                    codes::V002
                } else {
                    codes::V003
                };
                self.unknown_diag(code, &words[..k], found, kind)
            }
        };
        self.diags.push(d);
        Val::unknown()
    }

    /// `known` is spelled exactly (a `let` name: a term would have resolved
    /// before the candidate search) and `extra` are words it cannot take.
    /// That is a misuse (RHL-T002), not an unknown word: a distance-0
    /// "candidate" would make a V002 whose safe fix changes nothing.
    fn extra_words(&self, known: &[Word], extra: &[Word]) -> Diagnostic {
        let message = format!(
            "`{}` takes no further words, found `{}`",
            words_text(known),
            words_text(extra)
        );
        Diagnostic::error(
            codes::T002,
            self.file,
            self.source,
            words_span(extra),
            message,
        )
    }

    fn unknown_diag(
        &self,
        code: &str,
        words: &[Word],
        found: Vec<Candidate>,
        kind: &str,
    ) -> Diagnostic {
        let written = words_text(words);
        let message = format!("unknown {kind} `{written}`");
        let mut d = Diagnostic::error(code, self.file, self.source, words_span(words), message)
            .with_expected(Expected {
                kind: Some(kind.to_string()),
                ..Expected::default()
            });
        if let [only] = found.as_slice() {
            d.fixes.push(Fix {
                title: format!("replace with `{}`", only.term),
                safe: true,
                edits: vec![Edit {
                    span: d.span,
                    text: only.term.clone(),
                }],
            });
        }
        d.candidates = found;
        d
    }

    /// RHL-E001 for every effect used in `unit` that no `may` line declares.
    fn effects(&mut self, unit: &Unit) {
        let missing: Vec<EffectUse> = self
            .uses
            .iter()
            .filter(|u| !self.declared.contains(&u.effect))
            .cloned()
            .collect();
        let header = LineSpan::from_span(self.source, Span::new(unit.span.start, unit.span.start));
        for u in missing {
            let d = self.effect_diag(&u, header.line + 1);
            self.diags.push(d);
        }
    }

    fn effect_diag(&self, u: &EffectUse, insert_line: usize) -> Diagnostic {
        let (verb, target) = &u.effect;
        let line = format!("may {verb} {target}");
        let message = format!(
            "`{}` has the effect `{verb} {target}`, which this unit does not declare; add `{line}`",
            u.term
        );
        let mut d = Diagnostic::error(codes::E001, self.file, self.source, u.span, message);
        d.fixes.push(Fix {
            title: format!("declare `{line}`"),
            safe: false,
            edits: vec![Edit {
                span: LineSpan {
                    line: insert_line,
                    col: 1,
                    len: 0,
                },
                text: format!("  {line}\n"),
            }],
        });
        d
    }
}

/// The span from the first word to the last. `words` is never empty.
pub(crate) fn words_span(words: &[Word]) -> Span {
    let first = words.first().map_or(Span::default(), |w| w.span);
    words.iter().fold(first, |s, w| s.to(w.span))
}

/// The words joined by single spaces.
pub(crate) fn words_text(words: &[Word]) -> String {
    words
        .iter()
        .map(|w| w.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The span of an application.
pub(crate) fn app_span(app: &App) -> Span {
    match app {
        App::Call { phrase, args } => args.iter().fold(phrase.span, |s, a| s.to(atom_span(a))),
        App::Quantity(q) => q.span,
        App::Text(t) => t.span,
    }
}

/// The span of an argument.
pub(crate) fn atom_span(atom: &Atom) -> Span {
    match atom {
        Atom::Text(t) => t.span,
        Atom::Quantity(q) => q.span,
    }
}

/// The longest word-prefix of `phrase` that is a term: `(words used, term)`.
pub(crate) fn longest_term<'l>(lex: &'l Lexicon, phrase: &Phrase) -> Option<(usize, &'l LexTerm)> {
    (1..=phrase.words.len())
        .rev()
        .find_map(|k| lex.get(&words_text(&phrase.words[..k])).map(|t| (k, t)))
}
