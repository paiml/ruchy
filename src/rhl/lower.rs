//! RHL-4: lowering a checked `job` to ruchy source (spec RHL-001 §4; plan
//! RHLGA-1 ruling Q4; binding B1).
//!
//! RHL emits ruchy *source text*, which ruchy's own public [`crate::Parser`]
//! and [`crate::Transpiler`] turn into Rust. Lowering never builds a ruchy
//! `Expr` and never touches the transpiler. The emitted shape is the Q4
//! ruling: a `struct Facts` with one field per distinct (measure, literal
//! arguments) pair the job reads, an `enum Action` with one struct variant per
//! action term it uses, and `fun decide(facts: Facts) -> Vec<Action>`, built
//! from `let`, assignment, `if`/`else`, a counted `while`, and `plan.push`.
//! No closures, no maps, no `unsafe`.
//!
//! Lowering runs only on a program that checks clean ([`lower_source`]); a
//! construct the v0 lowering does not cover is refused by name with
//! `RHL-L001`, never guessed at. The rules, the unit table and the refusals
//! are written down in `docs/rhl/rhl-4.md`.
//!
//! Determinism (RHL-001 F5): the output depends only on the tree and the
//! vocabularies, in source order. Nothing is hashed, timed or read from the
//! host, and the file name is not in the output.

use super::check::{check, check_yaml, load_lexicon, Lexicon, Report};
use super::codes;
use super::diag::{Diagnostic, LineSpan};
use super::runtime::{ActionUse, MeasureUse, Uses};
use super::tree::{
    Action, App, Atom, CompOp, Cond, CondKind, Decl, Int, Phrase, Program, Quantity, Span, Stmt,
    StmtKind, Unit, UnitKind,
};
use super::vocab::{Param, Term, TermKind};
use std::path::Path;

/// Why a program was not lowered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LowerFailure {
    /// The program does not check clean; its report says why.
    Check(Report),
    /// The program checks clean, but has a construct the v0 lowering does not
    /// cover (`RHL-L001`).
    Refused(Diagnostic),
}

impl LowerFailure {
    /// The process exit code: the check's own, or 2 for a lowering refusal.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Check(report) => report.exit_code(),
            Self::Refused(_) => 2,
        }
    }
}

/// What lowering emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    /// `Facts`, `Action` and `decide`: what `ruchy transpile` emits.
    Decide,
    /// `decide` plus the runtime functions the job's terms are bound to,
    /// `observe`, `apply` and a dry-run-by-default `main`: what `ruchy
    /// compile` and `ruchy run` build (spec Amendment A4). A term bound to
    /// `[U]` is refused with `RHL-L002`. Its `main` refuses to apply a plan
    /// that breaks an `expect` (RHL-5, §9.4).
    Program,
    /// `decide`, the job's `expect` guards, one test per `example` and a test
    /// runner `main`: what `ruchy test` builds (RHL-5). No `observe`, no
    /// `apply` and no runtime binding: an example states its facts, so its
    /// test never reads or changes the host.
    Tests,
}

/// Check the RHL text `source` (named `file`), then lower it.
///
/// # Errors
///
/// [`LowerFailure::Check`] when the check does not pass;
/// [`LowerFailure::Refused`] for a construct the v0 lowering does not cover.
pub fn lower_source(file: &str, source: &str, root: Option<&Path>) -> Result<String, LowerFailure> {
    lower_source_as(Form::Decide, file, source, root)
}

/// [`lower_source`] to the given [`Form`].
///
/// # Errors
///
/// As [`lower_source`]; for [`Form::Program`] also `RHL-L002`.
pub fn lower_source_as(
    form: Form,
    file: &str,
    source: &str,
    root: Option<&Path>,
) -> Result<String, LowerFailure> {
    lower_checked(form, file, source, root, check(file, source, root))
}

/// Check the `.rhl.yaml` text `source`, then lower its tree. The tree is
/// lowered from its RHL normal form, so both surfaces lower identically.
///
/// # Errors
///
/// As [`lower_source`]; a refusal carries no position ([`LineSpan::UNKNOWN`]),
/// as every diagnostic of a `.rhl.yaml` file does.
pub fn lower_yaml(file: &str, source: &str, root: Option<&Path>) -> Result<String, LowerFailure> {
    lower_yaml_as(Form::Decide, file, source, root)
}

/// [`lower_yaml`] to the given [`Form`].
///
/// # Errors
///
/// As [`lower_yaml`]; for [`Form::Program`] also `RHL-L002`.
pub fn lower_yaml_as(
    form: Form,
    file: &str,
    source: &str,
    root: Option<&Path>,
) -> Result<String, LowerFailure> {
    let report = check_yaml(file, source, root);
    let program = match super::yaml::from_yaml(source) {
        Ok(p) => p,
        Err(_) => return Err(LowerFailure::Check(report)),
    };
    let normal = super::fmt::format(&program);
    lower_checked(form, file, &normal, root, report).map_err(|f| match f {
        LowerFailure::Refused(mut d) => {
            d.span = LineSpan::UNKNOWN;
            LowerFailure::Refused(d)
        }
        other => other,
    })
}

fn lower_checked(
    form: Form,
    file: &str,
    source: &str,
    root: Option<&Path>,
    report: Report,
) -> Result<String, LowerFailure> {
    if report.exit_code() != 0 {
        return Err(LowerFailure::Check(report));
    }
    let Ok(program) = super::parse(source) else {
        return Err(LowerFailure::Check(report));
    };
    let lex = load_lexicon(file, source, &program, root);
    let (decide, uses, checks) =
        lower_with(form, file, source, &program, &lex).map_err(LowerFailure::Refused)?;
    if form != Form::Program {
        return Ok(decide + &checks);
    }
    let rest = super::runtime::executable(root.unwrap_or(Path::new(".")), &uses).map_err(
        |(span, m)| LowerFailure::Refused(Diagnostic::error(codes::L002, file, source, span, m)),
    )?;
    Ok(decide + &checks + &rest)
}

