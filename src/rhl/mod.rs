//! RHL, the Ruchy High-Level language interface (spec RHL-001,
//! `docs/specifications/ruchy-high-level-language-interface.md`).
//!
//! RHL is a small, closed-keyword, line-oriented language in which a person or
//! an agent states *intent* — a job, a check, a command — in terms of a
//! versioned vocabulary (`disk free of`, `file ticket`), and Ruchy checks and
//! compiles that intent. Every surface ends in one intent tree ([`tree`]),
//! which [`parse`] produces from text. The parser is generated at build time by
//! LALRPOP from `src/rhl/grammar.lalrpop`, whose production skeleton is gated
//! identical to the pre-registered, conflict-free `grammar/rhl.lalrpop`
//! (`parse_tests`), so F1's "exactly one parse" is a claim about this parser.

pub mod check;
pub mod cli;
pub mod codes;
pub mod diag;
pub mod fix;
pub mod fmt;
pub mod parse;
pub mod tree;
pub mod vocab;
pub mod vocab_cli;

lalrpop_util::lalrpop_mod!(grammar, "/rhl/grammar.rs");

pub use parse::{parse, ParseFailure};

#[cfg(test)]
mod parse_tests;

/// Repository root, from the crate manifest, so a test reads the same files
/// under `cargo test`, nextest and an extracted `.crate` alike.
#[cfg(test)]
pub(crate) fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
