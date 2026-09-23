//! `ruchy fix`, `ruchy vocab` (spec RHL-001 §5; roadmap row RHL-2) and
//! `ruchy convert` (row RHL-3) for RHL.
//!
//! Every decision lives in the library (`ruchy::rhl::fix`,
//! `ruchy::rhl::vocab_cli`), under `cargo test --lib`. This handler only
//! prints an outcome and exits with its code: exit 2 cannot travel through
//! the generic `Err` path, which `main` maps to 1.

use anyhow::Result;
use ruchy::rhl::cli::{
    compile_file, convert_file, is_rhl, run_file, transpile_file, Outcome, OutputFormat,
};
use ruchy::rhl::fix::fix_files;
use ruchy::rhl::vocab_cli::{list_outcome, no_root, resolve_root, show_outcome, validate_outcome};
use std::path::{Path, PathBuf};

/// `ruchy fix [--safe] <file.rhl>...`.
///
/// # Errors
///
/// Returns an error for an unknown `--format` or an empty file list.
pub fn handle_fix_command(files: &[PathBuf], safe: bool, format: &str) -> Result<()> {
    if files.is_empty() {
        anyhow::bail!("No files specified for fixing");
    }
    let format = OutputFormat::parse(format).map_err(anyhow::Error::msg)?;
    let paths: Vec<&Path> = files.iter().map(PathBuf::as_path).collect();
    finish(&fix_files(&paths, safe, format))
}

/// `ruchy convert <input> [-o <output>] [--to rhl|yaml]`: `.rhl` ⇄ `.rhl.yaml`.
///
/// # Errors
///
/// Never returns one: every outcome, including a refusal, is printed and
/// carried by the exit code.
pub fn handle_convert_command(input: &Path, output: Option<&Path>, to: Option<&str>) -> Result<()> {
    finish(&convert_file(input, output, to))
}

/// `ruchy transpile`: an RHL file (`.rhl`, `.rhl.yaml`) is checked, lowered
/// and transpiled by the library (RHL-4); any other file takes the existing
/// ruchy path unchanged.
///
/// # Errors
///
/// Returns an error for `--emit` on a non-RHL file, or the ruchy path's error.
pub fn handle_transpile(
    file: &Path,
    output: Option<&Path>,
    minimal: bool,
    emit: Option<&str>,
    verbose: bool,
) -> Result<()> {
    if is_rhl(file) {
        return finish(&transpile_file(file, output, emit));
    }
    if emit.is_some() {
        anyhow::bail!("--emit applies to .rhl and .rhl.yaml files only");
    }
    super::handle_transpile_command(file, output, minimal, verbose)
}

/// `ruchy compile` on an RHL file: build the job (RHL-4).
///
/// # Errors
///
/// Never returns one: the outcome is printed and carried by the exit code.
pub fn handle_rhl_compile(file: &Path, output: &Path) -> Result<()> {
    finish(&compile_file(file, output))
}

/// `ruchy run [--apply]`: an RHL file is compiled and executed (RHL-4);
/// `--apply` exists only for RHL files; any other file takes the existing
/// ruchy path unchanged.
///
/// # Errors
///
/// Returns an error for `--apply` on a non-RHL file.
pub fn handle_run(file: &Path, apply: bool) -> Result<Option<()>> {
    if is_rhl(file) {
        return finish(&run_file(file, apply)).map(Some);
    }
    if apply {
        anyhow::bail!("--apply applies to .rhl and .rhl.yaml files only");
    }
    Ok(None)
}

/// What `ruchy vocab` was asked to do.
pub enum VocabRequest {
    /// `vocab list`.
    List,
    /// `vocab show <name> <version>`.
    Show {
        /// Vocabulary name.
        name: String,
        /// Version, `v2` or `2`.
        version: String,
    },
    /// `vocab validate <file.yaml> | <name> <version>`.
    Validate {
        /// The file, or the name and version.
        target: Vec<String>,
    },
}

/// `ruchy vocab list | show | validate`.
///
/// # Errors
///
/// Returns an error for an unknown `--format`, an unreadable current
/// directory, or no vocabulary root where one is needed.
pub fn handle_vocab_command(
    request: VocabRequest,
    format: &str,
    root: Option<&Path>,
) -> Result<()> {
    let format = OutputFormat::parse(format).map_err(anyhow::Error::msg)?;
    let found = resolve_root(root, &std::env::current_dir()?);
    let outcome = match (request, found) {
        (VocabRequest::Validate { target }, found) => {
            validate_outcome(found.as_deref(), &target, format)
        }
        (_, None) => anyhow::bail!(no_root()),
        (VocabRequest::List, Some(r)) => list_outcome(&r, format),
        (VocabRequest::Show { name, version }, Some(r)) => {
            show_outcome(&r, &name, &version, format)
        }
    };
    finish(&outcome)
}

fn finish(outcome: &Outcome) -> Result<()> {
    print!("{}", outcome.stdout);
    eprint!("{}", outcome.stderr);
    if outcome.exit != 0 {
        std::process::exit(outcome.exit);
    }
    Ok(())
}
