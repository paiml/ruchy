//! RHL-5b: the unit contract of a job (spec RHL-001 §3.1 principle 7, §4,
//! §7 F6, §9.3). `ruchy transpile x.rhl --emit contract` prints it and `ruchy
//! compile` writes it next to the binary.
//!
//! - one equation per `expect`, its formula over the plan-measure table as a
//!   `postconditions` (ensures) entry;
//! - the effect surface (`may …` lines) as the invariants of `effect_surface`;
//! - `runs on`, `every`, the vocabularies (version, sha256), the source sha256
//!   and every term used with its term contract, as metadata;
//! - falsification tests naming the generated example tests and guards.
//!
//! A job with no `expect` still has a contract: its effect surface and terms.
//! The bytes depend only on the tree, the vocabularies and the source bytes.
//! The layout is written down in `docs/rhl/rhl-5.md`.

use super::check::{load_lexicon, Lexicon};
use super::lower::{
    lower_source_as, lower_yaml_as, plan_measure_fn, Form, LowerFailure, PLAN_MEASURES,
};
use super::receipt::sha256_hex;
use super::tree::{App, Atom, CompOp, Cond, CondKind, Decl, Phrase, Program, Stmt, StmtKind, Unit};
use super::vocab::{parse_reference, vocab_path};
use std::path::Path;

/// The unit contract of the RHL text `source` (named `file`).
///
/// # Errors
///
/// Whatever `ruchy test`'s lowering reports: the check's failure, or a
/// refusal (`RHL-L001`, `RHL-X002`).
pub fn unit_contract(
    file: &str,
    source: &str,
    root: Option<&Path>,
) -> Result<String, LowerFailure> {
    lower_source_as(Form::Tests, file, source, root)?;
    let normal = super::parse(source).map(|p| super::fmt::format(&p));
    emit(file, source, normal.ok(), root)
}

/// The unit contract of the `.rhl.yaml` text `source`: the same bytes as for
/// its RHL normal form, but for `source_sha256`.
///
/// # Errors
///
/// As [`unit_contract`].
pub fn unit_contract_yaml(
    file: &str,
    source: &str,
    root: Option<&Path>,
) -> Result<String, LowerFailure> {
    lower_yaml_as(Form::Tests, file, source, root)?;
    let normal = super::yaml::from_yaml(source).map(|p| super::fmt::format(&p));
    emit(file, source, normal.ok(), root)
}

/// Render the contract from the normal form of the checked tree.
fn emit(
    file: &str,
    source: &str,
    normal: Option<String>,
    root: Option<&Path>,
) -> Result<String, LowerFailure> {
    let unchecked = || LowerFailure::Check(super::check::check(file, source, root));
    let normal = normal.ok_or_else(unchecked)?;
    let program = super::parse(&normal).map_err(|_| unchecked())?;
    let unit = job(&program).ok_or_else(unchecked)?;
    let lex = load_lexicon(file, &normal, &program, root);
    let job = Job::read(&program, unit, &lex, root.unwrap_or(Path::new(".")));
    Ok(job.render(&sha256_hex(source.as_bytes())))
}

fn job(program: &Program) -> Option<&Unit> {
    program.decls.iter().find_map(|d| match d {
        Decl::Unit(u) => Some(u),
        Decl::Use(_) => None,
    })
}

/// What the contract states about one job, in source order.
struct Job {
    name: String,
    runs_on: Option<String>,
    every: Option<String>,
    effects: Vec<String>,
    /// `(RHL text, formula)` of each top-level `expect`.
    expects: Vec<(String, String)>,
    examples: Vec<String>,
    /// `(name, version, sha256)` of each vocabulary used.
    vocabularies: Vec<(String, u32, String)>,
    /// `(term, vocabulary, contract path)` of each term used, first use first.
    terms: Vec<(String, String, String)>,
}

impl Job {
    fn read(program: &Program, unit: &Unit, lex: &Lexicon, root: &Path) -> Self {
        let mut job = Self {
            name: unit.name.value.clone(),
            runs_on: None,
            every: None,
            effects: Vec::new(),
            expects: Vec::new(),
            examples: Vec::new(),
            vocabularies: vocabularies(program, root),
            terms: Vec::new(),
        };
        for s in &unit.body {
            job.header(s);
        }
        let mut terms = TermWalk {
            lex,
            found: Vec::new(),
        };
        terms.stmts(&unit.body);
        job.terms = terms.found;
        job
    }