/// ruchy source to Rust, through ruchy's own parser and transpiler: the calls
/// `ruchy transpile x.ruchy` makes (`transpile_to_program`, then
/// `prettyplease`).
///
/// # Errors
///
/// Returns a message when ruchy's parser or transpiler rejects the source.
pub fn to_rust(ruchy: &str) -> Result<String, String> {
    let ast = crate::Parser::new(ruchy)
        .parse()
        .map_err(|e| format!("ruchy's parser rejects the lowered source: {e}"))?;
    let tokens = crate::Transpiler::new()
        .transpile_to_program(&ast)
        .map_err(|e| format!("ruchy's transpiler rejects the lowered source: {e}"))?;
    let file = syn::parse2::<syn::File>(tokens)
        .map_err(|e| format!("ruchy's transpiler produced tokens that are not Rust: {e}"))?;
    Ok(prettyplease::unparse(&file))
}

/// Lower `program` — which must hold exactly one `job` and must check clean —
/// to ruchy source. `source` is the text the tree was parsed from, for the
/// positions of refusals.
///
/// Terms resolve through `lex`, the checker's own lookup
/// ([`super::check::load_lexicon`]), so check and lower cannot disagree on
/// what a word means.
///
/// # Errors
///
/// `RHL-L001` for a construct the v0 lowering does not cover.
pub(crate) fn lower(
    file: &str,
    source: &str,
    program: &Program,
    lex: &Lexicon,
) -> Result<String, Diagnostic> {
    lower_with(Form::Decide, file, source, program, lex).map(|(text, _, _)| text)
}

/// The `decide` lowering, the measures and actions it uses, and — for every
/// form but [`Form::Decide`] — the `expect` guards and, for [`Form::Tests`],
/// the examples and their runner (RHL-5, [`examples`]).
fn lower_with(
    form: Form,
    file: &str,
    source: &str,
    program: &Program,
    lex: &Lexicon,
) -> Result<(String, Uses, String), Diagnostic> {
    let refuse = |code: &'static str, (span, message): Refusal| -> Diagnostic {
        Diagnostic::error(code, file, source, span, message)
    };
    let unit = single_job(program).map_err(|r| refuse(codes::L001, r))?;
    let mut l = Lowerer::new(lex, unit);
    l.stmts(&unit.body).map_err(|r| refuse(codes::L001, r))?;
    let checks = match form {
        Form::Decide => String::new(),
        _ => l
            .checks(source, unit, form == Form::Tests)
            .map_err(|(code, r)| refuse(code, r))?,
    };
    Ok((l.render(program, unit), l.uses(), checks))
}

/// A refusal before it becomes a diagnostic: where, and why.
type Refusal = (Span, String);

fn single_job(program: &Program) -> Result<&Unit, Refusal> {
    let units: Vec<&Unit> = program
        .decls
        .iter()
        .filter_map(|d| match d {
            Decl::Unit(u) => Some(u),
            Decl::Use(_) => None,
        })
        .collect();
    let unit = match units.as_slice() {
        [one] => *one,
        [] => {
            return Err((
                Span::default(),
                "the program has no unit to lower".to_string(),
            ))
        }
        [_, second, ..] => {
            let m = "v0 lowers one unit per file; this file has more than one".to_string();
            return Err((second.span, m));
        }
    };
    if unit.kind != UnitKind::Job {
        let m = format!(
            "a `{}` is not lowered in v0: RHL-4 lowers `job` only (RHL-12 owns the other unit kinds)",
            unit.kind.keyword()
        );
        return Err((unit.span, m));
    }
    Ok(unit)
}

// ---------------------------------------------------------------- types

/// The type of a lowered value, and its ruchy representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ty {
    Text,
    Size,
    Duration,
    Percent,
    Count,
    Bool,
}

impl Ty {
    /// The type a vocabulary names; `None` for a type with no v0 form.
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "Text" => Some(Self::Text),
            "Size" => Some(Self::Size),
            "Duration" => Some(Self::Duration),
            "Percent" => Some(Self::Percent),
            "Count" => Some(Self::Count),
            "Bool" => Some(Self::Bool),
            _ => None,
        }
    }

    fn rust(self) -> &'static str {
        match self {
            Self::Text => "String",
            Self::Bool => "bool",
            _ => "i64",
        }
    }

    /// How a `Facts` comment describes the field.
    fn describe(self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Size => "Size, bytes",
            Self::Duration => "Duration, seconds",
            Self::Percent => "Percent, hundredths of a percent",
            Self::Count => "Count",
            Self::Bool => "Bool",
        }
    }

    /// The value an attribute a `with` block does not set takes.
    fn zero(self) -> &'static str {
        match self {
            Self::Text => "String::new()",
            Self::Bool => "false",
            _ => "0",
        }
    }
}

