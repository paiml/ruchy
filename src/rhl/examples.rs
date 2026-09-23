//! RHL-5a: `example` lowers to a test and `expect` to a guard on the decided
//! plan (spec RHL-001 §3.1 principle 7, §4, §5 `ruchy test`, §9.4).
//!
//! - `given X is V` sets one `Facts` field: `X` is a measure applied to
//!   literal arguments that the job reads, or a `let` bound directly to one.
//!   Every fact the job reads must be given (`RHL-X002` otherwise), so a test
//!   never reads the host.
//! - `then C` and `expect C` are conditions over the decided plan. Their names
//!   resolve only to the plan measures of [`PLAN_MEASURES`].
//! - `expect` becomes `fun expect_<n>(plan: &Vec<Action>) -> bool`, and
//!   `failed_expectation` names the first that fails. The built job's `main`
//!   applies nothing when one fails; every example checks them too.
//!
//! Everything else is refused by name with `RHL-L001`. The rules are written
//! down in `docs/rhl/rhl-5.md`.

use super::{atom_text, child_bodies, snake, text_literal, Lowerer, Operand, Refusal, Ty};
use crate::rhl::codes;
use crate::rhl::tree::{
    App, Atom, CompOp, Cond, CondKind, Decl, Phrase, Program, Span, Stmt, StmtKind, Unit,
};

/// The plan-measure table: a measure read from the decided plan (never from
/// the world), and the action term whose occurrences in the plan it counts.
pub(crate) const PLAN_MEASURES: &[(&str, &str)] = &[("ticket count", "file ticket")];

/// The ruchy function that computes plan measure `term`: `plan_ticket_count`.
/// The unit contract's formulas name the same function (RHL-5b).
pub(crate) fn plan_measure_fn(term: &str) -> String {
    format!("plan_{}", snake(term))
}

