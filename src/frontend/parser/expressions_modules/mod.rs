//! Parser expression modules
//! Refactored from monolithic expressions.rs (2,063 lines) to maintain ≤10 complexity per function

// Core parsing modules
pub mod literals;       // Literal values: integers, floats, strings, chars, f-strings
pub mod control_flow;   // Control flow: if, match, while, for, loop, break, continue, return
pub mod patterns;       // Pattern matching: wildcard, literal, tuple, list, struct, qualified
pub mod variables;      // Variable declarations: let, var, const, static, type aliases
pub mod functions;      // Functions: fn, async fn, lambda, closures, methods
pub mod operators;      // Operators: binary, unary, comparison, logical, bitwise, range, pipeline
pub mod collections;    // Collections: list, tuple, set, dict literals and comprehensions
pub mod data_structures;// Data structures: struct, enum, trait, impl blocks
pub mod imports;        // Import/Export: import, from, export, use statements  
pub mod actors;         // Actor system: actors, spawn, send, receive, select, async/await

// Re-export commonly used parsing functions
pub use literals::{parse_literal, parse_integer, parse_float, parse_string, parse_fstring};
pub use control_flow::{parse_if, parse_match, parse_while, parse_for, parse_loop, parse_break, parse_continue, parse_return};
pub use patterns::{parse_pattern, parse_tuple_pattern, parse_list_pattern, parse_range_pattern};
pub use variables::{parse_let, parse_var, parse_const, parse_static, parse_type_alias, parse_assignment};
pub use functions::{parse_function, parse_lambda, parse_closure, parse_method, parse_associated_function};
pub use operators::{parse_binary_op, parse_unary_op, parse_comparison, parse_logical, parse_arithmetic, parse_bitwise, parse_range, parse_pipeline, parse_in_operator, parse_is_operator, parse_as_operator};
pub use collections::{parse_list, parse_tuple, parse_set, parse_dict};
pub use data_structures::{parse_struct, parse_enum, parse_trait, parse_impl};
pub use imports::{parse_import, parse_from_import, parse_export, parse_use};
pub use actors::{parse_actor, parse_spawn, parse_send, parse_select, parse_async, parse_await, parse_yield};