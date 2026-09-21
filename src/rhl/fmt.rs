//! `fmt` for RHL: the normal form (spec RHL-001 §3.1 principle 2; plan D7).
//!
//! The tree carries no layout, so printing it is the normal form: two
//! programs with the same tree format to the same bytes. The layout rules
//! (plan D7):
//!
//! - one statement per line, two-space indentation per block depth, single
//!   spaces between tokens, a trailing newline;
//! - one blank line between the `use` block and the first unit and between
//!   units; consecutive `use` lines have none;
//! - inside a unit body only, a blank line between two consecutive sibling
//!   statements when either is a block statement (`when`, `for each`,
//!   `repeat`, `example`, an action with a `with` block) or when both are
//!   simple statements of different groups (header, binding, assertion,
//!   other); never before the first statement or after the last;
//! - no blank lines inside nested bodies.
//!
//! [`format`] is total: it prints every tree, including trees the parser
//! cannot produce (those need not re-parse to themselves, since RHL has no
//! parentheses).

use super::tree::{
    Action, App, Atom, Cond, CondKind, Decl, Phrase, Program, Quantity, Stmt, StmtKind, Text, Unit,
};

/// Print `program` in the RHL normal form.
#[must_use]
pub fn format(program: &Program) -> String {
    let mut out = String::new();
    let mut prev: Option<&Decl> = None;
    for decl in &program.decls {
        if prev.is_some_and(|p| !(is_use(p) && is_use(decl))) {
            out.push('\n');
        }
        print_decl(&mut out, decl);
        prev = Some(decl);
    }
    out
}

/// Parse `source` and print it in the RHL normal form.
///
/// # Errors
///
/// Returns the [`crate::rhl::ParseFailure`] when `source` does not parse.
pub fn format_source(source: &str) -> Result<String, crate::rhl::ParseFailure> {
    crate::rhl::parse(source).map(|p| format(&p))
}

fn is_use(decl: &Decl) -> bool {
    matches!(decl, Decl::Use(_))
}

fn print_decl(out: &mut String, decl: &Decl) {
    match decl {
        Decl::Use(u) => line(out, 0, &format!("use vocabulary {}", u.vocabulary.text())),
        Decl::Unit(u) => print_unit(out, u),
    }
}

fn print_unit(out: &mut String, unit: &Unit) {
    let header = format!("{} {}", unit.kind.keyword(), text(&unit.name));
    line(out, 0, &header);
    print_unit_body(out, &unit.body);
    line(out, 0, "end");
}

/// One line at `depth` levels of two-space indentation.
fn line(out: &mut String, depth: usize, content: &str) {
    for _ in 0..depth {
        out.push_str("  ");
    }
    out.push_str(content);
    out.push('\n');
}

// ───────────────────────── bodies and the blank-line rule ─────────────────────────

/// The D7 groups a statement falls in for the blank-line rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Group {
    Header,
    Binding,
    Assertion,
    Other,
    Block,
}

fn is_block(kind: &StmtKind) -> bool {
    let block = matches!(
        kind,
        StmtKind::When { .. }
            | StmtKind::ForEach { .. }
            | StmtKind::Repeat { .. }
            | StmtKind::Example { .. }
    );
    block || matches!(kind, StmtKind::Action(a) if a.with.is_some())
}

fn group(kind: &StmtKind) -> Group {
    if is_block(kind) {
        Group::Block
    } else if matches!(
        kind,
        StmtKind::RunsOn(_) | StmtKind::Every(_) | StmtKind::May(_) | StmtKind::WaitUpTo(_)
    ) {
        Group::Header
    } else if matches!(kind, StmtKind::Let { .. } | StmtKind::Set { .. }) {
        Group::Binding
    } else if matches!(kind, StmtKind::Expect(_)) {
        Group::Assertion
    } else {
        Group::Other
    }
}

/// True when D7 puts a blank line between unit-body siblings `a` and `b`.
fn blank_between(a: &Stmt, b: &Stmt) -> bool {
    let (ga, gb) = (group(&a.kind), group(&b.kind));
    ga == Group::Block || gb == Group::Block || ga != gb
}

/// A unit body at depth 1, with the D7 blank lines between siblings.
fn print_unit_body(out: &mut String, body: &[Stmt]) {
    let mut prev: Option<&Stmt> = None;
    for s in body {
        if prev.is_some_and(|p| blank_between(p, s)) {
            out.push('\n');
        }
        print_stmt(out, 1, s);
        prev = Some(s);
    }
}

/// A nested body: no blank lines at all.
fn print_body(out: &mut String, depth: usize, body: &[Stmt]) {
    for s in body {
        print_stmt(out, depth, s);
    }
}

// ───────────────────────── statements ─────────────────────────

fn print_stmt(out: &mut String, depth: usize, s: &Stmt) {
    match simple_line(&s.kind) {
        Some(content) => line(out, depth, &content),
        None => print_block(out, depth, &s.kind),
    }
}

/// The one line of a statement that is not a block, or `None` for a block
/// or an action (actions are printed by [`print_block`]).
fn simple_line(kind: &StmtKind) -> Option<String> {
    header_line(kind)
        .or_else(|| binding_line(kind))
        .or_else(|| cond_stmt(kind).map(|(kw, c)| format!("{kw} {}", cond(c))))
}