/// The built-in unit table: `(unit term, the type it gives, factor to the
/// base unit)`. A vocabulary names a unit and its type; the factor is here.
const UNIT_SCALE: &[(&str, &str, i64)] = &[
    ("B", "Size", 1),
    ("KB", "Size", 1_000),
    ("MB", "Size", 1_000_000),
    ("GB", "Size", 1_000_000_000),
    ("TB", "Size", 1_000_000_000_000),
    ("KiB", "Size", 1 << 10),
    ("MiB", "Size", 1 << 20),
    ("GiB", "Size", 1 << 30),
    ("TiB", "Size", 1 << 40),
    ("second", "Duration", 1),
    ("seconds", "Duration", 1),
    ("minute", "Duration", 60),
    ("minutes", "Duration", 60),
    ("hour", "Duration", 3_600),
    ("hours", "Duration", 3_600),
    ("day", "Duration", 86_400),
    ("days", "Duration", 86_400),
];

/// `%` counts hundredths of a percent, so `90 %` is `9000`.
const PERCENT_SCALE: i64 = 100;

// ---------------------------------------------------------------- operands

/// A lowered operand: its expression, its type, and whether it is a literal
/// (a literal Text is a `&str`; a place of type Text is a `String`).
struct Operand {
    expr: String,
    ty: Ty,
    literal: bool,
}

impl Operand {
    fn literal(expr: String, ty: Ty) -> Self {
        Self {
            expr,
            ty,
            literal: true,
        }
    }

    fn place(expr: String, ty: Ty) -> Self {
        Self {
            expr,
            ty,
            literal: false,
        }
    }

    /// The operand as an owned value, for a `let`, a `set` or a field.
    fn owned(self) -> String {
        match (self.ty, self.literal) {
            (Ty::Text, true) => format!("{}.to_string()", self.expr),
            (Ty::Text, false) => format!("{}.clone()", self.expr),
            _ => self.expr,
        }
    }

    /// The operand as the argument of `contains`.
    fn pattern(self) -> String {
        if self.literal {
            self.expr
        } else {
            format!("&{}", self.expr)
        }
    }
}

/// A ruchy string literal for the RHL text `value`. RHL strings hold no `"`
/// (the grammar), so only backslashes and control characters are escaped.
fn text_literal(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// The RHL spelling of an argument, for `Facts` keys and comments.
fn atom_text(atom: &Atom) -> String {
    match atom {
        Atom::Text(t) => text_literal(&t.value),
        Atom::Quantity(q) => quantity_text(q),
    }
}

fn quantity_text(q: &Quantity) -> String {
    match &q.unit {
        Some(u) => format!("{} {}", q.value.digits, u.text),
        None => q.value.digits.clone(),
    }
}

/// The RHL spelling of an application, for header comments.
fn app_text(app: &App) -> String {
    match app {
        App::Call { phrase, args } => std::iter::once(phrase.text())
            .chain(args.iter().map(atom_text))
            .collect::<Vec<_>>()
            .join(" "),
        App::Quantity(q) => quantity_text(q),
        App::Text(t) => text_literal(&t.value),
    }
}

/// Words to a snake-case identifier part: `disk free of` → `disk_free_of`.
fn snake(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

/// Words to a variant name: `file ticket` → `FileTicket`.
fn pascal(text: &str) -> String {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut cs = w.chars();
            cs.next().map_or(String::new(), |f| {
                f.to_ascii_uppercase().to_string() + &cs.as_str().to_ascii_lowercase()
            })
        })
        .collect()
}

/// Words that cannot be a ruchy or Rust field name as written.
const RESERVED: &[&str] = &[
    "as",
    "async",
    "await",
    "break",
    "const",
    "continue",
    "crate",
    "dyn",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "fun",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "unsafe",
    "use",
    "where",
    "while",
    "yield",
    "actor",
    "spawn",
    "send",
    "try",
    "catch",
    "throw",
    "import",
    "export",
    "from",
    "module",
    "class",
    "requires",
    "ensures",
    "invariant",
    "decreases",
    "infra",
    "signal",
    "none",
    "null",
    "var",
];

/// A field name from a parameter or attribute name; `_` is appended to a
/// reserved word (`type` → `type_`).
fn field_ident(name: &str) -> String {
    let s = snake(name);
    if RESERVED.contains(&s.as_str()) {
        s + "_"
    } else {
        s
    }
}

/// A `let` name: `v_` and its words, so no RHL name meets a ruchy keyword
/// or the generated names `facts`, `plan` and `repeat_<n>`.
fn let_ident(name: &Phrase) -> String {
    format!("v_{}", snake(&name.text()))
}

// ---------------------------------------------------------------- the lowerer

/// One `Facts` field: a measure applied to literal arguments.
struct Fact {
    field: String,
    term: String,
    key: String,
    ty: Ty,
    /// The ruchy expressions of its literal arguments, in order.
    args: Vec<String>,
    span: Span,
}

/// One `Action` variant, with its fields in declaration order.
struct Variant {
    term: String,
    name: String,
    fields: Vec<(String, Ty)>,
    span: Span,
}

struct Lowerer<'a> {
    lex: &'a Lexicon,
    facts: Vec<Fact>,
    /// Every `let`, in order: its name, and the `Facts` field when it is bound
    /// directly to one measure application (what a `given` may set).
    lets: Vec<(String, Option<String>)>,
    /// `Some(plan expression)` while an `expect` or `then` is lowered: names
    /// then resolve only to plan measures ([`examples::PLAN_MEASURES`]).
    plan: Option<&'static str>,
    /// The plan measures used, in first-use order.
    plan_used: Vec<&'static str>,
    variants: Vec<Variant>,
    scopes: Vec<Vec<(String, Ty)>>,
    mutated: Vec<String>,
    repeats: usize,
    lines: Vec<String>,
    depth: usize,
}