/// A refusal with its code: `RHL-L001`, or `RHL-X002` for an unset fact.
type Coded = (&'static str, Refusal);

fn l001(r: Refusal) -> Coded {
    (codes::L001, r)
}

/// One lowered `example`.
struct Example {
    name: String,
    /// The `Facts { … }` literal its `given` lines build.
    facts: String,
    /// Each `then`: its RHL text and its ruchy condition.
    thens: Vec<(String, String)>,
}

/// The names of the `example` blocks of the program's first unit, in source
/// order: the order in which the test program reports them.
#[must_use]
pub fn example_names(program: &Program) -> Vec<String> {
    let unit = program.decls.iter().find_map(|d| match d {
        Decl::Unit(u) => Some(u),
        Decl::Use(_) => None,
    });
    unit.map_or_else(Vec::new, |u| {
        u.body
            .iter()
            .filter_map(|s| match &s.kind {
                StmtKind::Example { name, .. } => Some(name.value.clone()),
                _ => None,
            })
            .collect()
    })
}

impl Lowerer<'_> {
    /// The plan measures, `expect` guards and — when `tests` — examples and
    /// their runner `main`, after `decide` has been lowered.
    pub(super) fn checks(
        &mut self,
        source: &str,
        unit: &Unit,
        tests: bool,
    ) -> Result<String, Coded> {
        refuse_nested(&unit.body, 0).map_err(l001)?;
        let expects = self.expectations(source, &unit.body).map_err(l001)?;
        let examples = if tests {
            self.examples(source, &unit.body)?
        } else {
            Vec::new()
        };
        let mut out = self.render_plan_measures();
        out.push_str(&render_expectations(&expects));
        if tests {
            out.push_str(&render_examples(&examples));
        }
        Ok(out)
    }

    /// A name in an `expect` or `then`: a plan measure, read from `plan`.
    pub(super) fn plan_measure(
        &mut self,
        phrase: &Phrase,
        args: &[Atom],
        plan: &str,
    ) -> Result<Operand, Refusal> {
        let text = phrase.text();
        let Some((term, _)) = PLAN_MEASURES
            .iter()
            .find(|(t, _)| *t == text)
            .filter(|_| args.is_empty())
        else {
            return Err(not_a_plan_measure(&text, phrase.span));
        };
        if !self.plan_used.contains(term) {
            self.plan_used.push(term);
        }
        let expr = format!("{}({plan})", plan_measure_fn(term));
        Ok(Operand::place(expr, Ty::Count))
    }

    /// `c` as a condition over the plan named `plan`.
    fn plan_condition(&mut self, c: &Cond, plan: &'static str) -> Result<String, Refusal> {
        self.plan = Some(plan);
        let result = self.condition(c);
        self.plan = None;
        result
    }

    fn expectations(
        &mut self,
        source: &str,
        body: &[Stmt],
    ) -> Result<Vec<(String, String)>, Refusal> {
        let mut out = Vec::new();
        for s in body {
            if let StmtKind::Expect(c) = &s.kind {
                let cond = self.plan_condition(c, "plan")?;
                out.push((span_text(source, s.span), cond));
            }
        }
        Ok(out)
    }

    fn examples(&mut self, source: &str, body: &[Stmt]) -> Result<Vec<Example>, Coded> {
        let mut out = Vec::new();
        for s in body {
            if let StmtKind::Example { name, body } = &s.kind {
                out.push(self.example(source, s.span, &name.value, body)?);
            }
        }
        Ok(out)
    }

    fn example(
        &mut self,
        source: &str,
        span: Span,
        name: &str,
        body: &[Stmt],
    ) -> Result<Example, Coded> {
        let mut set: Vec<(String, String)> = Vec::new();
        let mut thens = Vec::new();
        for s in body {
            match &s.kind {
                StmtKind::Given(c) => {
                    let given = self.given(c).map_err(l001)?;
                    add_given(&mut set, given, s.span).map_err(l001)?;
                }
                StmtKind::Then(c) => {
                    let cond = self.plan_condition(c, "&plan").map_err(l001)?;
                    thens.push((span_text(source, s.span), cond));
                }
                _ => return Err(l001((s.span, only_given_then()))),
            }
        }
        let facts = self.facts_literal(name, span, &set)?;
        Ok(Example {
            name: name.to_string(),
            facts,
            thens,
        })
    }

    /// `given X is V`: the `Facts` field `X` names and `V` as its value.
    fn given(&mut self, c: &Cond) -> Result<(String, String), Refusal> {
        let CondKind::Compare {
            left,
            op: CompOp::Is,
            right,
        } = &c.kind
        else {
            return Err((c.span, only_given_is()));
        };
        let (field, ty) = self.given_subject(left, c.span)?;
        let value = self.given_value(right, ty, c.span)?;
        Ok((field, value))
    }

    fn given_subject(&self, left: &App, span: Span) -> Result<(String, Ty), Refusal> {
        let App::Call { phrase, args } = left else {
            return Err((span, only_given_is()));
        };
        let text = phrase.text();
        if args.is_empty() && self.lets.iter().any(|(n, _)| *n == text) {
            return self.given_let(&text, phrase.span);
        }
        let key: Vec<String> = args.iter().map(atom_text).collect();
        let key = key.join(" ");
        self.facts
            .iter()
            .find(|f| f.term == text && f.key == key)
            .map(|f| (f.field.clone(), f.ty))
            .ok_or_else(|| {
                let named = format!("{text} {key}");
                let m = format!(
                    "`{}` is not a fact this job reads, nor a `let` bound to one: a `given` sets only \
                     a fact the job reads",
                    named.trim_end()
                );
                (span, m)
            })
    }

    /// A `let` name: allowed when it is bound once, directly to one measure
    /// application, and no `set` changes it.
    fn given_let(&self, name: &str, span: Span) -> Result<(String, Ty), Refusal> {
        if self.mutated.iter().any(|m| m == name) {
            let m = format!(
                "`{name}` is changed by `set`, so a `given` cannot fix its value; give the \
                 measure it reads instead"
            );
            return Err((span, m));
        }
        let bound: Vec<&Option<String>> = self
            .lets
            .iter()
            .filter(|(n, _)| n == name)
            .map(|(_, f)| f)
            .collect();
        let field = match bound.as_slice() {
            [Some(field)] => field,
            [None] => {
                let m = format!(
                    "`{name}` is not bound directly to one measure application, so a `given` \
                     cannot set it"
                );
                return Err((span, m));
            }
            _ => return Err((span, format!("`{name}` is bound more than once"))),
        };
        let ty = self.facts.iter().find(|f| f.field == *field).map(|f| f.ty);
        ty.map(|t| (field.clone(), t))
            .ok_or_else(|| (span, format!("`{name}` reads no fact")))
    }

    /// The literal a `given` sets, in the base unit of the fact's type.
    fn given_value(&self, right: &App, ty: Ty, span: Span) -> Result<String, Refusal> {
        let operand = match right {
            App::Quantity(q) => self.quantity(q)?,
            App::Text(t) => Operand::literal(text_literal(&t.value), Ty::Text),
            App::Call { .. } => {
                let m = "the value of a `given` must be a literal".to_string();
                return Err((span, m));
            }
        };
        if operand.ty != ty {
            let m = format!(
                "this `given` sets a {} fact to a {} value",
                ty.describe(),
                operand.ty.describe()
            );
            return Err((span, m));
        }
        Ok(operand.owned())
    }

    fn facts_literal(
        &self,
        name: &str,
        span: Span,
        set: &[(String, String)],
    ) -> Result<String, Coded> {
        let mut fields = Vec::new();
        for f in &self.facts {
            let Some((_, value)) = set.iter().find(|(field, _)| *field == f.field) else {
                let applied = format!("{} {}", f.term, f.key);
                let m = format!(
                    "the example {} leaves `{}` unset: a test reads only the facts its `given` \
                     lines state, never the host",
                    text_literal(name),
                    applied.trim_end()
                );
                return Err((codes::X002, (span, m)));
            };
            fields.push(format!("{}: {value}", f.field));
        }
        Ok(format!("Facts {{ {} }}", fields.join(", ")))
    }

    fn render_plan_measures(&self) -> String {
        if self.plan_used.is_empty() {
            return String::new();
        }
        let mut out = String::from("\n// Plan measures (RHL-5): read from the decided plan.\n");
        for term in &self.plan_used {
            out.push_str(&self.render_plan_measure(term));
        }
        out
    }

    /// `fun plan_<term>(plan: &Vec<Action>) -> i64`: how many actions of the
    /// term the table names the plan holds (0 when the job has none).
    fn render_plan_measure(&self, term: &str) -> String {
        let action = PLAN_MEASURES
            .iter()
            .find(|(t, _)| *t == term)
            .map_or("", |(_, a)| a);
        let head = format!(
            "// {term}: the number of `{action}` actions in the plan\nfun {}(plan: &Vec<Action>) -> i64 {{\n",
            plan_measure_fn(term)
        );
        let Some(v) = self.variants.iter().find(|v| v.term == action) else {
            return format!("{head}    0\n}}\n");
        };
        let names: Vec<&str> = v.fields.iter().map(|(n, _)| n.as_str()).collect();
        let pattern = if names.is_empty() {
            format!("Action::{}", v.name)
        } else {
            format!("Action::{} {{ {} }}", v.name, names.join(", "))
        };
        let other = if self.variants.len() > 1 {
            "            _ => false,\n"
        } else {
            ""
        };
        format!(
            "{head}    let mut count: i64 = 0\n    for action in plan.iter() {{\n        \
             let hit: bool = match action {{\n            {pattern} => true,\n{other}        }}\n        \
             if hit {{\n            count = count + 1\n        }}\n    }}\n    count\n}}\n"
        )
    }
}