    fn header(&mut self, s: &Stmt) {
        match &s.kind {
            StmtKind::RunsOn(app) => self.runs_on = Some(app_text(app)),
            StmtKind::Every(q) => self.every = Some(atom_text(&Atom::Quantity(q.clone()))),
            StmtKind::May(e) => self
                .effects
                .push(format!("{} {}", e.verb.word(), e.target.text())),
            StmtKind::Expect(c) => self
                .expects
                .push((format!("expect {}", cond_text(c)), formula(c))),
            StmtKind::Example { name, .. } => self.examples.push(name.value.clone()),
            _ => {}
        }
    }
}

fn vocabularies(program: &Program, root: &Path) -> Vec<(String, u32, String)> {
    program
        .decls
        .iter()
        .filter_map(|d| match d {
            Decl::Use(u) => {
                let words: Vec<&str> = u.vocabulary.words.iter().map(|w| w.text.as_str()).collect();
                parse_reference(&words)
            }
            Decl::Unit(_) => None,
        })
        .map(|(name, version)| {
            let bytes = std::fs::read(vocab_path(root, &name, version)).unwrap_or_default();
            (name, version, sha256_hex(&bytes))
        })
        .collect()
}

/// Every vocabulary term a job's statements use, resolved as the checker
/// resolves a phrase: its longest leading run of words that is a term.
struct TermWalk<'l> {
    lex: &'l Lexicon,
    found: Vec<(String, String, String)>,
}

impl TermWalk<'_> {
    fn stmts(&mut self, body: &[Stmt]) {
        for s in body {
            self.stmt(&s.kind);
        }
    }

    fn stmt(&mut self, kind: &StmtKind) {
        match kind {
            StmtKind::RunsOn(a) => self.app(a),
            StmtKind::Every(q) | StmtKind::WaitUpTo(q) => {
                self.unit(q.unit.as_ref().map(|u| u.text.as_str()))
            }
            StmtKind::Let { value: c, .. }
            | StmtKind::Set { value: c, .. }
            | StmtKind::Expect(c)
            | StmtKind::GiveBack(c)
            | StmtKind::StopWith(c)
            | StmtKind::Given(c)
            | StmtKind::Then(c) => self.cond(c),
            StmtKind::Action(a) => {
                self.app(&a.head);
                a.target.iter().for_each(|t| self.app(t));
                a.with.iter().for_each(|w| self.stmts(w));
            }
            _ => self.blocks(kind),
        }
    }

    fn blocks(&mut self, kind: &StmtKind) {
        match kind {
            StmtKind::When {
                cond,
                then_body,
                otherwise,
            } => {
                self.cond(cond);
                self.stmts(then_body);
                otherwise.iter().for_each(|o| self.stmts(o));
            }
            StmtKind::ForEach {
                collection, body, ..
            } => {
                self.cond(collection);
                self.stmts(body);
            }
            StmtKind::Repeat { until, body, .. } => {
                until.iter().for_each(|c| self.cond(c));
                self.stmts(body);
            }
            StmtKind::Example { body, .. } => self.stmts(body),
            _ => {}
        }
    }

    fn cond(&mut self, c: &Cond) {
        match &c.kind {
            CondKind::Or(a, b) | CondKind::And(a, b) => {
                self.cond(a);
                self.cond(b);
            }
            CondKind::Not(a) => self.cond(a),
            CondKind::Compare { left, right, .. } | CondKind::In { left, right } => {
                self.app(left);
                self.app(right);
            }
            CondKind::App(a) => self.app(a),
        }
    }

    fn app(&mut self, app: &App) {
        match app {
            App::Call { phrase, args } => {
                self.phrase(phrase);
                for a in args {
                    if let Atom::Quantity(q) = a {
                        self.unit(q.unit.as_ref().map(|u| u.text.as_str()));
                    }
                }
            }
            App::Quantity(q) => self.unit(q.unit.as_ref().map(|u| u.text.as_str())),
            App::Text(_) => {}
        }
    }

    fn phrase(&mut self, phrase: &Phrase) {
        let words: Vec<&str> = phrase.words.iter().map(|w| w.text.as_str()).collect();
        let hit = (1..=words.len())
            .rev()
            .find_map(|k| self.lex.get(&words[..k].join(" ")));
        if let Some(t) = hit {
            self.add(&t.term.term, &t.vocab, &t.term.contract);
        }
    }

    fn unit(&mut self, unit: Option<&str>) {
        if let Some(t) = unit.and_then(|u| self.lex.unit(u)) {
            self.add(&t.term.term, &t.vocab, &t.term.contract);
        }
    }

    fn add(&mut self, term: &str, vocab: &str, contract: &str) {
        if !self.found.iter().any(|(t, v, _)| t == term && v == vocab) {
            self.found
                .push((term.to_string(), vocab.to_string(), contract.to_string()));
        }
    }
}

