//! Interactive theorem prover for Ruchy (RUCHY-0820)
//!
//! Provides REPL-based refinement type verification, property proving,
//! and counterexample generation.

pub mod prover;
pub mod tactics;
pub mod smt;
pub mod refinement;
pub mod counterexample;
pub mod verification;

pub use prover::{InteractiveProver, ProverSession, ProofResult, ProofGoal};
pub use tactics::{Tactic, TacticLibrary, TacticSuggestion};
pub use smt::{SmtSolver, SmtBackend, SmtQuery, SmtResult};
pub use refinement::{RefinementType, TypeRefinement, RefinementChecker};
pub use counterexample::{Counterexample, CounterexampleGenerator, TestCase};
pub use verification::{ProofVerificationResult, extract_assertions_from_ast, verify_single_assertion, verify_assertions_batch};