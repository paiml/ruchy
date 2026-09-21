//! Values: resolving a phrase to a `let` name, a term or an entity instance,
//! and typing conditions (spec RHL-001 §3.3; plan D3, D4, D6).

use super::checker::{app_span, atom_span, longest_term, words_span, Checker};
use super::lexicon::LexTerm;
use super::types::{compare_code, Ty, Val};
use crate::rhl::codes;
use crate::rhl::tree::{App, Atom, CompOp, Cond, CondKind, Phrase, Quantity, Span, Word};
use crate::rhl::vocab::{Param, TermKind};

/// The namespace name of a generic `App` position.
const GENERIC: &str = "term or let name";

impl Checker<'_> {
    /// The value of an application in a generic position.
    pub(crate) fn app(&mut self, app: &App) -> Val {
        match app {
            App::Call { phrase, args } => self.call(phrase, args),
            App::Quantity(q) => self.quantity(q),
            App::Text(_) => Val::of(Ty::Text),
        }
    }

    /// `INT [unit]`: the unit comes from the unit namespace, or is `%`.
    pub(crate) fn quantity(&mut self, q: &Quantity) -> Val {
        let Some(unit) = &q.unit else {
            return Val {
                ty: Ty::Count,
                bare: true,
            };
        };
        if unit.text == "%" {
            return Val::of(Ty::Percent);
        }
        match self.lex.unit(&unit.text) {
            Some(t) => Val::of(gives(t)),
            None => {
                let pool = self.lex.unit_pool();
                self.unknown(std::slice::from_ref(unit), &pool, "unit")
            }
        }
    }

    fn call(&mut self, phrase: &Phrase, args: &[Atom]) -> Val {
        if let Some(v) = self.let_value(phrase, args) {
            return v;
        }
        let lex = self.lex;
        match longest_term(lex, phrase) {
            Some((k, term)) => self.apply(term, phrase, k, args),
            None => {
                let pool = self.generic_pool();
                self.unknown(&phrase.words, &pool, GENERIC)
            }
        }
    }

    fn let_value(&mut self, phrase: &Phrase, args: &[Atom]) -> Option<Val> {
        let value = self.lookup_let(&phrase.text())?;
        if args.is_empty() {
            return Some(value);
        }
        let message = format!("`{}` is a `let` name and takes no arguments", phrase.text());
        self.error(codes::T002, phrase.span, message);
        Some(Val::unknown())
    }

    /// Apply `term`, which spells the first `k` words of `phrase`.
    fn apply(&mut self, term: &LexTerm, phrase: &Phrase, k: usize, args: &[Atom]) -> Val {
        let rest = &phrase.words[k..];
        let name = &term.term.term;
        match term.term.kind {
            TermKind::Entity => self.entity(term, rest, args, phrase.span),
            TermKind::Unit => {
                let message =
                    format!("the unit `{name}` needs a number before it, as in `1 {name}`");
                self.error(codes::T002, phrase.span, message);
                Val::unknown()
            }
            _ if !rest.is_empty() => {
                let message = format!("`{name}` takes no words after it");
                self.error(codes::T002, words_span(rest), message);
                Val::unknown()
            }
            _ => {
                self.check_args(name, &term.term.takes, args, phrase.span);
                self.value_term(term, phrase.span)
            }
        }
    }

    /// Record the use of a measure, noun or action and give its type.
    fn value_term(&mut self, term: &LexTerm, span: Span) -> Val {
        self.use_term(term, span);
        Val::of(gives(term))
    }

    /// Check `args` against `params` by count and type.
    pub(crate) fn check_args(&mut self, name: &str, params: &[Param], args: &[Atom], span: Span) {
        if params.len() != args.len() {
            let message = format!(
                "`{name}` takes {} argument(s), but {} are given",
                params.len(),
                args.len()
            );
            return self.error(codes::T002, span, message);
        }
        for (p, a) in params.iter().zip(args) {
            if !self.atom_fits(p, a) {
                let message = format!("`{name}` needs a {} for `{}`", p.ty, p.name);
                self.error(codes::T002, atom_span(a), message);
            }
        }
    }

    fn atom_fits(&mut self, param: &Param, atom: &Atom) -> bool {
        let v = match atom {
            Atom::Text(_) => Val::of(Ty::Text),
            Atom::Quantity(q) => self.quantity(q),
        };
        v.is_unknown() || v.ty == Ty::from_name(&param.ty)
    }

    fn entity(&mut self, term: &LexTerm, rest: &[Word], args: &[Atom], span: Span) -> Val {
        let Some(instance) = instance_word(rest, args) else {
            let name = &term.term.term;
            let message = format!("`{name}` needs one instance name, as in `{name} <name>`");
            self.error(codes::T002, span, message);
            return Val::unknown();
        };
        let known = Val::of(Ty::Entity(term.term.term.clone()));
        if term.instances.is_empty() {
            self.note_unverified(term, &instance.text);
            return known;
        }
        if term.instances.contains(&instance.text) {
            return known;
        }
        let pool: Vec<(String, String)> = term
            .instances
            .iter()
            .map(|i| (i.clone(), term.vocab.clone()))
            .collect();
        let kind = format!("{} instance", term.term.term);
        self.unknown(std::slice::from_ref(&instance), &pool, &kind)
    }

    /// The value of a condition.
    pub(crate) fn cond(&mut self, cond: &Cond) -> Val {
        match &cond.kind {
            CondKind::Or(a, b) | CondKind::And(a, b) => {
                self.condition(a);
                self.condition(b);
                Val::of(Ty::Bool)
            }
            CondKind::Not(c) => {
                self.condition(c);
                Val::of(Ty::Bool)
            }
            CondKind::Compare { left, op, right } => self.compare(left, *op, right, cond.span),
            CondKind::In { left, right } => self.in_cond(left, right, cond.span),
            CondKind::App(a) => self.app(a),
        }
    }

    /// Check a condition that must be Bool.
    pub(crate) fn condition(&mut self, cond: &Cond) {
        let v = self.cond(cond);
        if !v.is_unknown() && v.ty != Ty::Bool {
            let message = format!("a condition must be a comparison, not a {}", v.ty.name());
            self.error(codes::T002, cond.span, message);
        }
    }

    fn compare(&mut self, left: &App, op: CompOp, right: &App, span: Span) -> Val {
        let l = self.app(left);
        let r = self.app(right);
        if let Some(code) = compare_code(op, &l, &r) {
            let message = format!(
                "`{}` cannot compare a {} with a {}",
                op.keyword(),
                l.ty.name(),
                r.ty.name()
            );
            self.error(code, span, message);
        }
        Val::of(Ty::Bool)
    }

    /// `<left> in <right>`: a term ending in `in`, such as `tickets filed in`,
    /// applied to `right`.
    fn in_cond(&mut self, left: &App, right: &App, span: Span) -> Val {
        let phrase = match left {
            App::Call { phrase, args } if args.is_empty() => phrase,
            _ => return self.misplaced_in(left, right, span),
        };
        let lex = self.lex;
        if let Some(term) = lex.get(&format!("{} in", phrase.text())) {
            return self.apply_in(term, right, span);
        }
        if self.resolves(phrase) {
            return self.misplaced_in(left, right, span);
        }
        self.app(right);
        let mut words = phrase.words.clone();
        words.push(self.in_word(phrase.span.end, app_span(right).start));
        let pool = self.generic_pool();
        self.unknown(&words, &pool, GENERIC)
    }

    fn apply_in(&mut self, term: &LexTerm, right: &App, span: Span) -> Val {
        if !matches!(
            term.term.kind,
            TermKind::Measure | TermKind::Noun | TermKind::Action
        ) {
            let message = format!("`{}` cannot be applied with `in`", term.term.term);
            self.error(codes::T002, span, message);
            return Val::unknown();
        }
        match right {
            App::Text(t) => self.check_args(
                &term.term.term,
                &term.term.takes,
                &[Atom::Text(t.clone())],
                span,
            ),
            App::Quantity(q) => {
                let args = [Atom::Quantity(q.clone())];
                self.check_args(&term.term.term, &term.term.takes, &args, span);
            }
            App::Call { .. } => self.entity_argument(term, right),
        }
        self.value_term(term, span)
    }

    /// `right` supplies the single parameter of `term` whose name is the
    /// entity's term name, for example `repo` for `repo "paiml/infra"`.
    fn entity_argument(&mut self, term: &LexTerm, right: &App) {
        let v = self.app(right);
        let fits = match (&v.ty, term.term.takes.as_slice()) {
            (Ty::Unknown, _) => true,
            (Ty::Entity(e), [p]) => *e == p.name,
            (ty, [p]) => *ty == Ty::from_name(&p.ty),
            _ => false,
        };
        if !fits {
            let message = format!("`{}` cannot take a {}", term.term.term, v.ty.name());
            self.error(codes::T002, app_span(right), message);
        }
    }

    fn misplaced_in(&mut self, left: &App, right: &App, span: Span) -> Val {
        let l = self.app(left);
        let r = self.app(right);
        if !l.is_unknown() && !r.is_unknown() {
            let message = "`in` follows only a term that ends in `in`".to_string();
            self.error(codes::T002, span, message);
        }
        Val::unknown()
    }

    /// Whether `phrase` resolves by itself, without reporting anything.
    fn resolves(&self, phrase: &Phrase) -> bool {
        self.lookup_let(&phrase.text()).is_some() || longest_term(self.lex, phrase).is_some()
    }

    /// The keyword `in` between byte `after` and byte `before`, as a word.
    fn in_word(&self, after: usize, before: usize) -> Word {
        let at = self
            .source
            .get(after..before)
            .and_then(|s| s.find("in"))
            .map_or(after, |i| after + i);
        Word {
            text: "in".to_string(),
            span: Span::new(at, at + 2),
        }
    }
}

/// The type a term gives; Unknown when it names none.
fn gives(term: &LexTerm) -> Ty {
    term.term
        .gives
        .as_deref()
        .map_or(Ty::Unknown, Ty::from_name)
}

/// An entity's instance: one word (`host gx10`) or one string (`repo "a/b"`).
/// A string instance's span excludes its quotes, so a fix keeps them.
fn instance_word(rest: &[Word], args: &[Atom]) -> Option<Word> {
    match (rest, args) {
        ([w], []) => Some(w.clone()),
        ([], [Atom::Text(t)]) => Some(Word {
            text: t.value.clone(),
            span: Span::new(t.span.start + 1, t.span.end.saturating_sub(1)),
        }),
        _ => None,
    }
}
