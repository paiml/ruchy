//! `ruchy vocab list | show | validate` (spec RHL-001 §5; roadmap row RHL-2).
//!
//! Every subcommand reads vocabularies through the checker's own loader
//! ([`vocab::load`], [`vocab::parse_file`], [`vocab::shape_violations`],
//! [`vocab::missing_contracts`]), so `vocab validate` and `ruchy check` can
//! never disagree about whether a vocabulary is well formed. The shape is
//! stated in `contracts/rhl-vocabulary-shape-v1.yaml`.

use super::check::Verdict;
use super::cli::{json, Outcome, OutputFormat};
use super::codes;
use super::vocab::{self, Term, Vocabulary};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// The vocabulary root: `explicit` (`--root`), else the nearest ancestor of
/// `cwd`, `cwd` included, that holds a `vocab/` directory (the checker's
/// [`vocab::find_root`]).
#[must_use]
pub fn resolve_root(explicit: Option<&Path>, cwd: &Path) -> Option<PathBuf> {
    match explicit {
        Some(root) => Some(root.to_path_buf()),
        None => vocab::find_root(&cwd.join("_")),
    }
}

/// One row of `vocab list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Listed {
    /// The name, for example `fleet`.
    pub name: String,
    /// The version number.
    pub version: u32,
    /// How many terms it has (0 when it does not load).
    pub terms: usize,
    /// The file, relative to the root.
    pub file: String,
    /// Why the loader refused it, if it did.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Every `vocab/<name>-v<N>.yaml` under `root`, by name then version.
#[must_use]
pub fn list(root: &Path) -> Vec<Listed> {
    let mut rows: Vec<Listed> = std::fs::read_dir(root.join("vocab"))
        .map(|dir| dir.filter_map(Result::ok).collect::<Vec<_>>())
        .unwrap_or_default()
        .iter()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|f| file_reference(&f))
        .map(|(name, version)| listed(root, name, version))
        .collect();
    rows.sort_by(|a, b| (&a.name, a.version).cmp(&(&b.name, b.version)));
    rows
}

/// `fleet-v2.yaml` → `("fleet", 2)`.
fn file_reference(file: &str) -> Option<(String, u32)> {
    let stem = file.strip_suffix(".yaml")?;
    let (name, version) = stem.rsplit_once("-v")?;
    Some((name.to_string(), version.parse().ok()?))
}

fn listed(root: &Path, name: String, version: u32) -> Listed {
    let file = format!("vocab/{name}-v{version}.yaml");
    let loaded = vocab::load(root, &name, version);
    Listed {
        terms: loaded.as_ref().map_or(0, |v| v.terms.len()),
        error: loaded.err(),
        name,
        version,
        file,
    }
}

/// `ruchy vocab list`: exit 0, or 1 when a vocabulary file does not load.
#[must_use]
pub fn list_outcome(root: &Path, format: OutputFormat) -> Outcome {
    let rows = list(root);
    let exit = i32::from(rows.iter().any(|r| r.error.is_some()));
    let stdout = match format {
        OutputFormat::Json => json(&rows),
        OutputFormat::Text => rows.iter().map(list_line).collect(),
    };
    Outcome {
        stdout,
        stderr: String::new(),
        exit,
    }
}

fn list_line(r: &Listed) -> String {
    match &r.error {
        None => format!("{} v{}  {} terms  {}\n", r.name, r.version, r.terms, r.file),
        Some(e) => format!("{} v{}  does not load: {e}\n", r.name, r.version),
    }
}

/// `v2` or `2` → 2.
#[must_use]
pub fn parse_version(text: &str) -> Option<u32> {
    text.strip_prefix('v').unwrap_or(text).parse().ok()
}

/// `ruchy vocab show <name> <version>`: every term with its kind, `takes`,
/// `gives`, effect, attributes and contract. A vocabulary the loader refuses
/// is RHL-V004 on standard error, exit 2.
#[must_use]
pub fn show_outcome(root: &Path, name: &str, version: &str, format: OutputFormat) -> Outcome {
    let loaded = parse_version(version)
        .ok_or_else(|| format!("`{version}` is not a version: write `v2` or `2`"))
        .and_then(|n| vocab::load(root, name, n));
    match loaded {
        Ok(v) => Outcome {
            stdout: render_show(&v, format),
            stderr: String::new(),
            exit: 0,
        },
        Err(e) => Outcome {
            stdout: String::new(),
            stderr: format!(
                "error[{}]: unknown vocabulary `{name} {version}`: {e}\n",
                codes::V004
            ),
            exit: 2,
        },
    }
}

fn render_show(v: &Vocabulary, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => json(v),
        OutputFormat::Text => {
            let head = format!("{} ({} terms)\n", v.label(), v.terms.len());
            head + &v.terms.iter().map(term_line).collect::<String>()
        }
    }
}

