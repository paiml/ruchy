//! Middle-end compiler passes (type checking, inference, optimization)

pub mod environment;
pub mod infer;
pub mod types;
pub mod unify;
pub mod mir;

// Re-export commonly used types
pub use environment::TypeEnv;
pub use infer::InferenceContext;
pub use types::{MonoType, TyVar, TyVarGenerator, TypeScheme};
pub use unify::Unifier;
pub use mir::{Program as MirProgram, Function as MirFunction};
