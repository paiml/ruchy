//! The RHL tools of `ruchy mcp` (spec RHL-001 §6; row RHL-6).
//!
//! Every tool takes a JSON object and returns a JSON value — a report, a tree
//! rendering, a list of terms — never prose, and never acts on the world
//! (§9.4): `rhl_fix` returns the fixed text instead of writing a file, and
//! `rhl_transpile` returns code instead of building it. Each tool calls the
//! library function its CLI verb calls, so the tool and the verb cannot
//! disagree. Shapes are listed in `docs/rhl/rhl-6.md`.
//!
//! A program the tools cannot take is data too: a source that does not parse
//! or check comes back as `{"report": <the check report>}`. Only a malformed
//! request (no `source`, an unknown `to` or `emit`, no vocabulary root) is an
//! error.

use super::RuchyMCPTool;
use crate::rhl::cli::{
    check_source, convert_source, format_file_source, lower_named, Emit, Target,
};
use crate::rhl::diag::Diagnostic;
use crate::rhl::explain::{explain, tree_of};
use crate::rhl::fix::fix_safe;
use crate::rhl::lower::{to_rust, Form, LowerFailure};
use crate::rhl::vocab::{self, Term, Vocabulary};
use crate::rhl::vocab_cli::{list, resolve_root};
use crate::rhl::yaml::is_rhl_yaml;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// The eight tools of §6, in the spec's order.
pub const RHL_TOOL_NAMES: [&str; 8] = [
    "rhl_check",
    "rhl_fix",
    "rhl_format",
    "rhl_convert",
    "rhl_transpile",
    "rhl_explain",
    "rhl_vocabulary",
    "rhl_grammar",
];

/// The file name diagnostics carry when a request names none.
const DEFAULT_FILE: &str = "input.rhl";

type Handler = fn(&Value) -> Result<Value>;

const TOOLS: [(&str, &str, Handler); 8] = [
    ("rhl_check", "Check an RHL program; returns the §3.5 report (ruchy check --format json)", rhl_check),
    ("rhl_fix", "Apply the safe fixes to an RHL program; returns {fixed_source, applied, report}, writes nothing", rhl_fix),
    ("rhl_format", "Normal form of an RHL program; returns {source}, or {report} when it does not parse", rhl_format),
    ("rhl_convert", "Convert RHL text and .rhl.yaml losslessly; returns {output, to}", rhl_convert),
    ("rhl_transpile", "Lower an RHL program to ruchy or Rust; returns {ruchy_source} or {rust_source}, or {report}", rhl_transpile),
    ("rhl_explain", "Deterministic plain-language rendering of an RHL program's tree; returns {text}", rhl_explain),
    ("rhl_vocabulary", "Search vocabulary terms by kind, gives type and spelling; returns a list of terms", rhl_vocabulary),
    ("rhl_grammar", "The RHL grammar the parser is generated from; returns {grammar}", rhl_grammar),
];

/// The eight tools, ready to register on a server.
#[must_use]
pub fn create_rhl_tools() -> Vec<(&'static str, RuchyMCPTool)> {
    TOOLS
        .iter()
        .map(|&(name, description, handler)| {
            let tool = RuchyMCPTool::new(name.to_string(), description.to_string(), move |args| {
                handler(&args)
            })
            .with_input_schema(input_schema(name));
            (name, tool)
        })
        .collect()
}

/// The JSON Schema of a tool's arguments.
fn input_schema(name: &str) -> Value {
    let properties = match name {
        "rhl_vocabulary" => json!({
            "kind": {"type": "string", "enum": ["noun", "measure", "action", "unit", "entity"]},
            "gives": {"type": "string"},
            "query": {"type": "string"},
            "root": {"type": "string"},
        }),
        "rhl_grammar" => json!({}),
        _ => json!({
            "source": {"type": "string"},
            "file_name": {"type": "string"},
            "root": {"type": "string"},
            "to": {"type": "string", "enum": ["rhl", "yaml"]},
            "emit": {"type": "string", "enum": ["ruchy", "rust"]},
        }),
    };
    let required = if properties.get("source").is_some() {
        json!(["source"])
    } else {
        json!([])
    };
    json!({"type": "object", "properties": properties, "required": required})
}

// ───────────────────────── arguments ─────────────────────────

