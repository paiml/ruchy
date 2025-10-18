//! Expression parsing helper modules
//!
//! This module contains focused submodules extracted from the monolithic
//! expressions.rs file to improve maintainability and enable effective testing.
//!
//! Modularization improves TDG Structural score from 0/25 to target ≥21/25.

pub mod arrays;
pub mod async_expressions;
pub mod binary_operators;
pub mod classes;
pub mod control_flow;
pub mod dataframes;
pub mod enums;
pub mod error_handling;
pub mod identifiers;
pub mod impls;
pub mod increment_decrement;
pub mod lambdas;
pub mod literals;
pub mod loops;
pub mod modules;
pub mod patterns;
pub mod string_operations;
pub mod structs;
pub mod traits;
pub mod tuples;
pub mod type_aliases;
pub mod unary_operators;
pub mod use_statements;
pub mod variable_declarations;
pub mod visibility_modifiers;
