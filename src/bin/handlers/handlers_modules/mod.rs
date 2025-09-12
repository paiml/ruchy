//! Handlers modules - refactored for ≤10 complexity per function
//! Extracted from monolithic 1,938-line handlers/mod.rs
// Core command modules
pub mod prove;
pub mod prove_helpers;
pub mod test;
pub mod test_helpers;
// Re-export main handler functions