impl<'a> Lowerer<'a> {
    fn new(lex: &'a Lexicon, unit: &Unit) -> Self {
        let mut mutated = Vec::new();
        collect_sets(&unit.body, &mut mutated);
        Self {
            lex,
            facts: Vec::new(),
            lets: Vec::new(),
            plan: None,
            plan_used: Vec::new(),
            variants: Vec::new(),
            scopes: vec![Vec::new()],
            mutated,
            repeats: 0,
            lines: Vec::new(),
            depth: 1,
        }
    }

    fn line(&mut self, text: impl Into<String>) {
        let indent = "    ".repeat(self.depth);
        self.lines.push(format!("{indent}{}", text.into()));
    }

    /// The term spelled exactly `text`, by the checker's own lookup.
    fn term(&self, text: &str) -> Option<&'a Term> {
        self.lex.get(text).map(|t| &t.term)
    }

    fn lookup(&self, name: &str) -> Option<Ty> {
        self.scopes
            .iter()
            .rev()
            .flat_map(|s| s.iter().rev())
            .find(|(n, _)| n == name)
            .map(|(_, t)| *t)
    }

    fn bind(&mut self, name: String, ty: Ty) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.push((name, ty));
        }
    }

    // ------------------------------------------------------------ statements

    fn stmts(&mut self, body: &[Stmt]) -> Result<(), Refusal> {
        body.iter().try_for_each(|s| self.stmt(s))
    }

    fn stmt(&mut self, s: &Stmt) -> Result<(), Refusal> {
        match &s.kind {
            StmtKind::Let { name, value } => self.let_stmt(name, value),
            StmtKind::Set { name, value } => self.set_stmt(name, value),
            StmtKind::Action(a) => self.action(a, s.span),
            StmtKind::When {
                cond,
                then_body,
                otherwise,
            } => self.when(cond, then_body, otherwise.as_deref()),
            StmtKind::Repeat { times, until, body } => self.repeat(times, until.as_ref(), body),
            _ => self.other_stmt(s),
        }
    }

    /// Statements that lower to nothing in `decide`, or are refused.
    fn other_stmt(&mut self, s: &Stmt) -> Result<(), Refusal> {
        match &s.kind {
            StmtKind::RunsOn(_) | StmtKind::Every(_) | StmtKind::May(_) if self.depth == 1 => {
                Ok(())
            }
            StmtKind::Expect(_)
            | StmtKind::Example { .. }
            | StmtKind::Given(_)
            | StmtKind::Then(_) => Ok(()),
            _ => Err((s.span, refusal_message(&s.kind).to_string())),
        }
    }

    fn let_stmt(&mut self, name: &Phrase, value: &Cond) -> Result<(), Refusal> {
        let v = self.value(value)?;
        let text = name.text();
        let m = if self.mutated.contains(&text) {
            "mut "
        } else {
            ""
        };
        let ty = v.ty;
        let direct = matches!(value.kind, CondKind::App(_) | CondKind::In { .. });
        let field = v.expr.strip_prefix("facts.").filter(|_| direct);
        self.lets.push((text.clone(), field.map(str::to_string)));
        self.line(format!(
            "let {m}{}: {} = {}",
            let_ident(name),
            ty.rust(),
            v.owned()
        ));
        self.bind(text, ty);
        Ok(())
    }

    fn set_stmt(&mut self, name: &Phrase, value: &Cond) -> Result<(), Refusal> {
        if self.lookup(&name.text()).is_none() {
            return Err(out_of_scope(name));
        }
        let v = self.value(value)?;
        self.line(format!("{} = {}", let_ident(name), v.owned()));
        Ok(())
    }

    /// A nested body: its own scope, one level deeper, and `()` when it
    /// lowers to nothing — an empty `{ }` is an object literal in ruchy.
    fn block(&mut self, body: &[Stmt]) -> Result<(), Refusal> {
        self.depth += 1;
        self.scopes.push(Vec::new());
        let before = self.lines.len();
        let result = self.stmts(body);
        if self.lines.len() == before {
            self.line("()");
        }
        self.scopes.pop();
        self.depth -= 1;
        result
    }

    fn when(
        &mut self,
        cond: &Cond,
        then_body: &[Stmt],
        otherwise: Option<&[Stmt]>,
    ) -> Result<(), Refusal> {
        let c = self.condition(cond)?;
        self.line(format!("if {c} {{"));
        self.block(then_body)?;
        if let Some(o) = otherwise {
            self.line("} else {");
            self.block(o)?;
        }
        self.line("}");
        Ok(())
    }

    /// `repeat at most N times [until C]`: a counted `while`; `until` is
    /// tested before each round and breaks out when it holds.
    fn repeat(&mut self, times: &Int, until: Option<&Cond>, body: &[Stmt]) -> Result<(), Refusal> {
        let n: i64 = times.digits.parse().map_err(|_| {
            let m = format!(
                "`repeat at most {} times` does not fit in i64",
                times.digits
            );
            (times.span, m)
        })?;
        let counter = format!("repeat_{}", self.repeats);
        self.repeats += 1;
        self.line(format!("let mut {counter}: i64 = 0"));
        self.line(format!("while {counter} < {n} {{"));
        self.depth += 1;
        self.scopes.push(Vec::new());
        if let Some(u) = until {
            let c = self.condition(u)?;
            self.line(format!("if {c} {{"));
            self.depth += 1;
            self.line("break");
            self.depth -= 1;
            self.line("}");
        }
        self.stmts(body)?;
        self.line(format!("{counter} = {counter} + 1"));
        self.scopes.pop();
        self.depth -= 1;
        self.line("}");
        Ok(())
    }

    // ------------------------------------------------------------ actions

    fn action(&mut self, a: &Action, span: Span) -> Result<(), Refusal> {
        let App::Call { phrase, args } = &a.head else {
            return Err((
                span,
                "an action statement starts with an action term".to_string(),
            ));
        };
        let term = self
            .term(&phrase.text())
            .filter(|t| t.kind == TermKind::Action)
            .ok_or_else(|| (phrase.span, format!("`{}` is not an action", phrase.text())))?;
        let name = self.variant(term, span)?;
        let mut fields = self.takes_fields(term, args, a.target.as_ref(), span)?;
        fields.extend(self.attribute_fields(term, a.with.as_deref(), span)?);
        let push = if fields.is_empty() {
            format!("plan.push(Action::{name})")
        } else {
            format!("plan.push(Action::{name} {{ {} }})", fields.join(", "))
        };
        self.line(push);
        Ok(())
    }

    /// The variant for `term`, registered on first use.
    fn variant(&mut self, term: &Term, span: Span) -> Result<String, Refusal> {
        if let Some(v) = self.variants.iter().find(|v| v.term == term.term) {
            return Ok(v.name.clone());
        }
        let mut fields: Vec<(String, Ty)> = Vec::new();
        for p in term.takes.iter().chain(&term.attributes) {
            let ty = param_ty(term, p, span)?;
            let ident = field_ident(&p.name);
            if fields.iter().any(|(f, _)| *f == ident) {
                let m = format!("`{}` has two fields named `{ident}`", term.term);
                return Err((span, m));
            }
            fields.push((ident, ty));
        }
        let name = pascal(&term.term);
        self.variants.push(Variant {
            term: term.term.clone(),
            name: name.clone(),
            fields,
            span,
        });
        Ok(name)
    }

    /// `field: value` for each parameter: the target (after `in`) fills the
    /// one named after its entity, the arguments fill the rest in order.
    fn takes_fields(
        &self,
        term: &Term,
        args: &[Atom],
        target: Option<&App>,
        span: Span,
    ) -> Result<Vec<String>, Refusal> {
        let target = target.map(|t| self.entity_instance(t)).transpose()?;
        let mut rest = args.iter();
        let mut out = Vec::new();
        for p in &term.takes {
            let value = match &target {
                Some((entity, instance)) if *entity == p.name => {
                    format!("{}.to_string()", text_literal(instance))
                }
                _ => {
                    let atom = rest.next().ok_or_else(|| {
                        (
                            span,
                            format!("`{}` has no value for `{}`", term.term, p.name),
                        )
                    })?;
                    self.atom(atom)?.owned()
                }
            };
            out.push(format!("{}: {value}", field_ident(&p.name)));
        }
        Ok(out)
    }

    /// `field: value` for each declared attribute: the `with` line that sets
    /// it, else the type's zero value (`String::new()`, `0`, `false`).
    fn attribute_fields(
        &self,
        term: &Term,
        with: Option<&[Stmt]>,
        span: Span,
    ) -> Result<Vec<String>, Refusal> {
        let set = attribute_lines(term, with.unwrap_or_default(), span)?;
        let mut out = Vec::new();
        for p in &term.attributes {
            let value = match set.iter().find(|(n, _)| *n == p.name) {
                Some((_, atom)) => self.atom(atom)?.owned(),
                None => param_ty(term, p, span)?.zero().to_string(),
            };
            out.push(format!("{}: {value}", field_ident(&p.name)));
        }
        Ok(out)
    }

    /// `(entity term, instance)` for a target such as `repo "paiml/infra"`.
    fn entity_instance(&self, app: &App) -> Result<(String, String), Refusal> {
        let not_entity = |span: Span| {
            (
                span,
                "the target after `in` must be an entity instance".to_string(),
            )
        };
        let App::Call { phrase, args } = app else {
            return Err(not_entity(Span::default()));
        };
        let (first, rest) = phrase
            .words
            .split_first()
            .ok_or_else(|| not_entity(phrase.span))?;
        let entity = self
            .term(&first.text)
            .filter(|t| t.kind == TermKind::Entity)
            .ok_or_else(|| not_entity(phrase.span))?;
        match (rest, args.as_slice()) {
            ([w], []) => Ok((entity.term.clone(), w.text.clone())),
            ([], [Atom::Text(t)]) => Ok((entity.term.clone(), t.value.clone())),
            _ => Err(not_entity(phrase.span)),
        }
    }

    // ------------------------------------------------------------ values

    /// A condition in a value position (`let`, `set`): an operand, or a Bool.
    fn value(&mut self, c: &Cond) -> Result<Operand, Refusal> {
        match &c.kind {
            CondKind::App(a) => self.operand(a),
            CondKind::In { left, right } => self.measure_in(left, right, c.span),
            _ => Ok(Operand::place(self.condition(c)?, Ty::Bool)),
        }
    }

    /// A condition as a ruchy Bool expression. `and` binds tighter than `or`
    /// in both languages, and `not` applies to one comparison (grammar
    /// `Cmp`), so no parentheses are needed — and none are emitted, because
    /// ruchy's transpiler drops the parentheses of `!( … )`.
    fn condition(&mut self, c: &Cond) -> Result<String, Refusal> {
        match &c.kind {
            CondKind::Or(a, b) => Ok(format!("{} || {}", self.condition(a)?, self.condition(b)?)),
            CondKind::And(a, b) => Ok(format!("{} && {}", self.conjunct(a)?, self.conjunct(b)?)),
            CondKind::Not(inner) => self.negated(inner),
            CondKind::Compare { left, op, right } => self.compare(left, *op, right, c.span, false),
            _ => Ok(self.value(c)?.expr),
        }
    }

    /// One side of `and`; an `or` there (only a hand-built tree has one) is
    /// parenthesised.
    fn conjunct(&mut self, c: &Cond) -> Result<String, Refusal> {
        let text = self.condition(c)?;
        Ok(match c.kind {
            CondKind::Or(..) => format!("({text})"),
            _ => text,
        })
    }

    /// `not c`, pushed into `c`: a comparison is inverted, `not not c` is
    /// `c`, a Bool name is `!name`, and `and`/`or` follow De Morgan.
    fn negated(&mut self, c: &Cond) -> Result<String, Refusal> {
        match &c.kind {
            CondKind::Not(inner) => self.condition(inner),
            CondKind::Compare { left, op, right } => self.compare(left, *op, right, c.span, true),
            CondKind::Or(a, b) => Ok(format!("({}) && ({})", self.negated(a)?, self.negated(b)?)),
            CondKind::And(a, b) => Ok(format!("({}) || ({})", self.negated(a)?, self.negated(b)?)),
            _ => Ok(format!("!{}", self.value(c)?.expr)),
        }
    }

    fn compare(
        &mut self,
        left: &App,
        op: CompOp,
        right: &App,
        span: Span,
        negate: bool,
    ) -> Result<String, Refusal> {
        let l = self.operand(left)?;
        let r = self.operand(right)?;
        match op {
            CompOp::IsOneOf => Err((
                span,
                "`is one of` is not lowered in v0: no v2 term gives a list to compare against"
                    .to_string(),
            )),
            CompOp::Contains => {
                let not = if negate { "!" } else { "" };
                Ok(format!("{not}{}.contains({})", l.expr, r.pattern()))
            }
            _ => {
                let op = if negate { inverse(op) } else { op };
                Ok(format!("{} {} {}", l.expr, symbol(op), r.expr))
            }
        }
    }

    fn operand(&mut self, app: &App) -> Result<Operand, Refusal> {
        match app {
            App::Quantity(q) => self.quantity(q),
            App::Text(t) => Ok(Operand::literal(text_literal(&t.value), Ty::Text)),
            App::Call { phrase, args } => self.call(phrase, args),
        }
    }

    fn atom(&self, atom: &Atom) -> Result<Operand, Refusal> {
        match atom {
            Atom::Text(t) => Ok(Operand::literal(text_literal(&t.value), Ty::Text)),
            Atom::Quantity(q) => self.quantity(q),
        }
    }

    /// A `let` name in scope, or a measure applied to literal arguments.
    fn call(&mut self, phrase: &Phrase, args: &[Atom]) -> Result<Operand, Refusal> {
        if let Some(plan) = self.plan {
            return self.plan_measure(phrase, args, plan);
        }
        let text = phrase.text();
        if let (true, Some(ty)) = (args.is_empty(), self.lookup(&text)) {
            return Ok(Operand::place(let_ident(phrase), ty));
        }
        let Some(term) = self.term(&text) else {
            return Err(out_of_scope(phrase));
        };
        check_fact_term(term, phrase.span)?;
        let key: Vec<String> = args.iter().map(atom_text).collect();
        let exprs = args
            .iter()
            .map(|a| self.atom(a).map(|o| o.expr))
            .collect::<Result<Vec<_>, _>>()?;
        self.fact(term, &key.join(" "), exprs, phrase.span)
    }

    /// `<term> in <right>`, as `tickets filed in "paiml/infra"`: the measure
    /// `<term> in` applied to one literal or entity instance.
    fn measure_in(&mut self, left: &App, right: &App, span: Span) -> Result<Operand, Refusal> {
        let App::Call { phrase, .. } = left else {
            return Err((
                span,
                "`in` follows only a term that ends in `in`".to_string(),
            ));
        };
        let name = format!("{} in", phrase.text());
        if self.plan.is_some() {
            return Err(examples::not_a_plan_measure(&name, span));
        }
        let term = self
            .term(&name)
            .ok_or_else(|| (span, format!("`{name}` is not a term")))?;
        let (key, arg) = match right {
            App::Text(t) => (text_literal(&t.value), text_literal(&t.value)),
            App::Quantity(q) => (quantity_text(q), self.quantity(q)?.expr),
            App::Call { .. } => match self.entity_instance(right) {
                Ok((_, instance)) => (text_literal(&instance), text_literal(&instance)),
                Err(_) => return Err(non_literal(&name, span)),
            },
        };
        self.fact(term, &key, vec![arg], span)
    }

    /// The `Facts` field for `term` applied to the arguments spelled `key`,
    /// registered on first use.
    fn fact(
        &mut self,
        term: &Term,
        key: &str,
        args: Vec<String>,
        span: Span,
    ) -> Result<Operand, Refusal> {
        check_fact_term(term, span)?;
        let ty = gives_ty(term, span)?;
        if let Some(f) = self
            .facts
            .iter()
            .find(|f| f.term == term.term && f.key == key)
        {
            return Ok(Operand::place(format!("facts.{}", f.field), f.ty));
        }
        let field = format!("{}_{}", snake(&term.term), self.facts.len());
        self.facts.push(Fact {
            field: field.clone(),
            term: term.term.clone(),
            key: key.to_string(),
            ty,
            args,
            span,
        });
        Ok(Operand::place(format!("facts.{field}"), ty))
    }

    /// `INT [unit]` in the base unit of its type, checked to fit in i64.
    fn quantity(&self, q: &Quantity) -> Result<Operand, Refusal> {
        let (ty, factor) = match q.unit.as_ref().map(|u| u.text.as_str()) {
            None => (Ty::Count, 1),
            Some("%") => (Ty::Percent, PERCENT_SCALE),
            Some(unit) => self.unit_scale(unit, q.span)?,
        };
        let value = q
            .value
            .digits
            .parse::<i64>()
            .ok()
            .and_then(|v| v.checked_mul(factor))
            .ok_or_else(|| {
                (
                    q.span,
                    format!("`{}` does not fit in i64", quantity_text(q)),
                )
            })?;
        Ok(Operand::literal(value.to_string(), ty))
    }

    fn unit_scale(&self, unit: &str, span: Span) -> Result<(Ty, i64), Refusal> {
        let gives = self
            .lex
            .unit(unit)
            .and_then(|t| t.term.gives.as_deref())
            .unwrap_or_default();
        let (_, _, factor) = UNIT_SCALE
            .iter()
            .find(|(u, g, _)| *u == unit && *g == gives)
            .ok_or_else(|| (span, format!("the unit `{unit}` ({gives}) has no v0 scale")))?;
        let ty = Ty::from_name(gives).ok_or_else(|| (span, format!("`{unit}` gives no type")))?;
        Ok((ty, *factor))
    }

    // ------------------------------------------------------------ output

    /// The binding of the term spelled `text`, empty when it has none.
    fn binding(&self, text: &str) -> String {
        self.term(text)
            .map(|t| t.lowers_to.clone())
            .unwrap_or_default()
    }

    /// The measures and actions this job uses, in first-use order.
    fn uses(&self) -> Uses {
        let measures = self
            .facts
            .iter()
            .map(|f| MeasureUse {
                field: f.field.clone(),
                term: f.term.clone(),
                binding: self.binding(&f.term),
                args: f.args.clone(),
                span: f.span,
            })
            .collect();
        let actions = self
            .variants
            .iter()
            .map(|v| ActionUse {
                variant: v.name.clone(),
                term: v.term.clone(),
                binding: self.binding(&v.term),
                fields: v
                    .fields
                    .iter()
                    .map(|(n, t)| (n.clone(), *t == Ty::Text))
                    .collect(),
                span: v.span,
            })
            .collect();
        Uses { measures, actions }
    }

    fn render(&self, program: &Program, unit: &Unit) -> String {
        let mut out = header(program, unit);
        out.push('\n');
        out.push_str(&self.render_facts());
        out.push('\n');
        out.push_str(&self.render_actions());
        out.push_str("\nfun decide(facts: Facts) -> Vec<Action> {\n");
        out.push_str("    let mut plan: Vec<Action> = Vec::new()\n");
        for l in &self.lines {
            out.push_str(l);
            out.push('\n');
        }
        out.push_str("    plan\n}\n");
        out
    }

    fn render_facts(&self) -> String {
        let mut out = String::new();
        for f in &self.facts {
            let applied = [f.term.as_str(), f.key.as_str()].join(" ");
            let desc = f.ty.describe();
            out.push_str(&format!(
                "// Facts.{}: {} ({desc})\n",
                f.field,
                applied.trim_end()
            ));
        }
        out.push_str("struct Facts {\n");
        for f in &self.facts {
            out.push_str(&format!("    {}: {},\n", f.field, f.ty.rust()));
        }
        out.push_str("}\n");
        out
    }

    fn render_actions(&self) -> String {
        let mut out = String::from("enum Action {\n");
        for v in &self.variants {
            let fields: Vec<String> = v
                .fields
                .iter()
                .map(|(n, t)| format!("{n}: {}", t.rust()))
                .collect();
            if fields.is_empty() {
                out.push_str(&format!("    {},\n", v.name));
            } else {
                out.push_str(&format!("    {} {{ {} }},\n", v.name, fields.join(", ")));
            }
        }
        out.push_str("}\n");
        out
    }
}

