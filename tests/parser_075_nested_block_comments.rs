#![allow(missing_docs)]
//! PARSER-075: Nested block comments with depth tracking (GitHub Issue #58, Part 2/4)
//!
//! Tests Rust-style nested block comments: /* outer /* inner */ still outer */
//! Related: GitHub Issue #58 - Unary Plus Operator Support (comprehensive parser fixes)

use ruchy::frontend::lexer::{Token, TokenStream};

/// Helper to extract all tokens from source code
fn tokenize(source: &str) -> Vec<Token> {
    let mut stream = TokenStream::new(source);
    let mut tokens = Vec::new();
    while let Some((token, _span)) = stream.next() {
        tokens.push(token);
    }
    tokens
}

// ============================================================================
// Test Suite 1: Simple Block Comments (Regression)
// ============================================================================

#[test]
fn test_parser_075_01_simple_block_comment() {
    // Simple non-nested block comment should still work
    let source = "/* simple comment */";
    let tokens = tokenize(source);

    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::BlockComment(content) if content == " simple comment "));
}

#[test]
fn test_parser_075_01_block_comment_with_code() {
    // Block comment between code
    let source = "let x = /* comment */ 42";
    let tokens = tokenize(source);

    // Should have: Let, Identifier("x"), Equal, BlockComment, Integer("42")
    assert!(matches!(&tokens[0], Token::Let));
    assert!(matches!(&tokens[1], Token::Identifier(name) if name == "x"));
    assert!(matches!(&tokens[2], Token::Equal));
    assert!(matches!(&tokens[3], Token::BlockComment(content) if content == " comment "));
    assert!(matches!(&tokens[4], Token::Integer(n) if n == "42"));
}

#[test]
fn test_parser_075_01_empty_block_comment() {
    // Empty block comment
    let source = "/**/";
    let tokens = tokenize(source);

    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::BlockComment(content) if content.is_empty()));
}

#[test]
fn test_parser_075_01_multiline_block_comment() {
    // Multiline block comment
    let source = r"/*
 * This is a
 * multiline comment
 */";
    let tokens = tokenize(source);

    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::BlockComment(content) if content.contains("multiline")));
}

// ============================================================================
// Test Suite 2: Single-Level Nested Comments
// ============================================================================

#[test]
fn test_parser_075_02_single_nested_comment() {
    // Single level nesting: /* outer /* inner */ still outer */
    let source = "/* outer /* inner */ still outer */";
    let tokens = tokenize(source);

    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::BlockComment(content)
        if content.contains("outer") && content.contains("inner") && content.contains("still outer")));
}

#[test]
fn test_parser_075_02_nested_at_start() {
    // Nested comment at the start
    let source = "/* /* nested */ rest */";
    let tokens = tokenize(source);

    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::BlockComment(content)
        if content.contains("/* nested */ rest ")));
}

#[test]
fn test_parser_075_02_nested_at_end() {
    // Nested comment at the end
    let source = "/* start /* nested */ */";
    let tokens = tokenize(source);

    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::BlockComment(content)
        if content.contains("start /* nested */")));
}

#[test]
fn test_parser_075_02_multiple_nested_siblings() {
    // Multiple nested comments as siblings (not further nested)
    let source = "/* first /* a */ middle /* b */ end */";
    let tokens = tokenize(source);

    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::BlockComment(content)
        if content.contains("first") && content.contains("middle") && content.contains("end")));
}

// ============================================================================
// Test Suite 3: Multi-Level Nested Comments
// ============================================================================

#[test]
fn test_parser_075_03_triple_nested() {
    // Three levels of nesting
    let source = "/* level1 /* level2 /* level3 */ back2 */ back1 */";
    let tokens = tokenize(source);

    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::BlockComment(content)
        if content.contains("level1")
        && content.contains("level2")
        && content.contains("level3")
        && content.contains("back1")));
}

#[test]
fn test_parser_075_03_deep_nesting() {
    // Five levels deep
    let source = "/* 1 /* 2 /* 3 /* 4 /* 5 */ 4 */ 3 */ 2 */ 1 */";
    let tokens = tokenize(source);

    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::BlockComment(_)));
}

// ============================================================================
// Test Suite 4: Comments in Real Code Context
// ============================================================================

