//! The RHL checker: vocabulary, types and units, effects, bounds (spec RHL-001
//! §3.3-§3.5; plan D2-D6).
//!
//! [`check`] parses a program, loads the vocabularies it uses, and walks every
//! unit. The checks are the spec's, applied strictly (plan D2): an unknown word
//! is an error with candidates, never a guess (§9 rule 2).
//!
//! The namespace of a word is picked by its syntactic position only (plan D3,
//! reconciled with D4): after `may <verb>` it is the effect targets; after a
//! number it is the unit terms (plus `%`); everywhere else it is every term of
//! the used vocabularies plus the `let` names in scope.

mod checker;
mod lexicon;
mod near;
mod stmts;
mod types;
mod values;

use super::codes;
use super::diag::{self, Diagnostic};
use super::tree::{Decl, Program, UseDecl};
use super::vocab;
use checker::Checker;
use lexicon::Lexicon;
use serde::Serialize;
use std::path::Path;

/// The outcome of a check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// No error.
    Pass,
    /// At least one error, none of them a refusal.
    Fail,
    /// At least one diagnostic is a §3.5 refusal.
    Refused,
}

impl Verdict {
    /// The verdict as the JSON report spells it: `pass`, `fail` or `refused`.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Refused => "refused",
        }
    }
}

/// An entity instance that could not be verified, because the entity's
/// `instances_from` glob matches no file (plan D6). Not a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Unverified {
    /// The entity term, for example `host`.
    pub entity: String,
    /// The instance as written, for example `gx10`.
    pub instance: String,
    /// The glob that matched nothing, for example `machines/*/forjar.yaml`.
    pub source_glob: String,
}

/// The result of checking one file, in plan D8's JSON shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    /// The file checked.
    pub file: String,
    /// Pass, fail or refused.
    pub verdict: Verdict,
    /// Every diagnostic, in source order.
    pub diagnostics: Vec<Diagnostic>,
    /// Entity instances with no source of truth.
    pub unverified: Vec<Unverified>,
}

impl Report {
    fn new(file: &str, mut diagnostics: Vec<Diagnostic>, unverified: Vec<Unverified>) -> Self {
        diagnostics.sort_by_key(|d| (d.span.line, d.span.col));
        let verdict = if diagnostics.iter().any(|d| d.refusal.is_some()) {
            Verdict::Refused
        } else if diagnostics.is_empty() {
            Verdict::Pass
        } else {
            Verdict::Fail
        };
        Self {
            file: file.to_string(),
            verdict,
            diagnostics,
            unverified,
        }
    }

    /// A report of `diagnostics` alone, for tests of code that consumes reports.
    #[cfg(test)]
    pub(crate) fn for_test(file: &str, diagnostics: Vec<Diagnostic>) -> Self {
        Self::new(file, diagnostics, Vec::new())
    }

    /// The process exit code: 2 for a refusal, 1 for any other error, else 0.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self.verdict {
            Verdict::Refused => 2,
            Verdict::Fail => 1,
            Verdict::Pass => 0,
        }
    }
}

/// Check the program `source`, named `file` in diagnostics. Vocabularies are
/// read from `<root>/vocab/`; with no `root`, every `use vocabulary` is
/// RHL-V004. A parse failure is one P-code diagnostic and nothing else.
#[must_use]
pub fn check(file: &str, source: &str, root: Option<&Path>) -> Report {
    let mut report = check_once(file, source, root);
    for d in &mut report.diagnostics {
        demote_unsafe_fixes(d, file, source, root);
    }
    report
}

/// §3.5: "A fix is safe only when it is the unique candidate and preserves
/// types." The checker proposes a fix as safe on uniqueness alone; this
/// applies it, re-checks the result once, and demotes the fix when the fixed
/// program has a type or unit error (`RHL-T…`) on an edited line — `runs on
/// hou gx10` fixed to the unit `hour`, or `10 hou` fixed to a Duration where
/// a Size is compared.
fn demote_unsafe_fixes(d: &mut Diagnostic, file: &str, source: &str, root: Option<&Path>) {
    for fix in d.fixes.iter_mut().filter(|f| f.safe) {
        let fixed = diag::apply_edits(source, &fix.edits);
        let lines: Vec<usize> = fix.edits.iter().map(|e| e.span.line).collect();
        let after = check_once(file, &fixed, root);
        if let Some(t) = after
            .diagnostics
            .iter()
            .find(|x| x.code.starts_with("RHL-T") && lines.contains(&x.span.line))
        {
            fix.safe = false;
            fix.title = format!("{} (not safe: the result has {})", fix.title, t.code);
        }
    }
}

/// One check, with every fix as the checker proposed it.
fn check_once(file: &str, source: &str, root: Option<&Path>) -> Report {
    let program = match super::parse(source) {
        Ok(p) => p,
        Err(f) => {
            let d = diag::from_parse_failure(file, source, &f);
            return Report::new(file, vec![d], Vec::new());
        }
    };
    let mut loader = Loader {
        file,
        source,
        root,
        lex: Lexicon::default(),
        diags: Vec::new(),
    };
    loader.load_all(&program);
    let mut checker = Checker::new(file, source, &loader.lex);
    for decl in &program.decls {
        if let Decl::Unit(unit) = decl {
            checker.unit(unit);
        }
    }
    let mut diagnostics = loader.diags;
    diagnostics.extend(checker.diags);
    Report::new(file, diagnostics, checker.unverified)
}

/// Loads the vocabularies of a program's `use vocabulary` lines.
struct Loader<'a> {
    file: &'a str,
    source: &'a str,
    root: Option<&'a Path>,
    lex: Lexicon,
    diags: Vec<Diagnostic>,
}

impl Loader<'_> {
    fn load_all(&mut self, program: &Program) {
        for decl in &program.decls {
            if let Decl::Use(u) = decl {
                self.load_use(u);
            }
        }
    }

    fn load_use(&mut self, u: &UseDecl) {
        let words: Vec<&str> = u.vocabulary.words.iter().map(|w| w.text.as_str()).collect();
        let Some((name, version)) = vocab::parse_reference(&words) else {
            let message =
                "`use vocabulary` needs a name and a version, as in `fleet v1`".to_string();
            return self.error(codes::V004, u, message);
        };
        let Some(root) = self.root else {
            let message = format!("no `vocab/` directory holds vocabulary `{name} v{version}`");
            return self.error(codes::V004, u, message);
        };
        if self.lex.has(&format!("{name} v{version}")) {
            return;
        }
        match vocab::load(root, &name, version) {
            Ok(v) => self.accept(root, &v, u),
            Err(e) => self.error(
                codes::V004,
                u,
                format!("unknown vocabulary `{name} v{version}`: {e}"),
            ),
        }
    }

    /// Add the vocabulary's terms, refusing it by name (RHL-C001) for every
    /// term with no contract template. The terms are still added, so the rest
    /// of the check is not a cascade of unknown words.
    fn accept(&mut self, root: &Path, v: &vocab::Vocabulary, u: &UseDecl) {
        for t in vocab::missing_contracts(root, v) {
            let message = format!(
                "vocabulary `{}` is refused: term `{}` has no contract template `{}`",
                v.label(),
                t.term,
                t.contract
            );
            self.error(codes::C001, u, message);
        }
        self.lex.add(root, v);
    }

    fn error(&mut self, code: &str, u: &UseDecl, message: String) {
        let d = Diagnostic::error(code, self.file, self.source, u.span, message);
        self.diags.push(d);
    }
}

#[cfg(test)]
#[path = "check_tests.rs"]
mod check_tests;
