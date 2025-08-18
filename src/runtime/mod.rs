//! Runtime execution and REPL support
//!
//! This module provides the interactive REPL, runtime execution environment,
//! and actor system with supervision trees.

pub mod actor;
pub mod repl;
pub mod repl_grammar_coverage;
mod repl_tests;
pub mod repl_v2;
pub mod repl_v3;

// Export ReplV2 as the default Repl
pub use repl_v2::ReplV2 as Repl;
// Keep old REPL available as LegacyRepl for compatibility
pub use repl::Repl as LegacyRepl;
pub use repl_v2::ReplV2;

// Export actor system components
pub use actor::{
    ActorBehavior, ActorContext, ActorId, ActorRef, ActorSystem, EchoActor, Message, MessageValue,
    SupervisorActor, SupervisorDirective,
};