// ---------------------------------------------------------------- free helpers

/// The ruchy operator for an ordering or equality comparison.
fn symbol(op: CompOp) -> &'static str {
    match op {
        CompOp::Is => "==",
        CompOp::IsNot => "!=",
        CompOp::IsBelow => "<",
        CompOp::IsAbove => ">",
        CompOp::IsAtLeast => ">=",
        CompOp::IsAtMost => "<=",
        CompOp::IsOneOf | CompOp::Contains => "",
    }
}

/// The comparison that holds exactly when `op` does not.
fn inverse(op: CompOp) -> CompOp {
    match op {
        CompOp::Is => CompOp::IsNot,
        CompOp::IsNot => CompOp::Is,
        CompOp::IsBelow => CompOp::IsAtLeast,
        CompOp::IsAtLeast => CompOp::IsBelow,
        CompOp::IsAbove => CompOp::IsAtMost,
        CompOp::IsAtMost => CompOp::IsAbove,
        other => other,
    }
}

fn refusal_message(kind: &StmtKind) -> &'static str {
    match kind {
        StmtKind::GiveBack(_) => {
            "`give back` is not lowered in v0: a job gives back its plan of actions, not a value"
        }
        StmtKind::StopWith(_) => {
            "`stop with` is not lowered in v0: `decide` returns its whole plan and has no early stop with a reason"
        }
        StmtKind::WaitUpTo(_) => {
            "`wait up to` is not lowered in v0: waiting is part of running a job, which RHL-4b binds"
        }
        StmtKind::ForEach { .. } => {
            "`for each` is not lowered in v0: no term of a v2 vocabulary gives a finite collection"
        }
        _ => "`runs on`, `every` and `may` are lowered only at the top level of a job",
    }
}

