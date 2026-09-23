//! `ruchy explain` (spec RHL-001 §5, a new verb) and the MCP tool
//! `rhl_explain` (§6): a deterministic plain-language rendering of a program
//! from its tree. No model: the same tree gives the same bytes. The rules are
//! stated in `docs/rhl/rhl-6.md`.
//!
//! One line per `use` and per statement. A unit is a header line ending in
//! `:`, its statements indented two spaces per block depth; a block statement
//! (`when`, `for each`, `repeat`, `example`, an action `with` block) is a line
//! ending in `:` with its body one level deeper. Expressions are printed in
//! their RHL spelling ([`super::fmt`]), which is already plain words.

use super::cli::Outcome;
use super::diag::{self, render_text, Diagnostic};
use super::fmt::{app, atom, cond, quantity, text};
use super::tree::{Action, App, Decl, Program, Stmt, StmtKind, Unit, UnitKind};
use super::yaml::{self, is_rhl_yaml};
use std::path::Path;

/// The tree of `source`, named `file`: RHL text, or `.rhl.yaml` by name.
///
/// # Errors
///
/// Returns the `RHL-P…` diagnostic when `source` is not a tree.
pub fn tree_of(file: &str, source: &str) -> Result<Program, Diagnostic> {
    if is_rhl_yaml(Path::new(file)) {
        return yaml::from_yaml(source).map_err(|e| yaml::diagnostic(file, &e));
    }
    super::parse(source).map_err(|f| diag::from_parse_failure(file, source, &f))
}

/// The plain-language rendering of `program`.
#[must_use]
pub fn explain(program: &Program) -> String {
    let mut out = String::new();
    for decl in &program.decls {
        match decl {
            Decl::Use(u) => {
                line(
                    &mut out,
                    0,
                    &format!("Uses vocabulary {}.", u.vocabulary.text()),
                );
            }
            Decl::Unit(u) => {
                if !out.is_empty() {
                    out.push('\n');
                }
                explain_unit(&mut out, u);
            }
        }
    }
    out
}

/// `ruchy explain <file.rhl | file.rhl.yaml>`: the rendering on standard
/// output, exit 0; the parse diagnostic on standard error, exit 2, when the
/// file is not a tree; exit 1 when it cannot be read.
#[must_use]
pub fn explain_file(input: &Path) -> Outcome {
    let shown = input.display().to_string();
    let source = match std::fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => return outcome(String::new(), format!("{shown}: cannot read: {e}\n"), 1),
    };
    match tree_of(&shown, &source) {
        Ok(program) => outcome(explain(&program), String::new(), 0),
        Err(d) => outcome(String::new(), render_text(&[d]), 2),
    }
}

fn outcome(stdout: String, stderr: String, exit: i32) -> Outcome {
    Outcome {
        stdout,
        stderr,
        exit,
    }
}

fn line(out: &mut String, depth: usize, content: &str) {
    for _ in 0..depth {
        out.push_str("  ");
    }
    out.push_str(content);
    out.push('\n');
}

fn unit_word(kind: UnitKind) -> &'static str {
    match kind {
        UnitKind::Job => "Job",
        UnitKind::Command => "Command",
        UnitKind::Check => "Check",
        UnitKind::Pipeline => "Pipeline",
        UnitKind::Shape => "Shape",
    }
}

fn explain_unit(out: &mut String, unit: &Unit) {
    line(
        out,
        0,
        &format!("{} {}:", unit_word(unit.kind), text(&unit.name)),
    );
    body(out, 1, &unit.body);
}

fn body(out: &mut String, depth: usize, stmts: &[Stmt]) {
    for s in stmts {
        stmt(out, depth, &s.kind);
    }
}

fn stmt(out: &mut String, depth: usize, kind: &StmtKind) {
    match simple(kind) {
        Some(sentence) => line(out, depth, &format!("{sentence}.")),
        None => block(out, depth, kind),
    }
}

/// The sentence for a statement that has no body, without its full stop.
fn simple(kind: &StmtKind) -> Option<String> {
    header(kind)
        .or_else(|| binding(kind))
        .or_else(|| assertion(kind))
}

