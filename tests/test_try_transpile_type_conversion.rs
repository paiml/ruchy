//! TDD tests for try_transpile_type_conversion refactoring
//! Original complexity: 62 (WAY over limit!)
//! Target complexity: <20 per function

use ruchy::backend::Transpiler;
use ruchy::frontend::ast::{Expr, ExprKind, Literal};
use anyhow::Result;

#[test]
fn test_str_conversion_from_int() -> Result<()> {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Integer(42)),
        span: Default::default(),
    };
    
    let result = transpiler.try_transpile_type_conversion("str", &[expr])?;
    assert!(result.is_some());
    
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("format!"));
    
    Ok(())
}

#[test]
fn test_int_conversion_from_string() -> Result<()> {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::String("42".to_string())),
        span: Default::default(),
    };
    
    let result = transpiler.try_transpile_type_conversion("int", &[expr])?;
    assert!(result.is_some());
    
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("parse::<i64>()"));
    
    Ok(())
}

#[test]
fn test_int_conversion_from_float() -> Result<()> {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Float(42.5)),
        span: Default::default(),
    };
    
    let result = transpiler.try_transpile_type_conversion("int", &[expr])?;
    assert!(result.is_some());
    
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("as i64"));
    
    Ok(())
}

#[test]
fn test_float_conversion_from_int() -> Result<()> {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Integer(42)),
        span: Default::default(),
    };
    
    let result = transpiler.try_transpile_type_conversion("float", &[expr])?;
    assert!(result.is_some());
    
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("as f64"));
    
    Ok(())
}

#[test]
fn test_bool_conversion_from_int() -> Result<()> {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Integer(1)),
        span: Default::default(),
    };
    
    let result = transpiler.try_transpile_type_conversion("bool", &[expr])?;
    assert!(result.is_some());
    
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("!= 0"));
    
    Ok(())
}

#[test]
fn test_bool_conversion_from_string() -> Result<()> {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::String("hello".to_string())),
        span: Default::default(),
    };
    
    let result = transpiler.try_transpile_type_conversion("bool", &[expr])?;
    assert!(result.is_some());
    
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("is_empty()"));
    
    Ok(())
}

#[test]
fn test_invalid_arg_count() -> Result<()> {
    let transpiler = Transpiler::new();
    let expr1 = Expr {
        kind: ExprKind::Literal(Literal::Integer(1)),
        span: Default::default(),
    };
    let expr2 = Expr {
        kind: ExprKind::Literal(Literal::Integer(2)),
        span: Default::default(),
    };
    
    let result = transpiler.try_transpile_type_conversion("str", &[expr1, expr2]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("expects exactly 1 argument"));
    
    Ok(())
}

#[test]
fn test_unknown_type_conversion() -> Result<()> {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Integer(42)),
        span: Default::default(),
    };
    
    let result = transpiler.try_transpile_type_conversion("unknown", &[expr])?;
    assert!(result.is_none());
    
    Ok(())
}