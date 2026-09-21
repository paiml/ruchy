//! Statements: headers, bindings, actions and blocks (spec RHL-001 §3.3).

use super::checker::{app_span, longest_term, Checker};
use super::types::{duration_code, mismatch, Ty, Val};
use crate::rhl::codes;
use crate::rhl::tree::{Action, App, Cond, Effect, Phrase, Quantity, Stmt, StmtKind};
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
        if let Some(with) = &a.with {
            self.body(with, false);
        }
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

/// How a diagnostic names a value: `a bare number` or its type.
fn describe(v: &Val) -> String {
    if v.bare {
        "bare number".to_string()
    } else {
        v.ty.name()
    }
}