fn out_of_scope(name: &Phrase) -> Refusal {
    let m = format!(
        "`{}` is not in scope here: v0 lowering keeps a `let` inside the block that binds it",
        name.text()
    );
    (name.span, m)
}

fn non_literal(term: &str, span: Span) -> Refusal {
    let m = format!(
        "`{term}` is applied to a value that is not a literal: v0 reads every fact before `decide` runs, so its arguments must be literals"
    );
    (span, m)
}

/// A term `Facts` can hold: a measure with an effect, that is, one read from
/// the world. A noun, or a measure with no effect (such as `ticket count`,
/// read from the run's own plan), has no v0 binding.
fn check_fact_term(term: &Term, span: Span) -> Result<(), Refusal> {
    match (term.kind, term.effect.as_deref()) {
        (TermKind::Measure, Some(_)) => Ok(()),
        (TermKind::Measure, None) => Err((
            span,
            format!(
                "`{}` has no effect, so it is not read from the world: it is not a fact, and its binding is not v0's",
                term.term
            ),
        )),
        _ => Err((
            span,
            format!("`{}` is not a measure: only measures are lowered as facts in v0", term.term),
        )),
    }
}

fn gives_ty(term: &Term, span: Span) -> Result<Ty, Refusal> {
    let gives = term.gives.as_deref().unwrap_or_default();
    Ty::from_name(gives).ok_or_else(|| {
        (
            span,
            format!(
                "`{}` gives {gives}, which has no v0 representation",
                term.term
            ),
        )
    })
}