// ---------------------------------------------------------------- formulas

/// A plan condition as a formula over the plan-measure table:
/// `ticket count is at most 1` → `plan_ticket_count(plan) ≤ 1`.
fn formula(c: &Cond) -> String {
    match &c.kind {
        CondKind::Or(a, b) => format!("({} ∨ {})", formula(a), formula(b)),
        CondKind::And(a, b) => format!("({} ∧ {})", formula(a), formula(b)),
        CondKind::Not(a) => format!("¬({})", formula(a)),
        CondKind::Compare { left, op, right } => {
            format!("{} {} {}", operand(left), symbol(*op), operand(right))
        }
        CondKind::In { left, right } => format!("{} ∈ {}", operand(left), operand(right)),
        CondKind::App(a) => operand(a),
    }
}

fn operand(app: &App) -> String {
    match app {
        App::Call { phrase, args } if args.is_empty() => {
            let text = phrase.text();
            if PLAN_MEASURES.iter().any(|(t, _)| *t == text) {
                format!("{}(plan)", plan_measure_fn(&text))
            } else {
                text
            }
        }
        other => app_text(other),
    }
}

fn symbol(op: CompOp) -> &'static str {
    match op {
        CompOp::Is => "=",
        CompOp::IsNot => "≠",
        CompOp::IsBelow => "<",
        CompOp::IsAbove => ">",
        CompOp::IsAtLeast => "≥",
        CompOp::IsAtMost => "≤",
        CompOp::IsOneOf => "∈",
        CompOp::Contains => "∋",
    }
}

fn op_words(op: CompOp) -> &'static str {
    match op {
        CompOp::Is => "is",
        CompOp::IsNot => "is not",
        CompOp::IsBelow => "is below",
        CompOp::IsAbove => "is above",
        CompOp::IsAtLeast => "is at least",
        CompOp::IsAtMost => "is at most",
        CompOp::IsOneOf => "is one of",
        CompOp::Contains => "contains",
    }
}

/// The RHL spelling of a condition (normal form).
fn cond_text(c: &Cond) -> String {
    match &c.kind {
        CondKind::Or(a, b) => format!("{} or {}", cond_text(a), cond_text(b)),
        CondKind::And(a, b) => format!("{} and {}", cond_text(a), cond_text(b)),
        CondKind::Not(a) => format!("not {}", cond_text(a)),
        CondKind::Compare { left, op, right } => {
            format!("{} {} {}", app_text(left), op_words(*op), app_text(right))
        }
        CondKind::In { left, right } => format!("{} in {}", app_text(left), app_text(right)),
        CondKind::App(a) => app_text(a),
    }
}

fn app_text(app: &App) -> String {
    match app {
        App::Call { phrase, args } => std::iter::once(phrase.text())
            .chain(args.iter().map(atom_text))
            .collect::<Vec<_>>()
            .join(" "),
        App::Quantity(q) => atom_text(&Atom::Quantity(q.clone())),
        App::Text(t) => format!("\"{}\"", t.value),
    }
}

fn atom_text(atom: &Atom) -> String {
    match atom {
        Atom::Text(t) => format!("\"{}\"", t.value),
        Atom::Quantity(q) => match &q.unit {
            Some(u) => format!("{} {}", q.value.digits, u.text),
            None => q.value.digits.clone(),
        },
    }
}

#[path = "contract_render.rs"]
mod render;

#[cfg(test)]
#[path = "contract_tests.rs"]
mod contract_tests;
