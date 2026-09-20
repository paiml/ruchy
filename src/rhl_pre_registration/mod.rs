//! RHL-0 pre-registration gates (spec `RHL-001`, row RHL-0).
//!
//! These modules contain **no RHL compiler code**. RHL-0's whole point is that
//! the grammar, the task corpus, the planted-break corpus and the vocabularies
//! are committed *before* any compiler exists, so that the falsifiers F1–F9
//! cannot later be tuned to fit an implementation. What lives here is the set of
//! gates that hold those artifacts to the shape RHL-001 pre-registered — every
//! module is `#[cfg(test)]`, so nothing here enters a release artifact.
//!
//! # Why `src/` and not `tests/`
//!
//! Measured at HEAD (see `docs/rhl/phase0-bindings.md` binding B4): the required
//! status check `gate` re-exports `paiml/.github`'s `sovereign-ci.yml`, whose
//! test job runs `cargo test --lib` against the root crate only — `TEST_SCOPE`
//! is `--lib` because `.github/workflows/ci.yml` passes no `test_workspace`
//! input. An integration test under `tests/` would therefore never run in the
//! required check, and a gate that does not run is not a gate. This crate
//! already carries 66 `#[cfg(test)]` test modules under `src/` (for example
//! `src/frontend/ast_tests.rs`), so this placement is the crate's own pattern.
//!
//! # The five gates
//!
//! | Module | Falsifier | Holds |
//! |---|---|---|
//! | [`grammar_gate`] | F1 | the grammar is conflict-free by LALRPOP's own report; the fixture is rejected with LALRPOP's conflict error specifically; `Program` is the only entry point; and every §3.3 keyword still appears in a production |
//! | [`vocab_gate`] | F4, F6 | every vocabulary term has a contract template and a kind from the closed list |
//! | [`corpus_gate`] | F8 | the task corpus is the pre-registered size and shape, and its Rust tests are real Rust |
//! | [`breaks_gate`] | F7 | every planted break is a real mutation of a named valid program, in a declared class |
//! | [`manifest_gate`] | all | the pre-registered bytes still hash to what RHL-0 committed |
//!
//! # What these gates do NOT establish
//!
//! `grammar_gate` puts a floor under the grammar but no ceiling: it cannot show
//! that the committed corpus parses, because that needs a generated parser and
//! RHL-0 ships no compiler code. The manifest covers the pre-registered data,
//! the spec and the bindings, but deliberately NOT these gate modules, which
//! later rows are meant to extend. Both are named in this row's receipt.

#[cfg(test)]
mod breaks_gate;
#[cfg(test)]
mod corpus_gate;
#[cfg(test)]
mod grammar_gate;
#[cfg(test)]
mod manifest_gate;
#[cfg(test)]
mod vocab_gate;

/// Repository root, resolved from the crate manifest rather than the process
/// working directory, so a gate reads the same files under `cargo test`,
/// `cargo nextest` and the sovereign-ci container alike.
#[cfg(test)]
pub(crate) fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
