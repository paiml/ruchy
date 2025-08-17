//! Runtime execution and REPL support
//!
//! This module provides the interactive REPL and runtime execution environment.

pub mod repl;
pub mod repl_grammar_coverage;
mod repl_tests;
pub mod repl_v2;

// Export ReplV2 as the default Repl
pub use repl_v2::ReplV2 as Repl;
// Keep old REPL available as LegacyRepl for compatibility
pub use repl::Repl as LegacyRepl;
pub use repl_v2::ReplV2;