fn header_line(kind: &StmtKind) -> Option<String> {
    match kind {
        StmtKind::RunsOn(a) => Some(format!("runs on {}", app(a))),
        StmtKind::Every(q) => Some(format!("every {}", quantity(q))),
        StmtKind::May(e) => Some(format!("may {} {}", e.verb.word(), e.target.text())),
        StmtKind::WaitUpTo(q) => Some(format!("wait up to {}", quantity(q))),
        _ => None,
    }
}

fn binding_line(kind: &StmtKind) -> Option<String> {
    match kind {
        StmtKind::Let { name, value } => Some(format!("let {} be {}", name.text(), cond(value))),
        StmtKind::Set { name, value } => Some(format!("set {} to {}", name.text(), cond(value))),
        _ => None,
    }
}

/// The keyword-plus-condition statements.
fn cond_stmt(kind: &StmtKind) -> Option<(&'static str, &Cond)> {
    match kind {
        StmtKind::Expect(c) => Some(("expect", c)),
        StmtKind::GiveBack(c) => Some(("give back", c)),
        StmtKind::StopWith(c) => Some(("stop with", c)),
        StmtKind::Given(c) => Some(("given", c)),
        StmtKind::Then(c) => Some(("then", c)),
        _ => None,
    }
}

/// Block statements and actions. Called only for kinds [`simple_line`]
/// does not print, so the last arm is never reached from [`print_stmt`].
fn print_block(out: &mut String, depth: usize, kind: &StmtKind) {
    match kind {
        StmtKind::When {
            cond: c,
            then_body,
            otherwise,
        } => print_when(out, depth, c, then_body, otherwise.as_deref()),
        StmtKind::ForEach {
            name,
            collection,
            body,
        } => print_for_each(out, depth, name, collection, body),
        StmtKind::Repeat { times, until, body } => {
            let until = until
                .as_ref()
                .map_or(String::new(), |u| format!(" until {}", cond(u)));
            let header = format!("repeat at most {} times{until}", times.digits);
            print_closed(out, depth, &header, body);
        }
        StmtKind::Example { name, body } => {
            print_closed(out, depth, &format!("example {}", text(name)), body);
        }
        StmtKind::Action(a) => print_action(out, depth, a),
        other => line(out, depth, &simple_line(other).unwrap_or_default()),
    }
}

/// `header`, the body one level deeper, `end`.
fn print_closed(out: &mut String, depth: usize, header: &str, body: &[Stmt]) {
    line(out, depth, header);
    print_body(out, depth + 1, body);
    line(out, depth, "end");
}

fn print_when(out: &mut String, depth: usize, c: &Cond, then: &[Stmt], other: Option<&[Stmt]>) {
    line(out, depth, &format!("when {}", cond(c)));
    print_body(out, depth + 1, then);
    if let Some(o) = other {
        line(out, depth, "otherwise");
        print_body(out, depth + 1, o);
    }
    line(out, depth, "end");
}

fn print_for_each(out: &mut String, depth: usize, name: &Phrase, coll: &Cond, body: &[Stmt]) {
    let header = format!("for each {} in {}", name.text(), cond(coll));
    print_closed(out, depth, &header, body);
}

fn print_action(out: &mut String, depth: usize, a: &Action) {
    let mut head = app(&a.head);
    if let Some(t) = &a.target {
        head = format!("{head} in {}", app(t));
    }
    match &a.with {
        Some(body) => print_closed(out, depth, &format!("{head} with"), body),
        None => line(out, depth, &head),
    }
}

// ───────────────────────── conditions and values ─────────────────────────

fn cond(c: &Cond) -> String {
    match &c.kind {
        CondKind::Or(a, b) => format!("{} or {}", cond(a), cond(b)),
        CondKind::And(a, b) => format!("{} and {}", cond(a), cond(b)),
        CondKind::Not(inner) => format!("not {}", cond(inner)),
        CondKind::Compare { left, op, right } => {
            format!("{} {} {}", app(left), op.keyword(), app(right))
        }
        CondKind::In { left, right } => format!("{} in {}", app(left), app(right)),
        CondKind::App(a) => app(a),
    }
}

fn app(a: &App) -> String {
    match a {
        App::Call { phrase, args } => {
            let mut s = phrase.text();
            for arg in args {
                s.push(' ');
                s.push_str(&atom(arg));
            }
            s
        }
        App::Quantity(q) => quantity(q),
        App::Text(t) => text(t),
    }
}

fn atom(a: &Atom) -> String {
    match a {
        Atom::Text(t) => text(t),
        Atom::Quantity(q) => quantity(q),
    }
}

/// `<digits>` or `<digits> <unit>`, for example `100 GB` or `5 %`.
fn quantity(q: &Quantity) -> String {
    match &q.unit {
        Some(u) => format!("{} {}", q.value.digits, u.text),
        None => q.value.digits.clone(),
    }
}

/// A string literal with its quotes.
fn text(t: &Text) -> String {
    format!("\"{}\"", t.value)
}

#[cfg(test)]
#[path = "fmt_tests.rs"]
mod fmt_tests;
