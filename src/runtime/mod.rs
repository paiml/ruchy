//! Runtime execution and REPL support
//!
//! This module provides the interactive REPL, runtime execution environment,
//! and actor system with supervision trees.

pub mod actor;
pub mod cache;
pub mod grammar_coverage;
pub mod lazy;
pub mod repl;
#[cfg(test)]
mod repl_function_tests;

// Export the unified REPL
pub use repl::{Repl, ReplConfig, Value};

// Export actor system components
pub use actor::{
    ActorBehavior, ActorContext, ActorId, ActorRef, ActorSystem, EchoActor, Message, MessageValue,
    SupervisorActor, SupervisorDirective,
};
