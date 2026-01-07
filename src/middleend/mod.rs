//! Middle-end compiler passes (type checking, inference, optimization)
pub mod environment;
pub mod infer;
pub mod mir;
pub mod types;
pub mod unify;

#[cfg(test)]
mod infer_tests;
// Re-export commonly used types
pub use environment::TypeEnv;
pub use infer::InferenceContext;
pub use mir::{Function as MirFunction, Program as MirProgram};
pub use types::{MonoType, TyVar, TyVarGenerator, TypeScheme};
pub use unify::Unifier;

// Tests removed: Module only re-exports. Tests belong in submodules.