fn term_line(t: &Term) -> String {
    let params = |ps: &[vocab::Param]| {
        ps.iter()
            .map(|p| format!("{}: {}", p.name, p.ty))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut parts = vec![
        format!("  {}", t.term),
        format!("{:?}", t.kind).to_lowercase(),
    ];
    let optional = [
        ("takes", Some(params(&t.takes)).filter(|s| !s.is_empty())),
        ("gives", t.gives.clone()),
        ("effect", t.effect.clone()),
        (
            "attributes",
            Some(params(&t.attributes)).filter(|s| !s.is_empty()),
        ),
        ("instances from", t.instances_from.clone()),
        ("contract", Some(t.contract.clone())),
    ];
    parts.extend(
        optional
            .into_iter()
            .filter_map(|(k, v)| v.map(|v| format!("{k} {v}"))),
    );
    parts.join("  ") + "\n"
}

/// One finding of `vocab validate`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    /// RHL-V004 (the loader refuses the file) or RHL-C001 (a term with no
    /// contract template).
    pub code: String,
    /// The term, when the finding is about one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub term: Option<String>,
    /// A one-line message.
    pub message: String,
}

/// The result of `vocab validate`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Validation {
    /// The file validated.
    pub file: String,
    /// `fleet v2`, when the file deserialized.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocabulary: Option<String>,
    /// Pass, fail or refused, as for `ruchy check`.
    pub verdict: Verdict,
    /// Every finding.
    pub diagnostics: Vec<Finding>,
}

impl Validation {
    /// 0 pass, 1 error, 2 refusal.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self.verdict {
            Verdict::Pass => 0,
            Verdict::Fail => 1,
            Verdict::Refused => 2,
        }
    }
}

/// Validate the vocabulary file `path`, whose contract paths resolve under `root`.
#[must_use]
pub fn validate(path: &Path, root: &Path) -> Validation {
    let (vocabulary, diagnostics) = match vocab::parse_file(path) {
        Err(e) => (None, vec![finding(codes::V004, None, e)]),
        Ok(v) => (Some(v.label()), findings(&v, root)),
    };
    let verdict = verdict_of(&diagnostics);
    Validation {
        file: path.display().to_string(),
        vocabulary,
        verdict,
        diagnostics,
    }
}

/// Shape violations (RHL-V004: the loader refuses the file) then missing
/// contract templates (RHL-C001, §3.4).
fn findings(v: &Vocabulary, root: &Path) -> Vec<Finding> {
    let shape = vocab::shape_violations(v).into_iter().map(|m| {
        finding(
            codes::V004,
            None,
            format!("breaks the vocabulary shape: {m}"),
        )
    });
    let contracts = vocab::missing_contracts(root, v).into_iter().map(|t| {
        let message = format!(
            "term `{}` has no contract template `{}`",
            t.term, t.contract
        );
        finding(codes::C001, Some(t.term.clone()), message)
    });
    shape.chain(contracts).collect()
}

fn finding(code: &str, term: Option<String>, message: String) -> Finding {
    Finding {
        code: code.to_string(),
        term,
        message,
    }
}

fn verdict_of(findings: &[Finding]) -> Verdict {
    let refused = findings
        .iter()
        .any(|f| codes::lookup(&f.code).is_some_and(|c| c.refusal.is_some()));
    match (refused, findings.is_empty()) {
        (true, _) => Verdict::Refused,
        (false, true) => Verdict::Pass,
        (false, false) => Verdict::Fail,
    }
}

/// `ruchy vocab validate <file.yaml>` or `<name> <version>`.
#[must_use]
pub fn validate_outcome(root: Option<&Path>, target: &[String], format: OutputFormat) -> Outcome {
    let resolved = match target {
        [file] => Ok(PathBuf::from(file)),
        [name, version] => parse_version(version)
            .ok_or_else(|| format!("`{version}` is not a version: write `v2` or `2`"))
            .and_then(|n| {
                root.map(|r| vocab::vocab_path(r, name, n))
                    .ok_or_else(no_root)
            }),
        _ => Err("vocab validate takes <file.yaml> or <name> <version>".to_string()),
    };
    let path = match resolved {
        Ok(p) => p,
        Err(e) => return usage(e),
    };
    let Some(root) = root
        .map(Path::to_path_buf)
        .or_else(|| vocab::find_root(&path))
    else {
        return usage(no_root());
    };
    let v = validate(&path, &root);
    Outcome {
        stdout: render_validation(&v, format),
        stderr: String::new(),
        exit: v.exit_code(),
    }
}

/// The message for a vocabulary command run outside any vocabulary root.
#[must_use]
pub fn no_root() -> String {
    "no `vocab/` directory in this directory or any parent; pass --root".to_string()
}

fn usage(message: String) -> Outcome {
    Outcome {
        stdout: String::new(),
        stderr: message + "\n",
        exit: 1,
    }
}

fn render_validation(v: &Validation, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => json(v),
        OutputFormat::Text => {
            let mut out: String = v
                .diagnostics
                .iter()
                .map(|f| format!("{}: error[{}]: {}\n", v.file, f.code, f.message))
                .collect();
            out.push_str(&format!("{}: {}\n", v.file, v.verdict.word()));
            out
        }
    }
}

#[cfg(test)]
#[path = "vocab_cli_tests.rs"]
mod vocab_cli_tests;
