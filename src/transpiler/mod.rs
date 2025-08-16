//! Transpiler module implementing extreme quality engineering
//! 
//! Based on docs/ruchy-transpiler-docs.md

pub mod canonical_ast;
pub mod reference_interpreter;
pub mod provenance;

// Re-exports
pub use canonical_ast::{AstNormalizer, CoreExpr, CoreLiteral, DeBruijnIndex, PrimOp};
pub use reference_interpreter::{ReferenceInterpreter, Value, Environment};
pub use provenance::{CompilationTrace, ProvenanceTracker, TraceDiffer};