// ---------------------------------------------------------------- rendering

/// `failed = "<text>"` when `cond` does not hold and nothing failed before.
fn first_failure(cond: &str, text: &str) -> String {
    format!(
        "    if failed == \"\" {{\n        if {cond} {{\n            ()\n        }} else {{\n            \
         failed = {}.to_string()\n        }}\n    }}\n",
        ruchy_string(text)
    )
}

fn render_expectations(expects: &[(String, String)]) -> String {
    let mut out = String::from("\n// Expectations (RHL-5): guards on the decided plan (§9.4).\n");
    for (n, (text, cond)) in expects.iter().enumerate() {
        out.push_str(&format!(
            "// expect_{n}: {text}\nfun expect_{n}(plan: &Vec<Action>) -> bool {{\n    {cond}\n}}\n\n"
        ));
    }
    out.push_str("fun failed_expectation(plan: &Vec<Action>) -> String {\n");
    out.push_str("    let mut failed: String = String::new()\n");
    for (n, (text, _)) in expects.iter().enumerate() {
        out.push_str(&first_failure(&format!("expect_{n}(plan)"), text));
    }
    out.push_str("    failed\n}\n");
    out
}

fn render_examples(examples: &[Example]) -> String {
    let mut out = String::from(
        "\n// Examples (RHL-5): each decides over the facts its `given` lines state.\n",
    );
    for (n, e) in examples.iter().enumerate() {
        out.push_str(&format!(
            "// example_{n}: {}\nfun example_{n}() -> String {{\n    \
             let plan: Vec<Action> = decide({})\n    let mut failed: String = String::new()\n",
            text_literal(&e.name),
            e.facts
        ));
        for (text, cond) in &e.thens {
            out.push_str(&first_failure(cond, text));
        }
        out.push_str(
            "    if failed == \"\" {\n        failed = failed_expectation(&plan)\n    }\n    failed\n}\n\n",
        );
    }
    out.push_str(&runner_main(examples.len()));
    out
}

