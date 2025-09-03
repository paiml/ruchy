//! Comprehensive TDD test suite for lexer.rs
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every lexical analysis path must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::frontend::Token;
use logos::Logos;

// ==================== INTEGER LITERAL TESTS ====================

#[test]
fn test_lex_integer_simple() {
    let mut lex = Token::lexer("42");
    assert_eq!(lex.next(), Some(Ok(Token::Integer(42))));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_lex_integer_with_suffix() {
    let mut lex = Token::lexer("42i32");
    assert_eq!(lex.next(), Some(Ok(Token::Integer(42))));
}

#[test]
fn test_lex_integer_zero() {
    let mut lex = Token::lexer("0");
    assert_eq!(lex.next(), Some(Ok(Token::Integer(0))));
}

#[test]
fn test_lex_integer_large() {
    let mut lex = Token::lexer("1000000");
    assert_eq!(lex.next(), Some(Ok(Token::Integer(1000000))));
}

// ==================== FLOAT LITERAL TESTS ====================

#[test]
fn test_lex_float_simple() {
    let mut lex = Token::lexer("3.14");
    assert_eq!(lex.next(), Some(Ok(Token::Float(3.14))));
}

#[test]
fn test_lex_float_with_exponent() {
    let mut lex = Token::lexer("1.5e10");
    assert_eq!(lex.next(), Some(Ok(Token::Float(1.5e10))));
}

#[test]
fn test_lex_float_negative_exponent() {
    let mut lex = Token::lexer("2.5e-3");
    assert_eq!(lex.next(), Some(Ok(Token::Float(2.5e-3))));
}

#[test]
fn test_lex_float_zero() {
    let mut lex = Token::lexer("0.0");
    assert_eq!(lex.next(), Some(Ok(Token::Float(0.0))));
}

// ==================== STRING LITERAL TESTS ====================

#[test]
fn test_lex_string_simple() {
    let mut lex = Token::lexer(r#""hello""#);
    assert_eq!(lex.next(), Some(Ok(Token::String("hello".to_string()))));
}

#[test]
fn test_lex_string_empty() {
    let mut lex = Token::lexer(r#""""#);
    assert_eq!(lex.next(), Some(Ok(Token::String("".to_string()))));
}

#[test]
fn test_lex_string_with_escape_newline() {
    let mut lex = Token::lexer(r#""hello\nworld""#);
    assert_eq!(lex.next(), Some(Ok(Token::String("hello\nworld".to_string()))));
}

#[test]
fn test_lex_string_with_escape_tab() {
    let mut lex = Token::lexer(r#""hello\tworld""#);
    assert_eq!(lex.next(), Some(Ok(Token::String("hello\tworld".to_string()))));
}

#[test]
fn test_lex_string_with_escape_quote() {
    let mut lex = Token::lexer(r#""say \"hello\"""#);
    assert_eq!(lex.next(), Some(Ok(Token::String("say \"hello\"".to_string()))));
}

#[test]
fn test_lex_string_with_escape_backslash() {
    let mut lex = Token::lexer(r#""path\\to\\file""#);
    assert_eq!(lex.next(), Some(Ok(Token::String("path\\to\\file".to_string()))));
}

#[test]
fn test_lex_fstring() {
    let mut lex = Token::lexer(r#"f"Hello {name}""#);
    assert_eq!(lex.next(), Some(Ok(Token::FString("Hello {name}".to_string()))));
}

#[test]
fn test_lex_raw_string() {
    let mut lex = Token::lexer(r#"r"C:\path\to\file""#);
    assert_eq!(lex.next(), Some(Ok(Token::RawString(r"C:\path\to\file".to_string()))));
}

// ==================== CHAR LITERAL TESTS ====================

#[test]
fn test_lex_char_simple() {
    let mut lex = Token::lexer("'a'");
    assert_eq!(lex.next(), Some(Ok(Token::Char('a'))));
}

#[test]
fn test_lex_char_escape_newline() {
    let mut lex = Token::lexer(r"'\n'");
    assert_eq!(lex.next(), Some(Ok(Token::Char('\n'))));
}

#[test]
fn test_lex_char_escape_tab() {
    let mut lex = Token::lexer(r"'\t'");
    assert_eq!(lex.next(), Some(Ok(Token::Char('\t'))));
}

#[test]
fn test_lex_char_escape_backslash() {
    let mut lex = Token::lexer(r"'\\'");
    assert_eq!(lex.next(), Some(Ok(Token::Char('\\'))));
}

#[test]
fn test_lex_char_escape_quote() {
    let mut lex = Token::lexer(r"'\''");
    assert_eq!(lex.next(), Some(Ok(Token::Char('\''))));
}

// ==================== BOOLEAN LITERAL TESTS ====================

#[test]
fn test_lex_bool_true() {
    let mut lex = Token::lexer("true");
    assert_eq!(lex.next(), Some(Ok(Token::Bool(true))));
}

#[test]
fn test_lex_bool_false() {
    let mut lex = Token::lexer("false");
    assert_eq!(lex.next(), Some(Ok(Token::Bool(false))));
}

// ==================== KEYWORD TESTS ====================

#[test]
fn test_lex_keyword_fun() {
    let mut lex = Token::lexer("fun");
    assert_eq!(lex.next(), Some(Ok(Token::Fun)));
}

#[test]
fn test_lex_keyword_fn() {
    let mut lex = Token::lexer("fn");
    assert_eq!(lex.next(), Some(Ok(Token::Fn)));
}

#[test]
fn test_lex_keyword_let() {
    let mut lex = Token::lexer("let");
    assert_eq!(lex.next(), Some(Ok(Token::Let)));
}

#[test]
fn test_lex_keyword_var() {
    let mut lex = Token::lexer("var");
    assert_eq!(lex.next(), Some(Ok(Token::Var)));
}

#[test]
fn test_lex_keyword_if() {
    let mut lex = Token::lexer("if");
    assert_eq!(lex.next(), Some(Ok(Token::If)));
}

#[test]
fn test_lex_keyword_else() {
    let mut lex = Token::lexer("else");
    assert_eq!(lex.next(), Some(Ok(Token::Else)));
}

#[test]
fn test_lex_keyword_match() {
    let mut lex = Token::lexer("match");
    assert_eq!(lex.next(), Some(Ok(Token::Match)));
}

#[test]
fn test_lex_keyword_for() {
    let mut lex = Token::lexer("for");
    assert_eq!(lex.next(), Some(Ok(Token::For)));
}

#[test]
fn test_lex_keyword_while() {
    let mut lex = Token::lexer("while");
    assert_eq!(lex.next(), Some(Ok(Token::While)));
}

#[test]
fn test_lex_keyword_return() {
    let mut lex = Token::lexer("return");
    assert_eq!(lex.next(), Some(Ok(Token::Return)));
}

#[test]
fn test_lex_keyword_break() {
    let mut lex = Token::lexer("break");
    assert_eq!(lex.next(), Some(Ok(Token::Break)));
}

#[test]
fn test_lex_keyword_continue() {
    let mut lex = Token::lexer("continue");
    assert_eq!(lex.next(), Some(Ok(Token::Continue)));
}

// ==================== OPERATOR TESTS ====================

#[test]
fn test_lex_operator_plus() {
    let mut lex = Token::lexer("+");
    assert_eq!(lex.next(), Some(Ok(Token::Plus)));
}

#[test]
fn test_lex_operator_minus() {
    let mut lex = Token::lexer("-");
    assert_eq!(lex.next(), Some(Ok(Token::Minus)));
}

#[test]
fn test_lex_operator_star() {
    let mut lex = Token::lexer("*");
    assert_eq!(lex.next(), Some(Ok(Token::Star)));
}

#[test]
fn test_lex_operator_slash() {
    let mut lex = Token::lexer("/");
    assert_eq!(lex.next(), Some(Ok(Token::Slash)));
}

#[test]
fn test_lex_operator_equals() {
    let mut lex = Token::lexer("=");
    assert_eq!(lex.next(), Some(Ok(Token::Equals)));
}

#[test]
fn test_lex_operator_equals_equals() {
    let mut lex = Token::lexer("==");
    assert_eq!(lex.next(), Some(Ok(Token::EqualsEquals)));
}

#[test]
fn test_lex_operator_not_equals() {
    let mut lex = Token::lexer("!=");
    assert_eq!(lex.next(), Some(Ok(Token::BangEquals)));
}

#[test]
fn test_lex_operator_less_than() {
    let mut lex = Token::lexer("<");
    assert_eq!(lex.next(), Some(Ok(Token::LessThan)));
}

#[test]
fn test_lex_operator_greater_than() {
    let mut lex = Token::lexer(">");
    assert_eq!(lex.next(), Some(Ok(Token::GreaterThan)));
}

#[test]
fn test_lex_operator_pipe() {
    let mut lex = Token::lexer("|");
    assert_eq!(lex.next(), Some(Ok(Token::Pipe)));
}

#[test]
fn test_lex_operator_ampersand() {
    let mut lex = Token::lexer("&");
    assert_eq!(lex.next(), Some(Ok(Token::Ampersand)));
}

// ==================== DELIMITER TESTS ====================

#[test]
fn test_lex_delimiter_left_paren() {
    let mut lex = Token::lexer("(");
    assert_eq!(lex.next(), Some(Ok(Token::LeftParen)));
}

#[test]
fn test_lex_delimiter_right_paren() {
    let mut lex = Token::lexer(")");
    assert_eq!(lex.next(), Some(Ok(Token::RightParen)));
}

#[test]
fn test_lex_delimiter_left_brace() {
    let mut lex = Token::lexer("{");
    assert_eq!(lex.next(), Some(Ok(Token::LeftBrace)));
}

#[test]
fn test_lex_delimiter_right_brace() {
    let mut lex = Token::lexer("}");
    assert_eq!(lex.next(), Some(Ok(Token::RightBrace)));
}

#[test]
fn test_lex_delimiter_left_bracket() {
    let mut lex = Token::lexer("[");
    assert_eq!(lex.next(), Some(Ok(Token::LeftBracket)));
}

#[test]
fn test_lex_delimiter_right_bracket() {
    let mut lex = Token::lexer("]");
    assert_eq!(lex.next(), Some(Ok(Token::RightBracket)));
}

#[test]
fn test_lex_delimiter_comma() {
    let mut lex = Token::lexer(",");
    assert_eq!(lex.next(), Some(Ok(Token::Comma)));
}

#[test]
fn test_lex_delimiter_semicolon() {
    let mut lex = Token::lexer(";");
    assert_eq!(lex.next(), Some(Ok(Token::Semicolon)));
}

#[test]
fn test_lex_delimiter_dot() {
    let mut lex = Token::lexer(".");
    assert_eq!(lex.next(), Some(Ok(Token::Dot)));
}

#[test]
fn test_lex_delimiter_colon() {
    let mut lex = Token::lexer(":");
    assert_eq!(lex.next(), Some(Ok(Token::Colon)));
}

// ==================== IDENTIFIER TESTS ====================

#[test]
fn test_lex_identifier_simple() {
    let mut lex = Token::lexer("variable");
    assert_eq!(lex.next(), Some(Ok(Token::Identifier("variable".to_string()))));
}

#[test]
fn test_lex_identifier_with_underscore() {
    let mut lex = Token::lexer("my_var");
    assert_eq!(lex.next(), Some(Ok(Token::Identifier("my_var".to_string()))));
}

#[test]
fn test_lex_identifier_with_numbers() {
    let mut lex = Token::lexer("var123");
    assert_eq!(lex.next(), Some(Ok(Token::Identifier("var123".to_string()))));
}

#[test]
fn test_lex_identifier_camel_case() {
    let mut lex = Token::lexer("myVariable");
    assert_eq!(lex.next(), Some(Ok(Token::Identifier("myVariable".to_string()))));
}

// ==================== WHITESPACE AND COMMENT TESTS ====================

#[test]
fn test_lex_skip_whitespace() {
    let mut lex = Token::lexer("  42  ");
    assert_eq!(lex.next(), Some(Ok(Token::Integer(42))));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_lex_skip_newlines() {
    let mut lex = Token::lexer("42\n\n43");
    assert_eq!(lex.next(), Some(Ok(Token::Integer(42))));
    assert_eq!(lex.next(), Some(Ok(Token::Integer(43))));
}

#[test]
fn test_lex_skip_line_comment() {
    let mut lex = Token::lexer("42 // this is a comment\n43");
    assert_eq!(lex.next(), Some(Ok(Token::Integer(42))));
    assert_eq!(lex.next(), Some(Ok(Token::Integer(43))));
}

#[test]
fn test_lex_skip_block_comment() {
    let mut lex = Token::lexer("42 /* block comment */ 43");
    assert_eq!(lex.next(), Some(Ok(Token::Integer(42))));
    assert_eq!(lex.next(), Some(Ok(Token::Integer(43))));
}

#[test]
fn test_lex_skip_multiline_block_comment() {
    let mut lex = Token::lexer("42 /* block\ncomment\nhere */ 43");
    assert_eq!(lex.next(), Some(Ok(Token::Integer(42))));
    assert_eq!(lex.next(), Some(Ok(Token::Integer(43))));
}

// ==================== COMPLEX EXPRESSION TESTS ====================

#[test]
fn test_lex_expression_arithmetic() {
    let mut lex = Token::lexer("1 + 2 * 3");
    assert_eq!(lex.next(), Some(Ok(Token::Integer(1))));
    assert_eq!(lex.next(), Some(Ok(Token::Plus)));
    assert_eq!(lex.next(), Some(Ok(Token::Integer(2))));
    assert_eq!(lex.next(), Some(Ok(Token::Star)));
    assert_eq!(lex.next(), Some(Ok(Token::Integer(3))));
}

#[test]
fn test_lex_function_call() {
    let mut lex = Token::lexer("print(\"hello\")");
    assert_eq!(lex.next(), Some(Ok(Token::Identifier("print".to_string()))));
    assert_eq!(lex.next(), Some(Ok(Token::LeftParen)));
    assert_eq!(lex.next(), Some(Ok(Token::String("hello".to_string()))));
    assert_eq!(lex.next(), Some(Ok(Token::RightParen)));
}

#[test]
fn test_lex_variable_declaration() {
    let mut lex = Token::lexer("let x = 42");
    assert_eq!(lex.next(), Some(Ok(Token::Let)));
    assert_eq!(lex.next(), Some(Ok(Token::Identifier("x".to_string()))));
    assert_eq!(lex.next(), Some(Ok(Token::Equals)));
    assert_eq!(lex.next(), Some(Ok(Token::Integer(42))));
}

// Run all tests with: cargo test lexer_tdd --test lexer_tdd