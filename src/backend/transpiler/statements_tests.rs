//! Tests for statement and control flow transpilation
//! EXTREME TDD Round 83: Extracted from statements.rs
//!
//! This module contains comprehensive tests for the transpiler's statement handling.

use crate::backend::transpiler::return_type_helpers::{
    expr_is_string, returns_boolean, returns_object_literal, returns_string,
    returns_string_literal, returns_vec,
};
use crate::backend::transpiler::Transpiler;
use crate::frontend::ast::{
    BinaryOp, Expr, ExprKind, Literal, Param, Pattern, Span, Type, TypeKind, UnaryOp,
};
use crate::frontend::parser::Parser;

fn create_transpiler() -> Transpiler {
    Transpiler::new()
}

fn _create_transpiler_dup() -> Transpiler {
    Transpiler::new()
}

#[path = "statements_tests_part1.rs"]
mod part1;
#[path = "statements_tests_part2.rs"]
mod part2;
#[path = "statements_tests_part3.rs"]
mod part3;
