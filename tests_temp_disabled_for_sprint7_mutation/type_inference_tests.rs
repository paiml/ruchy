#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::approx_constant)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::unwrap_used, clippy::panic)]
//! Type inference tests to improve coverage

use anyhow::Result;
use ruchy::{middleend::InferenceContext, Parser};

#[test]
fn test_infer_integer_literal() -> Result<()> {
    let mut parser = Parser::new("42");
    let ast = parser.parse()?;
    let mut ctx = InferenceContext::new();
    let ty = ctx.infer(&ast)?;

    // Should infer as integer type
    assert!(ty.to_string().contains('i'));

    Ok(())
}

#[test]
fn test_infer_float_literal() -> Result<()> {
    let mut parser = Parser::new("3.14");
    let ast = parser.parse()?;
    let mut ctx = InferenceContext::new();
    let ty = ctx.infer(&ast)?;

    // Should infer as float type
    assert!(ty.to_string().contains('f'));

    Ok(())
}

#[test]
fn test_infer_bool_literal() -> Result<()> {
    let mut parser = Parser::new("true");
    let ast = parser.parse()?;
    let mut ctx = InferenceContext::new();
    let ty = ctx.infer(&ast)?;

    assert_eq!(ty.to_string(), "bool");

    Ok(())
}

#[test]
fn test_infer_string_literal() -> Result<()> {
    let mut parser = Parser::new("\"hello\"");
    let ast = parser.parse()?;
    let mut ctx = InferenceContext::new();
    let ty = ctx.infer(&ast)?;

    assert_eq!(ty.to_string(), "String");

    Ok(())
}

#[test]
fn test_infer_list_literal() -> Result<()> {
    let mut parser = Parser::new("[1, 2, 3]");
    let ast = parser.parse()?;
    let mut ctx = InferenceContext::new();
    let ty = ctx.infer(&ast)?;

    // Should infer as list/vector type
    let ty_str = ty.to_string();
    assert!(ty_str.contains('[') || ty_str.contains("List") || ty_str.contains("Vec"));

    Ok(())
}

#[test]
fn test_infer_binary_arithmetic() -> Result<()> {
    let mut parser = Parser::new("1 + 2");
    let ast = parser.parse()?;
    let mut ctx = InferenceContext::new();
    let ty = ctx.infer(&ast)?;

    // Should infer as numeric type
    assert!(ty.to_string().contains('i'));

    Ok(())
}

#[test]
fn test_infer_binary_comparison() -> Result<()> {
    let mut parser = Parser::new("1 < 2");
    let ast = parser.parse()?;
    let mut ctx = InferenceContext::new();
    let ty = ctx.infer(&ast)?;

    assert_eq!(ty.to_string(), "bool");

    Ok(())
}

#[test]
fn test_infer_binary_logical() -> Result<()> {
    let mut parser = Parser::new("true && false");
    let ast = parser.parse()?;
    let mut ctx = InferenceContext::new();
    let ty = ctx.infer(&ast)?;

    assert_eq!(ty.to_string(), "bool");

    Ok(())
}

#[test]
fn test_infer_if_expression() -> Result<()> {
    let mut parser = Parser::new("if true { 1 } else { 2 }");
    let ast = parser.parse()?;
    let mut ctx = InferenceContext::new();
    let ty = ctx.infer(&ast)?;

    // Should infer as the common type of branches
    assert!(ty.to_string().contains('i'));

    Ok(())
}

#[test]
fn test_infer_let_binding() -> Result<()> {
    let mut parser = Parser::new("let x = 42 in x + 1");
    let ast = parser.parse()?;
    let mut ctx = InferenceContext::new();
    let ty = ctx.infer(&ast)?;

    // Should infer as numeric type
    assert!(ty.to_string().contains('i'));

    Ok(())
}

#[test]
fn test_infer_function_type() -> Result<()> {
    let mut parser = Parser::new("fun add(x: i32, y: i32) -> i32 { x + y }");
    let ast = parser.parse()?;
    let mut ctx = InferenceContext::new();
    let ty = ctx.infer(&ast)?;

    // Function should have function type
    assert!(ty.to_string().contains("->"));

    Ok(())
}

#[test]
fn test_infer_lambda() -> Result<()> {
    let mut parser = Parser::new("fun (x) { x + 1 }");
    let ast = parser.parse()?;
    let mut ctx = InferenceContext::new();
    let ty = ctx.infer(&ast)?;

    // Lambda should have function type
    assert!(ty.to_string().contains("->"));

    Ok(())
}

#[test]
fn test_infer_unary_negation() -> Result<()> {
    let mut parser = Parser::new("-42");
    let ast = parser.parse()?;
    let mut ctx = InferenceContext::new();
    let ty = ctx.infer(&ast)?;

    // Should preserve numeric type
    assert!(ty.to_string().contains('i'));

    Ok(())
}

#[test]
fn test_infer_unary_not() -> Result<()> {
    let mut parser = Parser::new("!true");
    let ast = parser.parse()?;
    let mut ctx = InferenceContext::new();
    let ty = ctx.infer(&ast)?;

    assert_eq!(ty.to_string(), "bool");

    Ok(())
}

#[test]
fn test_infer_call_expression() -> Result<()> {
    // First define a function, then call it
    let mut parser = Parser::new("let add = fun (x, y) { x + y } in add(1, 2)");
    let ast = parser.parse()?;
    let mut ctx = InferenceContext::new();
    let ty = ctx.infer(&ast)?;

    // Should infer return type of function
    assert!(ty.to_string().contains('i'));

    Ok(())
}

#[test]
fn test_infer_match_expression() -> Result<()> {
    let mut parser = Parser::new(
        r"
        match 1 {
            0 => false,
            _ => true
        }
    ",
    );
    let ast = parser.parse()?;
    let mut ctx = InferenceContext::new();
    let ty = ctx.infer(&ast)?;

    // All branches return bool
    assert_eq!(ty.to_string(), "bool");

    Ok(())
}

#[test]
fn test_infer_struct_type() -> Result<()> {
    let mut parser = Parser::new("struct Point { x: f64, y: f64 }");
    let ast = parser.parse()?;
    let mut ctx = InferenceContext::new();
    let ty = ctx.infer(&ast)?;

    // Struct definition has unit type
    assert_eq!(ty.to_string(), "()");

    Ok(())
}

#[test]
fn test_infer_for_loop() -> Result<()> {
    let mut parser = Parser::new("for x in [1, 2, 3] { x }");
    let ast = parser.parse()?;
    let mut ctx = InferenceContext::new();
    let ty = ctx.infer(&ast)?;

    // For loop has unit type
    assert_eq!(ty.to_string(), "()");

    Ok(())
}