fn header(kind: &StmtKind) -> Option<String> {
    match kind {
        StmtKind::RunsOn(a) => Some(format!("Runs on {}", app(a))),
        StmtKind::Every(q) => Some(format!("Runs every {}", quantity(q))),
        StmtKind::May(e) => Some(format!("May {} {}", e.verb.word(), e.target.text())),
        StmtKind::WaitUpTo(q) => Some(format!("Waits up to {}", quantity(q))),
        _ => None,
    }
}

fn binding(kind: &StmtKind) -> Option<String> {
    match kind {
        StmtKind::Let { name, value } => Some(format!("Sets {} to {}", name.text(), cond(value))),
        StmtKind::Set { name, value } => {
            Some(format!("Changes {} to {}", name.text(), cond(value)))
        }
        _ => None,
    }
}

fn assertion(kind: &StmtKind) -> Option<String> {
    let (word, c) = match kind {
        StmtKind::Expect(c) => ("Expects", c),
        StmtKind::GiveBack(c) => ("Gives back", c),
        StmtKind::StopWith(c) => ("Stops with", c),
        StmtKind::Given(c) => ("Given", c),
        StmtKind::Then(c) => ("Then", c),
        StmtKind::Action(a) if a.with.is_none() => return Some(action_head(a)),
        _ => return None,
    };
    Some(format!("{word} {}", cond(c)))
}

fn block(out: &mut String, depth: usize, kind: &StmtKind) {
    match kind {
        StmtKind::When {
            cond: c,
            then_body,
            otherwise,
        } => {
            line(out, depth, &format!("When {}:", cond(c)));
            body(out, depth + 1, then_body);
            if let Some(o) = otherwise {
                line(out, depth, "Otherwise:");
                body(out, depth + 1, o);
            }
        }
        StmtKind::ForEach {
            name,
            collection,
            body: b,
        } => {
            line(
                out,
                depth,
                &format!("For each {} in {}:", name.text(), cond(collection)),
            );
            body(out, depth + 1, b);
        }
        other => closed_block(out, depth, other),
    }
}

fn closed_block(out: &mut String, depth: usize, kind: &StmtKind) {
    match kind {
        StmtKind::Repeat {
            times,
            until,
            body: b,
        } => {
            let until = until
                .as_ref()
                .map_or(String::new(), |u| format!(" until {}", cond(u)));
            line(
                out,
                depth,
                &format!("Repeats at most {} times{until}:", times.digits),
            );
            body(out, depth + 1, b);
        }
        StmtKind::Example { name, body: b } => {
            line(out, depth, &format!("Example {}:", text(name)));
            body(out, depth + 1, b);
        }
        StmtKind::Action(a) => with_block(out, depth, a),
        _ => {}
    }
}

fn action_head(a: &Action) -> String {
    match &a.target {
        Some(t) => format!("Does {} in {}", app(&a.head), app(t)),
        None => format!("Does {}", app(&a.head)),
    }
}

/// An action with a `with` block: its head, then one `name: value` line per
/// attribute. A statement in the block that is not an attribute is explained
/// as any other statement.
fn with_block(out: &mut String, depth: usize, a: &Action) {
    line(out, depth, &format!("{}, with:", action_head(a)));
    for s in a.with.as_deref().unwrap_or_default() {
        match attribute(&s.kind) {
            Some(attr) => line(out, depth + 1, &attr),
            None => stmt(out, depth + 1, &s.kind),
        }
    }
}

/// `title "gx10 disk watch"` → `title: "gx10 disk watch"`; an action with
/// no argument, a target or a block of its own is not an attribute.
fn attribute(kind: &StmtKind) -> Option<String> {
    let StmtKind::Action(Action {
        head: App::Call { phrase, args },
        target: None,
        with: None,
    }) = kind
    else {
        return None;
    };
    if args.is_empty() {
        return None;
    }
    let values: Vec<String> = args.iter().map(atom).collect();
    Some(format!("{}: {}", phrase.text(), values.join(" ")))
}

#[cfg(test)]
#[path = "explain_tests.rs"]
mod explain_tests;
