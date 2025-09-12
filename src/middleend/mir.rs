//! Middle Intermediate Representation (MIR) module
mod builder;
mod lower;
mod optimize;
mod types;
pub use builder::MirBuilder;
pub use lower::LoweringContext;
pub use optimize::{
    optimize_function, optimize_program, CommonSubexpressionElimination, ConstantPropagation,
    DeadCodeElimination,
};
pub use types::*;
// Re-export main types
pub use types::{BasicBlock, Function, Program, Statement, Terminator, Type};
