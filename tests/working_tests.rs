//! Minimal working test suite to verify coverage measurement works

use ruchy::frontend::lexer::Lexer;

#[test]
fn test_lexer_basic() {
    let lexer = Lexer::new("let x = 5");
    let tokens: Vec<_> = lexer.collect();
    assert!(!tokens.is_empty());
}

#[test]
fn test_lexer_operators() {
    let lexer = Lexer::new("+ - * /");
    let tokens: Vec<_> = lexer.collect();
    assert_eq!(tokens.len(), 4);
}

#[test] 
fn test_lexer_strings() {
    let lexer = Lexer::new(r#""hello world""#);
    let tokens: Vec<_> = lexer.collect();
    assert!(!tokens.is_empty());
}

#[test]
fn test_lexer_numbers() {
    let lexer = Lexer::new("42 3.14 1e10");
    let tokens: Vec<_> = lexer.collect();
    assert_eq!(tokens.len(), 3);
}

#[test]
fn test_lexer_identifiers() {
    let lexer = Lexer::new("foo bar_baz _test");
    let tokens: Vec<_> = lexer.collect();
    assert_eq!(tokens.len(), 3);
}
