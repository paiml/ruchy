//! Interactive theorem prover for Ruchy (RUCHY-0820)
//!
//! Provides REPL-based refinement type verification, property proving,
//! and counterexample generation.
pub mod counterexample;
pub mod prover;
pub mod refinement;
pub mod smt;
pub mod tactics;
pub mod verification;
pub use counterexample::{Counterexample, CounterexampleGenerator, TestCase};
pub use prover::{InteractiveProver, ProofGoal, ProofResult, ProverSession};
pub use refinement::{RefinementChecker, RefinementType, TypeRefinement};
pub use smt::{SmtBackend, SmtQuery, SmtResult, SmtSolver};
pub use tactics::{Tactic, TacticLibrary, TacticSuggestion};
pub use verification::{
    extract_assertions_from_ast, verify_assertions_batch, verify_single_assertion,
    ProofVerificationResult,
};

// Tests removed: This module only re-exports from submodules.
// Actual implementations and tests belong in the submodules.
