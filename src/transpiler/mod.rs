//! Transpiler module implementing extreme quality engineering
//!
//! Based on docs/ruchy-transpiler-docs.md
pub mod canonical_ast;
pub mod provenance;
pub mod reference_interpreter;
// Re-exports
pub use canonical_ast::{AstNormalizer, CoreExpr, CoreLiteral, DeBruijnIndex, PrimOp};
pub use provenance::{CompilationTrace, ProvenanceTracker, TraceDiffer};
pub use reference_interpreter::{Environment, ReferenceInterpreter, Value};

// Tests removed: This module only re-exports from submodules.
// Actual implementations and tests belong in the submodules.
