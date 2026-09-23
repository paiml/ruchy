//! The YAML surface `.rhl.yaml` (spec RHL-001 §2, row RHL-3): a lossless
//! serialization of the intent tree, the same tree [`super::parse`] makes.
//!
//! The YAML is data. Reading it builds a [`Program`] and nothing else: no
//! tag, alias or key runs anything, a local tag or an unknown key is refused
//! rather than dropped, and a tree the RHL grammar could not have produced is refused
//! too (spec §9 rule 5: one tree, one checker; a surface adds no meaning).
//! The shape is documented in `docs/rhl/rhl-3.md`.
//!
//! "Could have produced" is decided by the grammar itself, not by a second
//! list of rules: a tree is accepted when its RHL normal form parses back to
//! the same tree. That one test covers an empty or keyword word, a quote in a
//! string, a leading zero, and an `or` under an `and` (RHL has no
//! parentheses), and it is exactly the property F3 needs.

use super::codes;
use super::diag::{self, Diagnostic, LineSpan};
use super::fmt::format;
use super::tree::{Decl, Program, Span};
use serde::{Deserialize, Serialize};
use serde_yaml_ng::with::singleton_map_recursive;
use serde_yaml_ng::Value;
use std::path::Path;

/// The schema version a `.rhl.yaml` file declares in its `rhl:` key.
pub const SCHEMA: u32 = 1;

/// The file suffix of the YAML surface.
pub const SUFFIX: &str = ".rhl.yaml";

/// Why a YAML text is not a program tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlError {
    /// What is wrong.
    pub message: String,
    /// 1-based line in the YAML, or 0 when the problem has no one place.
    pub line: usize,
    /// 1-based column in the YAML, or 0 when the problem has no one place.
    pub col: usize,
}

impl YamlError {
    /// An error about the whole tree rather than one place in the text.
    fn unplaced(message: String) -> Self {
        Self {
            message,
            line: 0,
            col: 0,
        }
    }

    fn from_serde(e: &serde_yaml_ng::Error) -> Self {
        let (line, col) = e.location().map_or((0, 0), |l| (l.line(), l.column()));
        Self {
            message: format!("not an RHL tree: {e}"),
            line,
            col,
        }
    }
}

/// The document as written: the schema marker, then the tree's own field.
#[derive(Serialize)]
struct DocOut<'a> {
    rhl: u32,
    #[serde(with = "singleton_map_recursive")]
    decls: &'a Vec<Decl>,
}

/// The document as read. Enums are one-key maps (`use: …`, `when: …`),
/// never YAML tags, so the file stays in the vocabulary files' schema family.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DocIn {
    rhl: u32,
    #[serde(with = "singleton_map_recursive")]
    decls: Vec<Decl>,
}

/// True for a path the YAML surface owns: a file name ending `.rhl.yaml`.
#[must_use]
pub fn is_rhl_yaml(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.ends_with(SUFFIX))
}

/// Serialize `program` as canonical `.rhl.yaml`. Total: every tree prints.
#[must_use]
pub fn to_yaml(program: &Program) -> String {
    let doc = DocOut {
        rhl: SCHEMA,
        decls: &program.decls,
    };
    serde_yaml_ng::to_string(&doc)
        .unwrap_or_else(|e| format!("# RHL-3: the tree does not serialize: {e}\n"))
}

/// Read a `.rhl.yaml` text as a program tree.
///
/// # Errors
///
/// Returns a [`YamlError`] when the text is not YAML, does not have the
/// tree's shape (including an unknown key or a missing `rhl:` marker),
/// declares another schema version, or holds a tree the RHL grammar could
/// not have produced.
pub fn from_yaml(source: &str) -> Result<Program, YamlError> {
    let doc: DocIn = serde_yaml_ng::from_str(source).map_err(|e| YamlError::from_serde(&e))?;
    untagged(source)?;
    if doc.rhl != SCHEMA {
        return Err(YamlError::unplaced(format!(
            "schema `rhl: {}` is not supported; this ruchy reads `rhl: {SCHEMA}`",
            doc.rhl
        )));
    }
    let program = Program { decls: doc.decls };
    grammatical(&program)?;
    Ok(program)
}

/// Refuse a local YAML tag (`!x`) anywhere. The shape has none, and serde
/// would otherwise read a tagged scalar as its plain value, dropping the tag
/// without a word. A `!!` tag is resolved by the YAML library itself (to a
/// core-schema scalar), before any value is built, so it is not visible here.
fn untagged(source: &str) -> Result<(), YamlError> {
    let value: Value = serde_yaml_ng::from_str(source).map_err(|e| YamlError::from_serde(&e))?;
    match find_tag(&value) {
        Some(tag) => Err(YamlError::unplaced(format!(
            "not an RHL tree: YAML tags are not part of the shape, found `{tag}`"
        ))),
        None => Ok(()),
    }
}

fn find_tag(value: &Value) -> Option<String> {
    match value {
        Value::Tagged(t) => Some(t.tag.to_string()),
        Value::Sequence(items) => items.iter().find_map(find_tag),
        Value::Mapping(map) => map
            .iter()
            .find_map(|(k, v)| find_tag(k).or_else(|| find_tag(v))),
        _ => None,
    }
}

/// Refuse a tree the grammar could not produce: its normal form must parse
/// back to the same tree.
fn grammatical(program: &Program) -> Result<(), YamlError> {
    let text = format(program);
    let back = super::parse(&text).map_err(|f| {
        let d = diag::from_parse_failure("", &text, &f);
        YamlError::unplaced(format!(
            "not a tree the RHL grammar produces: its RHL form fails at line {}: {} ({})",
            d.span.line, d.message, d.code
        ))
    })?;
    if to_yaml(&back) == to_yaml(program) {
        Ok(())
    } else {
        Err(YamlError::unplaced(
            "not a tree the RHL grammar produces: its RHL form reads back as a different tree"
                .to_string(),
        ))
    }
}

/// `ruchy fmt` on a `.rhl.yaml` text: the canonical YAML of its tree.
///
/// # Errors
///
/// Returns the [`YamlError`] of [`from_yaml`].
pub fn format_yaml(source: &str) -> Result<String, YamlError> {
    from_yaml(source).map(|p| to_yaml(&p))
}

/// The §3.5 diagnostic for a YAML text that is not a tree: `RHL-P001`, at the
/// YAML's line and column (line 0 when the problem has no one place).
#[must_use]
pub fn diagnostic(file: &str, e: &YamlError) -> Diagnostic {
    let mut d = Diagnostic::error(codes::P001, file, "", Span::default(), e.message.clone());
    d.span = LineSpan {
        line: e.line,
        col: e.col,
        len: 0,
    };
    d
}

#[cfg(test)]
#[path = "yaml_tests.rs"]
mod yaml_tests;
