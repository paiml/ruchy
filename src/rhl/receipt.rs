//! RHL-5b: the compile receipt (spec RHL-001 §4: "inputs sha · vocabulary
//! versions · ruchy version · verdict"; "The receipt verdict is three-valued —
//! Pass, Fail, Unknown{reason} — never a fabricated GO").
//!
//! `ruchy compile x.rhl -o out` writes `out.receipt.json`. The verdict is the
//! result of running the job's examples with `ruchy test`'s runner: every
//! example holds → `Pass`; one fails → `Fail`; no examples, or the runner
//! could not build → `Unknown`. The bytes carry no wall-clock value.

use super::cli::{ExampleResult, Outcome};
use super::lower::to_rust;
use super::runtime::parse_binding;
use super::tree::Decl;
use super::vocab::{find_root, load, parse_reference, vocab_path};
use super::yaml::is_rhl_yaml;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Lower-case hex sha256 of `bytes`.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// The three-valued verdict: `"Pass"`, `"Fail"` or `{"Unknown": "<reason>"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Verdict {
    /// Every example ran and held.
    Pass,
    /// At least one example ran and did not hold.
    Fail,
    /// No evidence either way, and why.
    Unknown(String),
}

/// The verdict of running the examples: their results, or why the runner
/// could not produce any. Never `Pass` without an example that held.
#[must_use]
pub fn verdict_of(run: Result<&[ExampleResult], String>) -> Verdict {
    match run {
        Err(reason) => Verdict::Unknown(reason),
        Ok([]) => Verdict::Unknown("no examples".to_string()),
        Ok(results) if results.iter().all(|r| r.ok) => Verdict::Pass,
        Ok(_) => Verdict::Fail,
    }
}

/// One input file by its sha256.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileInput {
    /// The path, relative to the vocabulary root.
    pub path: String,
    /// Its sha256.
    pub sha256: String,
}

/// One vocabulary the job uses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VocabularyInput {
    /// Its name, `fleet`.
    pub name: String,
    /// Its version, `2`.
    pub version: u32,
    /// The sha256 of `vocab/<name>-v<version>.yaml`.
    pub sha256: String,
}

/// Everything the build read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Inputs {
    /// The sha256 of the RHL source bytes.
    pub source_sha256: String,
    /// The vocabularies of its `use vocabulary` lines, in source order.
    pub vocabularies: Vec<VocabularyInput>,
    /// The `vocab/runtime/*.ruchy` files its bindings name, sorted.
    pub runtime_files: Vec<FileInput>,
}

/// The receipt `ruchy compile` writes next to the binary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Receipt {
    /// What the build read.
    pub inputs: Inputs,
    /// The ruchy that built it.
    pub ruchy_version: String,
    /// The sha256 of the generated Rust.
    pub rust_sha256: String,
    /// The sha256 of the unit contract written beside it.
    pub contract_sha256: String,
    /// The examples' verdict.
    pub verdict: Verdict,
}

impl Receipt {
    /// Pretty JSON with a trailing newline: deterministic bytes.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default() + "\n"
    }
}

/// The receipt of compiling `input` to the lowered `ruchy`, with its unit
/// `contract` and the result of running its examples.
///
/// # Errors
///
/// A message when the source cannot be read or the Rust cannot be generated.
pub fn build(
    input: &Path,
    ruchy: &str,
    contract: &str,
    run: Result<Vec<ExampleResult>, Outcome>,
) -> Result<Receipt, String> {
    let source = std::fs::read(input).map_err(|e| format!("cannot read: {e}"))?;
    let root = find_root(input).unwrap_or_else(|| PathBuf::from("."));
    let rust = to_rust(ruchy)?;
    let run = run.map_err(|o| unknown_reason(&o));
    let vocabularies = used_vocabularies(input, &source);
    Ok(Receipt {
        inputs: Inputs {
            source_sha256: sha256_hex(&source),
            runtime_files: runtime_files(&root, &vocabularies),
            vocabularies: vocabularies
                .iter()
                .map(|(name, version)| VocabularyInput {
                    name: name.clone(),
                    version: *version,
                    sha256: sha256_hex(
                        &std::fs::read(vocab_path(&root, name, *version)).unwrap_or_default(),
                    ),
                })
                .collect(),
        },
        ruchy_version: env!("CARGO_PKG_VERSION").to_string(),
        rust_sha256: sha256_hex(rust.as_bytes()),
        contract_sha256: sha256_hex(contract.as_bytes()),
        verdict: verdict_of(run.as_deref().map_err(Clone::clone)),
    })
}

/// Why the runner gave no results: the first line it printed, or its code.
fn unknown_reason(o: &Outcome) -> String {
    let line = o
        .stderr
        .lines()
        .chain(o.stdout.lines())
        .find(|l| !l.trim().is_empty());
    format!(
        "the examples could not be built or run: {}",
        line.map_or_else(|| format!("exit {}", o.exit), str::to_string)
    )
}

/// `(name, version)` of each `use vocabulary` line, in source order.
fn used_vocabularies(input: &Path, source: &[u8]) -> Vec<(String, u32)> {
    let text = String::from_utf8_lossy(source);
    let program = if is_rhl_yaml(input) {
        super::yaml::from_yaml(&text).ok()
    } else {
        super::parse(&text).ok()
    };
    let Some(program) = program else {
        return Vec::new();
    };
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
        .collect()
}

/// The `vocab/runtime/<module>.ruchy` files the used vocabularies bind their
/// terms to (`lowers_to: <module>::<fn>`), sorted and by sha256.
fn runtime_files(root: &Path, vocabularies: &[(String, u32)]) -> Vec<FileInput> {
    let mut modules: Vec<String> = vocabularies
        .iter()
        .filter_map(|(n, v)| load(root, n, *v).ok())
        .flat_map(|vocab| vocab.terms)
        .filter_map(|t| parse_binding(&t.lowers_to).map(|(m, _)| m.to_string()))
        .collect();
    modules.sort();
    modules.dedup();
    modules
        .into_iter()
        .map(|m| format!("vocab/runtime/{m}.ruchy"))
        .filter_map(|path| {
            let bytes = std::fs::read(root.join(&path)).ok()?;
            Some(FileInput {
                path,
                sha256: sha256_hex(&bytes),
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "receipt_tests.rs"]
mod receipt_tests;