fn source(args: &Value) -> Result<&str> {
    args.get("source")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing string argument `source`"))
}

fn file_name(args: &Value) -> &str {
    args.get("file_name")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_FILE)
}

/// The vocabulary root: the `root` argument, else the nearest ancestor of the
/// server's working directory that holds `vocab/` (what the CLI verbs find).
fn root(args: &Value) -> Option<PathBuf> {
    let explicit = args.get("root").and_then(Value::as_str).map(Path::new);
    let cwd = std::env::current_dir().unwrap_or_default();
    resolve_root(explicit, &cwd)
}

/// `{"report": …}`: the check report of a source a tool cannot take.
fn report_of(file: &str, source: &str, args: &Value) -> Result<Value> {
    let report = check_source(file, source, root(args).as_deref());
    Ok(json!({ "report": report }))
}

// ───────────────────────── the tools ─────────────────────────

/// `rhl_check {source, file_name?, root?}` → the §3.5 report.
///
/// # Errors
///
/// Returns an error when `source` is missing.
pub fn rhl_check(args: &Value) -> Result<Value> {
    let report = check_source(file_name(args), source(args)?, root(args).as_deref());
    Ok(serde_json::to_value(report)?)
}

/// `rhl_fix {source, file_name?, root?}` → `{fixed_source, applied, reverted,
/// rounds, report}`, where `report` checks the fixed source.
///
/// # Errors
///
/// Returns an error when `source` is missing or `file_name` is `.rhl.yaml`
/// (fixes are edits of RHL text).
pub fn rhl_fix(args: &Value) -> Result<Value> {
    let file = file_name(args);
    if is_rhl_yaml(Path::new(file)) {
        return Err(anyhow!(
            "{file}: rhl_fix edits RHL text; rhl_convert it to rhl first"
        ));
    }
    let fixed = fix_safe(file, source(args)?, root(args).as_deref());
    Ok(json!({
        "fixed_source": fixed.source,
        "applied": fixed.report.applied,
        "reverted": fixed.report.reverted,
        "rounds": fixed.report.rounds,
        "report": fixed.report.report,
    }))
}

/// `rhl_format {source, file_name?}` → `{source}` in normal form, or `{report}`.
///
/// # Errors
///
/// Returns an error when `source` is missing.
pub fn rhl_format(args: &Value) -> Result<Value> {
    let (file, text) = (file_name(args), source(args)?);
    match format_file_source(file, text) {
        Ok(formatted) => Ok(json!({ "source": formatted })),
        Err(_) => report_of(file, text, args),
    }
}

/// `rhl_convert {source, to?, file_name?}` → `{output, to}`, or `{report}`.
/// Without `to`, the direction is the other surface than `file_name`'s.
///
/// # Errors
///
/// Returns an error when `source` is missing or `to` is not `rhl` or `yaml`.
pub fn rhl_convert(args: &Value) -> Result<Value> {
    let target = convert_target(args)?;
    let text = source(args)?;
    let file = input_name(file_name(args), target);
    match convert_source(&file, text, target) {
        Ok(output) => Ok(json!({ "output": output, "to": target_word(target) })),
        Err(_) => report_of(&file, text, args),
    }
}

fn convert_target(args: &Value) -> Result<Target> {
    match args.get("to").and_then(Value::as_str) {
        Some(to) => Target::parse(to).map_err(|m| anyhow!(m.trim_end().to_string())),
        None => Target::for_input(Path::new(file_name(args)))
            .ok_or_else(|| anyhow!("pass `to`: rhl or yaml")),
    }
}

/// The name of the surface `target` converts from, so a refusal is checked
/// by the right checker.
fn input_name(file: &str, target: Target) -> String {
    let yaml = is_rhl_yaml(Path::new(file));
    match (target, yaml) {
        (Target::Rhl, false) => "input.rhl.yaml".to_string(),
        (Target::Yaml, true) => DEFAULT_FILE.to_string(),
        _ => file.to_string(),
    }
}

fn target_word(target: Target) -> &'static str {
    match target {
        Target::Rhl => "rhl",
        Target::Yaml => "yaml",
    }
}

