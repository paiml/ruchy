//! `ruchy fix` and `ruchy vocab` for RHL (spec RHL-001 §5; roadmap row RHL-2).
//!
//! Every decision lives in the library (`ruchy::rhl::fix`,
//! `ruchy::rhl::vocab_cli`), under `cargo test --lib`. This handler only
//! prints an outcome and exits with its code: exit 2 cannot travel through
//! the generic `Err` path, which `main` maps to 1.

use anyhow::Result;
use ruchy::rhl::cli::{Outcome, OutputFormat};
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
