use ruchy::Parser;

#[test]
fn test_validate_url_import_https() {
    let code = r#"import "https://example.com/lib.ruchy""#;
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Should accept HTTPS imports");
}

#[test]
fn test_validate_url_import_localhost() {
    let code = r#"import "http://localhost:8080/lib.ruchy""#;
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Should accept localhost HTTP imports");
}

#[test]
fn test_validate_url_import_invalid_http() {
    let code = r#"import "http://example.com/lib.ruchy""#;
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_err(), "Should reject non-localhost HTTP imports");
}

#[test]
fn test_validate_url_import_wrong_extension() {
    let code = r#"import "https://example.com/lib.js""#;
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_err(), "Should reject non-.ruchy extensions");
}

#[test]
fn test_validate_url_import_path_traversal() {
    let code = r#"import "https://example.com/../etc/passwd.ruchy""#;
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_err(), "Should reject path traversal");
}

#[test]
fn test_validate_url_import_javascript_scheme() {
    let code = r#"import "javascript:alert('xss').ruchy""#;
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_err(), "Should reject javascript: scheme");
}

#[test]
fn test_parse_params_simple() {
    let code = "fun add(x, y) { x + y }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ruchy::frontend::ast::ExprKind::Function { params, .. } => {
            assert_eq!(params.len(), 2);
            // Params parsed successfully
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_parse_params_with_types() {
    let code = "fun add(x: i32, y: i32) { x + y }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ruchy::frontend::ast::ExprKind::Function { params, .. } => {
            assert_eq!(params.len(), 2);
            // Type annotations parsed
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_parse_params_with_defaults() {
    let code = "fun greet(name = \"World\") { println(f\"Hello {name}\") }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ruchy::frontend::ast::ExprKind::Function { params, .. } => {
            assert_eq!(params.len(), 1);
            // Default value parsed
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_parse_simple_let() {
    let code = "let x = 42";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ruchy::frontend::ast::ExprKind::Let { name, .. } => {
            assert_eq!(name, "x");
        }
        _ => panic!("Expected let statement"),
    }
}

#[test]
fn test_parse_let_with_type() {
    let code = "let x: i32 = 42";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ruchy::frontend::ast::ExprKind::Let { name, type_annotation, .. } => {
            assert_eq!(name, "x");
            assert!(type_annotation.is_some());
        }
        _ => panic!("Expected let statement"),
    }
}

#[test]
fn test_parse_let_mut() {
    let code = "let mut x = 42";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ruchy::frontend::ast::ExprKind::Let { is_mutable, .. } => {
            assert!(*is_mutable);
        }
        _ => panic!("Expected let statement"),
    }
}


#[test]
fn test_parse_rest_pattern() {
    let code = "fun sum(...numbers) { numbers.reduce((a, b) => a + b) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ruchy::frontend::ast::ExprKind::Function { params, .. } => {
            assert_eq!(params.len(), 1);
            // Rest param parsed
        }
        _ => panic!("Expected function"),
    }
}