/// `rhl_transpile {source, emit?, file_name?, root?}` → `{ruchy_source}` for
/// `emit: "ruchy"`, `{rust_source}` for `emit: "rust"` (the default); a
/// program that does not check, or that the lowering refuses, is `{report}`.
///
/// # Errors
///
/// Returns an error when `source` is missing or `emit` is not `ruchy` or `rust`.
pub fn rhl_transpile(args: &Value) -> Result<Value> {
    let emit = match Emit::parse(args.get("emit").and_then(Value::as_str)) {
        Ok(Emit::Contract) => return Err(anyhow!("rhl_transpile emits ruchy or rust")),
        Ok(e) => e,
        Err(m) => return Err(anyhow!(m.trim_end().to_string())),
    };
    let file = file_name(args);
    match lower_named(Form::Decide, file, source(args)?, root(args).as_deref()) {
        Ok(ruchy) if emit == Emit::Ruchy => Ok(json!({ "ruchy_source": ruchy })),
        Ok(ruchy) => Ok(rust_of(&ruchy)),
        Err(failure) => Ok(json!({ "report": failure_report(file, failure) })),
    }
}

fn rust_of(ruchy: &str) -> Value {
    match to_rust(ruchy) {
        Ok(rust) => json!({ "rust_source": rust }),
        Err(message) => json!({ "error": {"stage": "rust", "message": message} }),
    }
}

/// A lowering failure in the report shape: the check's own report, or a
/// refused report holding the `RHL-L…` diagnostic.
fn failure_report(file: &str, failure: LowerFailure) -> Value {
    match failure {
        LowerFailure::Check(report) => json!(report),
        LowerFailure::Refused(d) => refused(file, d),
    }
}

fn refused(file: &str, d: Diagnostic) -> Value {
    json!({"file": file, "verdict": "refused", "diagnostics": [d], "unverified": []})
}

/// `rhl_explain {source, file_name?}` → `{text}`, the deterministic
/// plain-language rendering of the tree (`ruchy explain`), or `{report}`.
///
/// # Errors
///
/// Returns an error when `source` is missing.
pub fn rhl_explain(args: &Value) -> Result<Value> {
    let (file, text) = (file_name(args), source(args)?);
    match tree_of(file, text) {
        Ok(program) => Ok(json!({ "text": explain(&program) })),
        Err(_) => report_of(file, text, args),
    }
}

/// `rhl_vocabulary {kind?, gives?, query?, root?}` → every term of every
/// vocabulary under the root that has kind `kind`, gives type `gives` and
/// contains `query` in its spelling, as `{vocabulary, version, term, kind,
/// takes, gives, effect}`, by vocabulary, version, then file order.
///
/// # Errors
///
/// Returns an error when no vocabulary root is found.
pub fn rhl_vocabulary(args: &Value) -> Result<Value> {
    let root = root(args)
        .ok_or_else(|| anyhow!(crate::rhl::vocab_cli::no_root().trim_end().to_string()))?;
    let rows: Vec<Value> = list(&root)
        .iter()
        .filter_map(|l| vocab::load(&root, &l.name, l.version).ok())
        .flat_map(|v| rows_of(&v))
        .filter(|row| wanted(row, args))
        .collect();
    Ok(Value::Array(rows))
}

fn rows_of(v: &Vocabulary) -> Vec<Value> {
    v.terms.iter().map(|t| row(v, t)).collect()
}

fn row(v: &Vocabulary, t: &Term) -> Value {
    json!({
        "vocabulary": v.vocabulary,
        "version": v.version,
        "term": t.term,
        "kind": t.kind,
        "takes": t.takes,
        "gives": t.gives,
        "effect": t.effect,
    })
}

/// True when `row` passes every filter `args` sets.
fn wanted(row: &Value, args: &Value) -> bool {
    let equal = |key: &str| {
        args.get(key)
            .map_or(true, |want| row.get(key) == Some(want))
    };
    let query = args.get("query").and_then(Value::as_str).unwrap_or("");
    let term = row.get("term").and_then(Value::as_str).unwrap_or("");
    equal("kind") && equal("gives") && term.contains(query)
}

/// `rhl_grammar {}` → `{grammar}`: the text of `src/rhl/grammar.lalrpop`, the
/// grammar the parser is generated from.
///
/// # Errors
///
/// Never returns one.
pub fn rhl_grammar(_args: &Value) -> Result<Value> {
    Ok(json!({ "grammar": include_str!("../rhl/grammar.lalrpop") }))
}