/// The test runner: one `rhl-example <n> ok|FAILED <what>` line per example,
/// exit 1 when any failed. `ruchy test` reads these lines.
fn runner_main(count: usize) -> String {
    let mut out = String::from("fun main() {\n    let mut failures: i64 = 0\n");
    for n in 0..count {
        out.push_str(&format!(
            "    let result_{n}: String = example_{n}()\n    if result_{n} == \"\" {{\n        \
             println!(\"rhl-example {n} ok\")\n    }} else {{\n        \
             println!(\"rhl-example {n} FAILED {{}}\", result_{n})\n        \
             failures = failures + 1\n    }}\n"
        ));
    }
    out.push_str("    if failures > 0 {\n        std::process::exit(1)\n    }\n    ()\n}\n");
    out
}

// ---------------------------------------------------------------- helpers

/// The RHL text of a statement, for the messages a test prints.
fn span_text(source: &str, span: Span) -> String {
    source
        .get(span.start..span.end)
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// A ruchy string literal for any text, `"` included.
fn ruchy_string(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
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

/// `expect` and `example` belong to the job itself, not to a block of it.
fn refuse_nested(body: &[Stmt], depth: usize) -> Result<(), Refusal> {
    body.iter().try_for_each(|s| refuse_nested_stmt(s, depth))
}

fn refuse_nested_stmt(s: &Stmt, depth: usize) -> Result<(), Refusal> {
    if depth > 0 && matches!(s.kind, StmtKind::Expect(_) | StmtKind::Example { .. }) {
        let m = "an `expect` or `example` inside a block is not lowered: it belongs to the job";
        return Err((s.span, m.to_string()));
    }
    child_bodies(&s.kind)
        .into_iter()
        .try_for_each(|child| refuse_nested(child, depth + 1))
}

fn add_given(
    set: &mut Vec<(String, String)>,
    (field, value): (String, String),
    span: Span,
) -> Result<(), Refusal> {
    if set.iter().any(|(f, _)| *f == field) {
        let m = "this `given` sets a fact the example already set: a fact is given once, not twice"
            .to_string();
        return Err((span, m));
    }
    set.push((field, value));
    Ok(())
}

pub(super) fn not_a_plan_measure(name: &str, span: Span) -> Refusal {
    let names: Vec<String> = PLAN_MEASURES
        .iter()
        .map(|(t, _)| format!("`{t}`"))
        .collect();
    let m = format!(
        "`{name}` is not a plan measure: an `expect` or a `then` reads only the decided plan, \
         through a plan measure ({})",
        names.join(", ")
    );
    (span, m)
}

fn only_given_then() -> String {
    "only `given` and `then` lines are lowered in an `example`".to_string()
}

fn only_given_is() -> String {
    "only `given <fact> is <literal>` is lowered: the fact is a measure applied to literals, \
     or a `let` bound directly to one"
        .to_string()
}

#[cfg(test)]
#[path = "examples_tests.rs"]
mod examples_tests;