fn param_ty(term: &Term, p: &Param, span: Span) -> Result<Ty, Refusal> {
    Ty::from_name(&p.ty).ok_or_else(|| {
        let m = format!(
            "`{}` takes `{}` of type {}, which has no v0 representation",
            term.term, p.name, p.ty
        );
        (span, m)
    })
}

/// The `(attribute, value)` lines of a `with` block, each set at most once.
fn attribute_lines<'s>(
    term: &Term,
    with: &'s [Stmt],
    span: Span,
) -> Result<Vec<(String, &'s Atom)>, Refusal> {
    let mut out: Vec<(String, &Atom)> = Vec::new();
    for s in with {
        let (name, atom) = attribute_line(term, s)?;
        if out.iter().any(|(n, _)| *n == name) {
            return Err((s.span, format!("the attribute `{name}` is set twice")));
        }
        out.push((name, atom));
    }
    if !with.is_empty() && term.attributes.is_empty() {
        let m = format!(
            "`{}` declares no attributes, so its `with` block is not lowered",
            term.term
        );
        return Err((span, m));
    }
    Ok(out)
}

fn attribute_line<'s>(term: &Term, s: &'s Stmt) -> Result<(String, &'s Atom), Refusal> {
    let refuse = || {
        let m = format!(
            "only `<attribute> <value>` lines of `{}` are lowered in a `with` block",
            term.term
        );
        (s.span, m)
    };
    let StmtKind::Action(Action {
        head: App::Call { phrase, args },
        target: None,
        with: None,
    }) = &s.kind
    else {
        return Err(refuse());
    };
    match args.as_slice() {
        [atom] => Ok((phrase.text(), atom)),
        _ => Err(refuse()),
    }
}

