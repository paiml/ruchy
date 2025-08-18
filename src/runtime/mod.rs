//! Runtime execution and REPL support
//!
//! This module provides the interactive REPL, runtime execution environment,
//! and actor system with supervision trees.

pub mod actor;
pub mod grammar_coverage;
pub mod repl;

// Export the unified REPL
pub use repl::{Repl, ReplConfig, Value};

// Export actor system components
pub use actor::{
    ActorBehavior, ActorContext, ActorId, ActorRef, ActorSystem, EchoActor, Message, MessageValue,
    SupervisorActor, SupervisorDirective,
};
