//! RHL-4b: the ruchy runtime a vocabulary term's `lowers_to` names (spec
//! RHL-001 §4, Amendment A4). A binding `<module>::<name>` names the function
//! `fun <name>` in `<root>/vocab/runtime/<module>.ruchy`; lowering copies that
//! one function into the generated source. `[U]` is the unbound marker: no
//! local source of truth exists, and a job that needs one is refused by name.

use super::tree::Span;
use std::path::Path;

/// The unbound marker of a vocabulary field.
pub const UNBOUND: &str = "[U]";

/// Split a binding into `(module, function)`: both lowercase ASCII words, so
/// a binding can name no path outside `vocab/runtime/`.
#[must_use]
pub fn parse_binding(binding: &str) -> Option<(&str, &str)> {
    let (module, name) = binding.split_once("::")?;
    let word = |w: &str| {
        !w.is_empty()
            && w.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    };
    (word(module) && word(name)).then_some((module, name))
}

/// The source of the function `binding` names, from `<root>/vocab/runtime/`:
/// from its `fun <name>(` line through the first line that is exactly `}`.
///
/// # Errors
///
/// A message naming the binding when it is `[U]`, malformed, its runtime file
/// cannot be read, or the file defines no such function.
pub fn binding_function(root: &Path, binding: &str) -> Result<String, String> {
    if binding == UNBOUND {
        return Err("the binding is `[U]`: no local source of truth".to_string());
    }
    let (module, name) =
        parse_binding(binding).ok_or_else(|| format!("`{binding}` is not `<module>::<name>`"))?;
    let path = root.join("vocab/runtime").join(format!("{module}.ruchy"));
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("`{binding}`: cannot read {}: {e}", path.display()))?;
    extract(&text, name)
        .ok_or_else(|| format!("`{binding}`: {} has no `fun {name}`", path.display()))
}

fn extract(text: &str, name: &str) -> Option<String> {
    let head = format!("fun {name}(");
    let mut lines = text.lines().skip_while(|l| !l.starts_with(&head));
    let first = lines.next()?;
    let mut out = format!("{first}\n");
    for line in lines {
        out.push_str(line);
        out.push('\n');
        if line == "}" {
            return Some(out);
        }
    }
    None
}

/// A measure a job reads: the `Facts` field it fills, and the call that
/// fills it (the binding applied to the literal arguments' ruchy forms).
pub(crate) struct MeasureUse {
    pub field: String,
    pub term: String,
    pub binding: String,
    pub args: Vec<String>,
    pub span: Span,
}

/// An action a job may perform: its `Action` variant, its fields in order
/// (`true` for a Text field, passed as `&str`), and its binding.
pub(crate) struct ActionUse {
    pub variant: String,
    pub term: String,
    pub binding: String,
    pub fields: Vec<(String, bool)>,
    pub span: Span,
}

/// What a lowered job uses, in first-use order.
pub(crate) struct Uses {
    pub measures: Vec<MeasureUse>,
    pub actions: Vec<ActionUse>,
}

/// The `main` of every built job: `observe`, `decide`, one `{:?}` line per
/// planned action on standard output, and `apply` ONLY under `--apply`
/// (§9.4: the plan is a draft, never an action). A plan that breaks an
/// `expect` (`failed_expectation`, RHL-5) is never applied: exit 1 naming it.
/// Exit 1 also when `--apply` performed fewer actions than planned. It ends
/// in `()` because ruchy wraps a trailing call of `main` in `println!`; it
/// reads the arguments with a loop because `.collect()` makes ruchy emit
/// polars.
const MAIN: &str = "fun main() {
    let mut apply_flag: bool = false
    for arg in std::env::args() {
        if arg == \"--apply\" {
            apply_flag = true
        }
    }
    let plan: Vec<Action> = decide(observe())
    let total: i64 = plan.len() as i64
    for action in plan.iter() {
        println!(\"{:?}\", action)
    }
    let failed: String = failed_expectation(&plan)
    if failed != \"\" {
        eprintln!(\"expectation failed: {}; nothing applied\", failed)
        std::process::exit(1)
    }
    let mut performed: i64 = 0
    if apply_flag {
        performed = apply(plan)
    }
    if apply_flag && performed < total {
        eprintln!(\"applied {} of {} planned action(s)\", performed, total)
        std::process::exit(1)
    }
    ()
}
";

/// Everything a built job needs after `decide`: the runtime functions its
/// terms are bound to (each once), `observe`, `apply` and `main`.
///
/// # Errors
///
/// `(span, message)` naming the first term, at its first use, whose binding
/// is `[U]` or does not resolve under `root` (`RHL-L002`).
pub(crate) fn executable(root: &Path, uses: &Uses) -> Result<String, (Span, String)> {
    let wanted = uses
        .measures
        .iter()
        .map(|m| (&m.term, &m.binding, m.span))
        .chain(uses.actions.iter().map(|a| (&a.term, &a.binding, a.span)));
    let mut seen: Vec<&str> = Vec::new();
    let mut out =
        String::from("\n// Runtime bindings (vocab/runtime) of the terms this job uses.\n");
    for (term, binding, span) in wanted {
        let f = binding_function(root, binding).map_err(|e| (span, unbound(term, &e)))?;
        if !seen.contains(&binding.as_str()) {
            seen.push(binding);
            out.push_str(&format!("\n{f}"));
        }
    }
    Ok(format!("{out}\n{}\n{}\n{MAIN}", observe(uses), apply(uses)))
}

fn unbound(term: &str, why: &str) -> String {
    format!(
        "`{term}` has no runtime binding ({why}), so a job that uses it cannot be compiled or \
         run; `ruchy transpile` still gives its `decide`"
    )
}

fn observe(uses: &Uses) -> String {
    let fields: Vec<String> = uses
        .measures
        .iter()
        .map(|m| {
            format!(
                "{}: {}({})",
                m.field,
                call_name(&m.binding),
                m.args.join(", ")
            )
        })
        .collect();
    format!(
        "fun observe() -> Facts {{\n    Facts {{ {} }}\n}}\n",
        fields.join(", ")
    )
}

fn apply(uses: &Uses) -> String {
    if uses.actions.is_empty() {
        return "fun apply(plan: Vec<Action>) -> i64 {\n    0\n}\n".to_string();
    }
    let arms: String = uses.actions.iter().map(apply_arm).collect();
    format!(
        "fun apply(plan: Vec<Action>) -> i64 {{\n    let mut performed: i64 = 0\n    \
         for action in plan {{\n        let done: bool = match action {{\n{arms}        }}\n        \
         if done {{\n            performed = performed + 1\n        }}\n    }}\n    performed\n}}\n"
    )
}

fn apply_arm(a: &ActionUse) -> String {
    let names: Vec<&str> = a.fields.iter().map(|(n, _)| n.as_str()).collect();
    let args: Vec<String> = a
        .fields
        .iter()
        .map(|(n, text)| if *text { format!("&{n}") } else { n.clone() })
        .collect();
    let pattern = if names.is_empty() {
        format!("Action::{}", a.variant)
    } else {
        format!("Action::{} {{ {} }}", a.variant, names.join(", "))
    };
    format!(
        "            {pattern} => {}({}),\n",
        call_name(&a.binding),
        args.join(", ")
    )
}

/// The function a (resolved) binding names.
fn call_name(binding: &str) -> &str {
    parse_binding(binding).map_or(binding, |(_, name)| name)
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod runtime_tests;