/// Every name some `set` assigns, anywhere in `body`.
fn collect_sets(body: &[Stmt], out: &mut Vec<String>) {
    for s in body {
        if let StmtKind::Set { name, .. } = &s.kind {
            out.push(name.text());
        }
        for child in child_bodies(&s.kind) {
            collect_sets(child, out);
        }
    }
}

/// The nested bodies of a block statement that lower into `decide`.
fn child_bodies(kind: &StmtKind) -> Vec<&[Stmt]> {
    match kind {
        StmtKind::When {
            then_body,
            otherwise,
            ..
        } => vec![
            then_body.as_slice(),
            otherwise.as_deref().unwrap_or_default(),
        ],
        StmtKind::Repeat { body, .. } | StmtKind::ForEach { body, .. } => vec![body.as_slice()],
        _ => Vec::new(),
    }
}

/// The leading comment block: provenance, the job, its vocabularies, and its
/// `runs on` / `every` / `may` lines, which RHL-4b's runner reads.
fn header(program: &Program, unit: &Unit) -> String {
    let mut lines = vec![
        "// Lowered from RHL by ruchy (RHL-4). Regenerate it from the RHL source; do not edit."
            .to_string(),
        format!("// job {}", text_literal(&unit.name.value)),
    ];
    for decl in &program.decls {
        if let Decl::Use(u) = decl {
            lines.push(format!("// use vocabulary {}", u.vocabulary.text()));
        }
    }
    lines.extend(unit.body.iter().filter_map(header_line));
    if unit.body.iter().any(|s| is_rhl5(&s.kind)) {
        lines.push("// Not lowered into decide: expect and example (RHL-5).".to_string());
    }
    lines.iter().map(|l| format!("{l}\n")).collect()
}

fn header_line(s: &Stmt) -> Option<String> {
    match &s.kind {
        StmtKind::RunsOn(app) => Some(format!("// runs on {}", app_text(app))),
        StmtKind::Every(q) => Some(format!("// every {}", quantity_text(q))),
        StmtKind::May(e) => Some(format!("// may {} {}", e.verb.word(), e.target.text())),
        _ => None,
    }
}

fn is_rhl5(kind: &StmtKind) -> bool {
    matches!(kind, StmtKind::Expect(_) | StmtKind::Example { .. })
}

#[path = "examples.rs"]
mod examples;

pub use examples::example_names;
pub(crate) use examples::{plan_measure_fn, PLAN_MEASURES};

#[cfg(test)]
#[path = "lower_tests.rs"]
mod lower_tests;
