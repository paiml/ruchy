//! Middle Intermediate Representation (MIR) module

mod types;
mod lower;
mod builder;
mod optimize;

pub use types::*;
pub use lower::LoweringContext;
pub use builder::MirBuilder;
pub use optimize::{optimize_function, optimize_program, DeadCodeElimination, ConstantPropagation, CommonSubexpressionElimination};

// Re-export main types
pub use types::{Program, Function, BasicBlock, Statement, Terminator, Type};