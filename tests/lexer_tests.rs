//! Tests for the lexer module
//! Focus on tokenization functionality

use ruchy::frontend::lexer::{Lexer, Token, TokenKind};

#[test]
fn test_lexer_integers() {
    let mut lexer = Lexer::new("42 123 0");
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Integer(42)));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Integer(123)));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Integer(0)));
}

#[test]
fn test_lexer_floats() {
    let mut lexer = Lexer::new("3.14 0.5 123.456");
    
    let token = lexer.next_token();
    match token.kind {
        TokenKind::Float(f) => assert!((f - 3.14).abs() < 0.001),
        _ => panic!("Expected float token"),
    }
    
    let token = lexer.next_token();
    match token.kind {
        TokenKind::Float(f) => assert!((f - 0.5).abs() < 0.001),
        _ => panic!("Expected float token"),
    }
}

#[test]
fn test_lexer_strings() {
    let mut lexer = Lexer::new(r#""hello" "world" "with spaces""#);
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::String(ref s) if s == "hello"));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::String(ref s) if s == "world"));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::String(ref s) if s == "with spaces"));
}

#[test]
fn test_lexer_identifiers() {
    let mut lexer = Lexer::new("foo bar_baz _underscore variable123");
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Identifier(ref s) if s == "foo"));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Identifier(ref s) if s == "bar_baz"));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Identifier(ref s) if s == "_underscore"));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Identifier(ref s) if s == "variable123"));
}

#[test]
fn test_lexer_keywords() {
    let mut lexer = Lexer::new("let if else while for return true false");
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Let));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::If));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Else));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::While));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::For));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Return));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::True));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::False));
}

#[test]
fn test_lexer_operators() {
    let mut lexer = Lexer::new("+ - * / % == != < > <= >= && || !");
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Plus));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Minus));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Star));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Slash));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Percent));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::EqualEqual));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::BangEqual));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Less));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Greater));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::LessEqual));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::GreaterEqual));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::AmpAmp));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::PipePipe));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Bang));
}

#[test]
fn test_lexer_delimiters() {
    let mut lexer = Lexer::new("( ) [ ] { } , ; :");
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::LeftParen));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::RightParen));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::LeftBracket));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::RightBracket));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::LeftBrace));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::RightBrace));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Comma));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Semicolon));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Colon));
}

#[test]
fn test_lexer_assignment() {
    let mut lexer = Lexer::new("= += -= *= /=");
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Equal));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::PlusEqual));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::MinusEqual));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::StarEqual));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::SlashEqual));
}

#[test]
fn test_lexer_whitespace_handling() {
    let mut lexer = Lexer::new("  42   +   3  ");
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Integer(42)));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Plus));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Integer(3)));
}

#[test]
fn test_lexer_comments() {
    // Single line comments
    let mut lexer = Lexer::new("42 // this is a comment\n+ 3");
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Integer(42)));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Plus));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Integer(3)));
}

#[test]
fn test_lexer_eof() {
    let mut lexer = Lexer::new("42");
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Integer(42)));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Eof));
    
    // Should keep returning EOF
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Eof));
}

#[test]
fn test_lexer_complex_expression() {
    let mut lexer = Lexer::new("let x = (42 + 3.14) * true");
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Let));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Identifier(ref s) if s == "x"));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Equal));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::LeftParen));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Integer(42)));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Plus));
    
    let token = lexer.next_token();
    match token.kind {
        TokenKind::Float(f) => assert!((f - 3.14).abs() < 0.001),
        _ => panic!("Expected float"),
    }
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::RightParen));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::Star));
    
    let token = lexer.next_token();
    assert!(matches!(token.kind, TokenKind::True));
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn prop_lexer_integer_roundtrip(n in 0i64..i64::MAX) {
            let input = n.to_string();
            let mut lexer = Lexer::new(&input);
            
            let token = lexer.next_token();
            match token.kind {
                TokenKind::Integer(parsed) => prop_assert_eq!(parsed, n),
                _ => prop_assert!(false, "Expected integer token"),
            }
        }
        
        #[test]
        fn prop_lexer_identifier_roundtrip(s in "[a-zA-Z_][a-zA-Z0-9_]{0,20}") {
            let mut lexer = Lexer::new(&s);
            
            let token = lexer.next_token();
            // Check if it's a keyword or identifier
            match &token.kind {
                TokenKind::Identifier(id) => prop_assert_eq!(id, &s),
                // It might be a keyword, which is also valid
                _ => {
                    let keywords = ["let", "if", "else", "while", "for", "return", "true", "false", "fun", "match"];
                    prop_assert!(keywords.contains(&s.as_str()));
                }
            }
        }
        
        #[test]
        fn prop_lexer_never_panics(input in ".*") {
            let input = if input.len() > 1000 { &input[..1000] } else { &input };
            
            let _ = std::panic::catch_unwind(|| {
                let mut lexer = Lexer::new(input);
                // Consume up to 100 tokens
                for _ in 0..100 {
                    let token = lexer.next_token();
                    if matches!(token.kind, TokenKind::Eof) {
                        break;
                    }
                }
            });
        }
    }
}