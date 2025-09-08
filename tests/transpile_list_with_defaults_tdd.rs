use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::ast::{Expr, ExprKind, Literal, Pattern, Param};
use ruchy::frontend::parser::Parser;
use std::collections::HashMap;

fn create_transpiler() -> Transpiler {
    Transpiler {
        struct_definitions: HashMap::new(),
        ..Default::default()
    }
}

fn parse_expr(code: &str) -> Expr {
    let mut parser = Parser::new(code);
    parser.parse_expression().expect("Failed to parse expression")
}

fn parse_pattern(code: &str) -> Pattern {
    let mut parser = Parser::new(code);
    parser.parse_pattern().expect("Failed to parse pattern")
}

#[test]
fn test_list_with_defaults_simple_identifiers() {
    let transpiler = create_transpiler();
    
    // Test: [a = 10, b = 20] = arr
    let patterns = vec![
        Pattern::WithDefault {
            pattern: Box::new(Pattern::Identifier("a".to_string())),
            default: Box::new(parse_expr("10")),
        },
        Pattern::WithDefault {
            pattern: Box::new(Pattern::Identifier("b".to_string())),
            default: Box::new(parse_expr("20")),
        },
    ];
    
    let value = parse_expr("[1, 2]");
    let body = Expr {
        kind: ExprKind::Literal(Literal::Unit),
        span: Default::default(),
    };
    
    let result = transpiler.transpile_list_with_defaults(&patterns, &value, &body);
    assert!(result.is_ok());
    
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    // Should generate code that:
    // 1. Declares variables
    // 2. Creates temp_array
    // 3. Conditionally assigns values or defaults
    assert!(code.contains("let mut a"));
    assert!(code.contains("let mut b"));
    assert!(code.contains("temp_array"));
    assert!(code.contains("if temp_array.len() > 0"));
    assert!(code.contains("if temp_array.len() > 1"));
}

#[test]
fn test_list_with_defaults_mixed_patterns() {
    let transpiler = create_transpiler();
    
    // Test: [a, b = 20] = arr
    let patterns = vec![
        Pattern::Identifier("a".to_string()),
        Pattern::WithDefault {
            pattern: Box::new(Pattern::Identifier("b".to_string())),
            default: Box::new(parse_expr("20")),
        },
    ];
    
    let value = parse_expr("[1]");
    let body = Expr {
        kind: ExprKind::Literal(Literal::Unit),
        span: Default::default(),
    };
    
    let result = transpiler.transpile_list_with_defaults(&patterns, &value, &body);
    assert!(result.is_ok());
    
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    // a should use direct indexing, b should check length
    assert!(code.contains("a = temp_array[0].clone()"));
    assert!(code.contains("if temp_array.len() > 1"));
}

#[test]
fn test_list_with_defaults_with_body() {
    let transpiler = create_transpiler();
    
    // Test: [a = 10, b = 20] = arr; a + b
    let patterns = vec![
        Pattern::WithDefault {
            pattern: Box::new(Pattern::Identifier("a".to_string())),
            default: Box::new(parse_expr("10")),
        },
        Pattern::WithDefault {
            pattern: Box::new(Pattern::Identifier("b".to_string())),
            default: Box::new(parse_expr("20")),
        },
    ];
    
    let value = parse_expr("[5]");
    let body = parse_expr("a + b");
    
    let result = transpiler.transpile_list_with_defaults(&patterns, &value, &body);
    assert!(result.is_ok());
    
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    // Should wrap in a block and include body
    assert!(code.starts_with("{"));
    assert!(code.ends_with("}"));
    assert!(code.contains("a + b"));
}

#[test]
fn test_list_with_defaults_empty_array() {
    let transpiler = create_transpiler();
    
    // Test: [a = 10, b = 20] = []
    let patterns = vec![
        Pattern::WithDefault {
            pattern: Box::new(Pattern::Identifier("a".to_string())),
            default: Box::new(parse_expr("10")),
        },
        Pattern::WithDefault {
            pattern: Box::new(Pattern::Identifier("b".to_string())),
            default: Box::new(parse_expr("20")),
        },
    ];
    
    let value = parse_expr("[]");
    let body = Expr {
        kind: ExprKind::Literal(Literal::Unit),
        span: Default::default(),
    };
    
    let result = transpiler.transpile_list_with_defaults(&patterns, &value, &body);
    assert!(result.is_ok());
    
    // Both should use defaults since array is empty
    let tokens = result.unwrap();
    let code = tokens.to_string();
    assert!(code.contains("10"));
    assert!(code.contains("20"));
}

#[test]
fn test_list_with_defaults_complex_defaults() {
    let transpiler = create_transpiler();
    
    // Test: [a = get_default(), b = 10 * 2] = arr
    let patterns = vec![
        Pattern::WithDefault {
            pattern: Box::new(Pattern::Identifier("a".to_string())),
            default: Box::new(parse_expr("get_default()")),
        },
        Pattern::WithDefault {
            pattern: Box::new(Pattern::Identifier("b".to_string())),
            default: Box::new(parse_expr("10 * 2")),
        },
    ];
    
    let value = parse_expr("arr");
    let body = Expr {
        kind: ExprKind::Literal(Literal::Unit),
        span: Default::default(),
    };
    
    let result = transpiler.transpile_list_with_defaults(&patterns, &value, &body);
    assert!(result.is_ok());
    
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    // Should preserve complex default expressions
    assert!(code.contains("get_default()"));
    assert!(code.contains("10 * 2"));
}

#[test]
fn test_list_with_defaults_unsupported_pattern() {
    let transpiler = create_transpiler();
    
    // Test: [[nested], b = 20] = arr (nested patterns not supported)
    let patterns = vec![
        Pattern::List(vec![Pattern::Identifier("nested".to_string())]),
        Pattern::WithDefault {
            pattern: Box::new(Pattern::Identifier("b".to_string())),
            default: Box::new(parse_expr("20")),
        },
    ];
    
    let value = parse_expr("arr");
    let body = Expr {
        kind: ExprKind::Literal(Literal::Unit),
        span: Default::default(),
    };
    
    let result = transpiler.transpile_list_with_defaults(&patterns, &value, &body);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unsupported pattern type"));
}