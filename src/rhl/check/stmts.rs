//! Statements: headers, bindings, actions and blocks (spec RHL-001 §3.3).

use super::checker::{app_span, longest_term, Checker};
use super::lexicon::{LexTerm, Lexicon};
use super::types::{duration_code, mismatch, Ty, Val};
use crate::rhl::codes;
use crate::rhl::diag::Expected;
use crate::rhl::tree::{Action, App, Cond, Effect, Phrase, Quantity, Span, Stmt, StmtKind, Word};
use crate::rhl::vocab::{Param, TermKind};

impl Checker<'_> {
    /// Check `stmts` in order. `in_example` is true only for the direct
    /// statements of an `example` body.
    pub(crate) fn body(&mut self, stmts: &[Stmt], in_example: bool) {
        for s in stmts {
            self.stmt(s, in_example);
        }
    }

    fn stmt(&mut self, s: &Stmt, in_example: bool) {
        match &s.kind {
            StmtKind::RunsOn(a) => self.runs_on(a),
            StmtKind::Every(q) | StmtKind::WaitUpTo(q) => self.duration(q),
            StmtKind::May(e) => self.may(e),
            StmtKind::Let { name, value } => self.let_stmt(name, value),
            StmtKind::Set { name, value } => self.set_stmt(name, value),
            _ => self.line_stmt(s, in_example),
        }
    }

    fn line_stmt(&mut self, s: &Stmt, in_example: bool) {
        match &s.kind {
            StmtKind::Expect(c) => self.condition(c),
            StmtKind::GiveBack(c) | StmtKind::StopWith(c) => {
                self.cond(c);
            }
            StmtKind::Given(c) | StmtKind::Then(c) => self.example_line(s, c, in_example),
            StmtKind::Action(a) => self.action(a),
            _ => self.block_stmt(s),
        }
    }

    fn block_stmt(&mut self, s: &Stmt) {
        match &s.kind {
            StmtKind::When {
                cond,
                then_body,
                otherwise,
            } => {
                self.condition(cond);
                self.body(then_body, false);
                self.body(otherwise.as_deref().unwrap_or_default(), false);
            }
            StmtKind::ForEach {
                name,
                collection,
                body,
            } => self.for_each(name, collection, body),
            StmtKind::Repeat { until, body, .. } => {
                if let Some(u) = until {
                    self.condition(u);
                }
                self.body(body, false);
            }
            StmtKind::Example { body, .. } => self.body(body, true),
            _ => {}
        }
    }

    /// `runs on <app>`: the app must be an entity instance.
    fn runs_on(&mut self, app: &App) {
        let v = self.app(app);
        if !v.is_unknown() && !matches!(v.ty, Ty::Entity(_)) {
            let message = format!(
                "`runs on` needs an entity such as `host gx10`, not a {}",
                v.ty.name()
            );
            self.error(codes::T002, app_span(app), message);
        }
    }

    /// `every` and `wait up to` need a Duration.
    fn duration(&mut self, q: &Quantity) {
        let v = self.quantity(q);
        if let Some(code) = duration_code(&v) {
            let message = format!(
                "a Duration such as `1 hour` is needed here, not a {}",
                describe(&v)
            );
            self.error(code, q.span, message);
        }
    }

    /// `may <verb> <target>`: the target comes from the effect targets.
    fn may(&mut self, e: &Effect) {
        let target = e.target.text();
        let pool = self.lex.effect_target_pool();
        if pool.iter().any(|(t, _)| *t == target) {
            self.declare(e.verb.word(), target);
        } else {
            self.unknown(&e.target.words, &pool, "effect target");
        }
    }

    fn let_stmt(&mut self, name: &Phrase, value: &Cond) {
        let v = self.cond(value);
        self.bind(name.text(), v);
    }

    fn set_stmt(&mut self, name: &Phrase, value: &Cond) {
        let v = self.cond(value);
        let Some(old) = self.lookup_let(&name.text()) else {
            let pool = self.generic_pool();
            self.unknown(&name.words, &pool, "let name");
            return;
        };
        if !old.is_unknown() && !v.is_unknown() && old.ty != v.ty {
            let message = format!(
                "`{}` is a {}, not a {}",
                name.text(),
                old.ty.name(),
                v.ty.name()
            );
            self.error(mismatch(&old, &v), value.span, message);
        }
    }

    /// `given`/`then`: RHL-X001 unless directly inside an `example` body.
    fn example_line(&mut self, s: &Stmt, c: &Cond, in_example: bool) {
        if !in_example {
            let message =
                "`given` and `then` belong directly inside an `example` block".to_string();
            self.error(codes::X001, s.span, message);
        }
        self.condition(c);
    }

    /// `for each`: RHL-B001, because no v1 term gives a finite collection.
    fn for_each(&mut self, name: &Phrase, collection: &Cond, body: &[Stmt]) {
        let message =
            "`for each` needs a finite collection, and no term of the used vocabularies gives one"
                .to_string();
        self.error(codes::B001, collection.span, message);
        self.cond(collection);
        self.bind(name.text(), Val::unknown());
        self.body(body, false);
    }

    fn action(&mut self, a: &Action) {
        self.action_head(a);
        let Some(with) = &a.with else { return };
        match attributed_action(self.lex, a) {
            Some(term) => self.attribute_block(term, with),
            None => self.body(with, false),
        }
    }

    /// The `with` block of an action that declares `attributes:` (v2,
    /// RHL-16): each action-shaped line is an attribute assignment; any other
    /// statement is checked as usual.
    fn attribute_block(&mut self, term: &LexTerm, with: &[Stmt]) {
        for s in with {
            match &s.kind {
                StmtKind::Action(line) => self.attribute_line(term, line, s.span),
                _ => self.stmt(s, false),
            }
        }
    }

    /// `<attribute> <value>`: the attribute must be declared by `term` and the
    /// value must have its type.
    fn attribute_line(&mut self, term: &LexTerm, line: &Action, span: Span) {
        let action = &term.term.term;
        let App::Call { phrase, args } = &line.head else {
            let message = format!("a line in the `with` block of `{action}` names an attribute");
            return self.error(codes::T002, app_span(&line.head), message);
        };
        if line.target.is_some() || line.with.is_some() {
            let message = format!("the attribute `{}` takes only a value", phrase.text());
            return self.error(codes::T002, span, message);
        }
        let attrs = &term.term.attributes;
        match attrs.iter().find(|p| p.name == phrase.text()) {
            Some(p) => self.check_args(&p.name, std::slice::from_ref(p), args, phrase.span),
            None => self.unknown_attribute(term, &phrase.words),
        }
    }

    /// An attribute `term` does not declare: V002/V003 for a near miss, as for
    /// any unknown word, and V001 when nothing is near. V001 means "no
    /// candidate" (the catalogue), so the declared attributes travel in the
    /// diagnostic's `expected` set — `{kind: attribute, of, names}` — never as
    /// candidates (RHL-2's ruling on RHL-16's open question).
    fn unknown_attribute(&mut self, term: &LexTerm, words: &[Word]) {
        let action = &term.term.term;
        let pool: Vec<(String, String)> = term
            .term
            .attributes
            .iter()
            .map(|p| (p.name.clone(), term.vocab.clone()))
            .collect();
        let at = self.diags.len();
        self.unknown(words, &pool, &format!("attribute of `{action}`"));
        let Some(d) = self
            .diags
            .get_mut(at)
            .filter(|d| d.code.starts_with("RHL-V"))
        else {
            return;
        };
        let names: Vec<String> = pool.into_iter().map(|(n, _)| n).collect();
        if d.code == codes::V001 {
            let listed: Vec<String> = names.iter().map(|n| format!("`{n}`")).collect();
            d.message = format!("{}; `{action}` has {}", d.message, listed.join(", "));
        }
        d.expected = Some(Expected {
            kind: Some("attribute".to_string()),
            of: Some(action.clone()),
            names,
            ..Expected::default()
        });
    }

    /// The head of an action statement must be an action term.
    fn action_head(&mut self, a: &Action) {
        let App::Call { phrase, args } = &a.head else {
            let message = "an action statement starts with an action term".to_string();
            return self.error(codes::T002, app_span(&a.head), message);
        };
        let lex = self.lex;
        match longest_term(lex, phrase) {
            Some((k, t)) if k == phrase.words.len() && t.term.kind == TermKind::Action => {
                self.use_term(t, phrase.span);
                let params = self.action_target(&t.term.term, &t.term.takes, a.target.as_ref());
                if let Some(params) = params {
                    self.check_args(&t.term.term, &params, args, phrase.span);
                }
            }
            None if self.lookup_let(&phrase.text()).is_none() => {
                let pool = self.generic_pool();
                self.unknown(&phrase.words, &pool, "term or let name");
            }
            _ => {
                let message = format!("`{}` is not an action", phrase.text());
                self.error(codes::T002, phrase.span, message);
            }
        }
    }

    /// The parameters left for the head's arguments once the target (after
    /// `in`) has supplied the one named after its entity; `None` after an error.
    fn action_target(
        &mut self,
        name: &str,
        takes: &[Param],
        target: Option<&App>,
    ) -> Option<Vec<Param>> {
        let Some(target) = target else {
            return Some(takes.to_vec());
        };
        let v = self.app(target);
        let slot = match &v.ty {
            Ty::Unknown => return None,
            Ty::Entity(e) => takes.iter().position(|p| p.name == *e),
            _ => None,
        };
        let Some(slot) = slot else {
            let message = format!("`{name}` cannot take a {} after `in`", v.ty.name());
            self.error(codes::T002, app_span(target), message);
            return None;
        };
        let mut rest = takes.to_vec();
        rest.remove(slot);
        Some(rest)
    }
}

/// The action term that heads `a`, when it declares `attributes:` (v2).
fn attributed_action<'l>(lex: &'l Lexicon, a: &Action) -> Option<&'l LexTerm> {
    let App::Call { phrase, .. } = &a.head else {
        return None;
    };
    let (k, t) = longest_term(lex, phrase)?;
    let whole = k == phrase.words.len() && t.term.kind == TermKind::Action;
    (whole && !t.term.attributes.is_empty()).then_some(t)
}

/// How a diagnostic names a value: `a bare number` or its type.
fn describe(v: &Val) -> String {
    if v.bare {
        "bare number".to_string()
    } else {
        v.ty.name()
    }
}