#[test]
fn test_parser_075_04_commented_out_code_with_nesting() {
    // Commenting out code that itself contains comments
    let source = r"
        let active = 42;
        /* temporarily disabled
        let disabled = /* old value */ 99;
        */
        let also_active = 7;
    ";
    let tokens = tokenize(source);

    // Should have tokens for: let, active, =, 42, ;, BlockComment, let, also_active, =, 7, ;
    let block_comments: Vec<_> = tokens.iter()
        .filter(|t| matches!(t, Token::BlockComment(_)))
        .collect();

    assert_eq!(block_comments.len(), 1, "Should have exactly one block comment");

    if let Token::BlockComment(content) = &block_comments[0] {
        assert!(content.contains("disabled"));
        assert!(content.contains("/* old value */"));
    }
}

#[test]
fn test_parser_075_04_nested_comment_preserves_structure() {
    // Verify the exact content is preserved including nested comment markers
    let source = "/* outer /* inner */ outer */";
    let tokens = tokenize(source);

    assert_eq!(tokens.len(), 1);
    if let Token::BlockComment(content) = &tokens[0] {
        assert_eq!(content, " outer /* inner */ outer ");
    } else {
        panic!("Expected BlockComment token");
    }
}

// ============================================================================
// Test Suite 5: Edge Cases
// ============================================================================

#[test]
fn test_parser_075_05_star_not_followed_by_slash() {
    // Asterisks that aren't part of comment delimiters
    let source = "/* 2 * 3 = 6 */";
    let tokens = tokenize(source);

    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::BlockComment(content) if content == " 2 * 3 = 6 "));
}

#[test]
fn test_parser_075_05_slash_not_followed_by_star() {
    // Slashes that aren't part of comment delimiters
    let source = "/* division: a/b */";
    let tokens = tokenize(source);

    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::BlockComment(content) if content.contains("a/b")));
}

#[test]
fn test_parser_075_05_unclosed_comment_error_recovery() {
    // Unclosed block comment - should consume to end of input
    let source = "/* unclosed comment";
    let tokens = tokenize(source);

    // Should produce a BlockComment token (error recovery)
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::BlockComment(content) if content.contains("unclosed")));
}

#[test]
fn test_parser_075_05_unclosed_nested_comment() {
    // Nested comment that's never closed
    let source = "/* outer /* inner";
    let tokens = tokenize(source);

    // Should consume everything (error recovery)
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::BlockComment(_)));
}

#[test]
fn test_parser_075_05_consecutive_block_comments() {
    // Multiple block comments in sequence
    let source = "/* first */ /* second */ /* third */";
    let tokens = tokenize(source);

    assert_eq!(tokens.len(), 3);
    assert!(matches!(&tokens[0], Token::BlockComment(c) if c == " first "));
    assert!(matches!(&tokens[1], Token::BlockComment(c) if c == " second "));
    assert!(matches!(&tokens[2], Token::BlockComment(c) if c == " third "));
}

// ============================================================================
// Test Suite 6: Integration with Other Token Types
// ============================================================================

#[test]
fn test_parser_075_06_block_comment_with_strings() {
    // Block comment containing string-like content
    let source = r#"/* "this looks like a string" */"#;
    let tokens = tokenize(source);

    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::BlockComment(content)
        if content.contains(r#""this looks like a string""#)));
}

#[test]
fn test_parser_075_06_block_comment_with_line_comment_syntax() {
    // Block comment containing line comment syntax
    let source = "/* this // looks like line comment */";
    let tokens = tokenize(source);

    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::BlockComment(content) if content.contains("//")));
}

#[test]
fn test_parser_075_06_mixed_comment_types() {
    // Mix of line and block comments
    let source = r"
        // line comment
        /* block comment */
        let x = 42; // another line comment
    ";
    let tokens = tokenize(source);

    let line_comments: Vec<_> = tokens.iter()
        .filter(|t| matches!(t, Token::LineComment(_)))
        .collect();
    let block_comments: Vec<_> = tokens.iter()
        .filter(|t| matches!(t, Token::BlockComment(_)))
        .collect();

    assert_eq!(line_comments.len(), 2);
    assert_eq!(block_comments.len(), 1);
}
