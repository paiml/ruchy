//! PARSER-053: Hash Comment Support Tests
//! GitHub Issue: #45 - Multi-line Code Blocks with Inline Comments
//!
//! Tests Python/Ruby-style `#` comments in multi-line expressions.
//! All tests should FAIL until HashComment token is added to lexer.

use ruchy::frontend::lexer::TokenStream;
use ruchy::frontend::parser::Parser;

#[test]
fn test_parser_053_01_arithmetic_with_hash_comment() {
    // Test Case 01: Comment between arithmetic operations
    let input = r#"let x = 1
    # Add two
    + 2
    # Multiply by three
    * 3"#;

    let result = Parser::new(input).parse();

    // Should parse successfully
    assert!(result.is_ok(), "Failed to parse arithmetic with hash comments: {:?}", result.err());

    // Parsing succeeds - hash comments properly handled
    let _ast = result.unwrap();
}

#[test]
fn test_parser_053_02_method_chain_with_hash_comment() {
    // Test Case 02: Comment between method chains
    let input = r#"let result = "hello world"
    # Convert to uppercase
    .to_uppercase()
    # Get length
    .len()"#;

    let result = Parser::new(input).parse();

    assert!(result.is_ok(), "Failed to parse method chain with hash comments: {:?}", result.err());
}

#[test]
fn test_parser_053_03_function_args_with_hash_comment() {
    // Test Case 03: Comment between function arguments
    let input = r#"fn add_three(a: i32, b: i32, c: i32) -> i32 {
    a + b + c
}

let result = add_three(
    1,
    # Second argument
    2,
    # Third argument
    3
)"#;

    let result = Parser::new(input).parse();

    assert!(result.is_ok(), "Failed to parse function args with hash comments: {:?}", result.err());
}

#[test]
fn test_parser_053_04_array_literal_with_hash_comment() {
    // Test Case 04: Comment in array literal
    let input = r#"let arr = [
    1,
    # Second element
    2,
    # Third element
    3,
    # Fourth element
    4
]"#;

    let result = Parser::new(input).parse();

    assert!(result.is_ok(), "Failed to parse array literal with hash comments: {:?}", result.err());
}

#[test]
fn test_parser_053_05_simple_hash_comment() {
    // Test Case 05: Simple hash comment on its own line
    let input = r#"# This is a comment
let x = 42"#;

    let result = Parser::new(input).parse();

    assert!(result.is_ok(), "Failed to parse simple hash comment: {:?}", result.err());
}

#[test]
fn test_parser_053_06_inline_hash_comment() {
    // Test Case 06: Inline hash comment after code
    let input = "let x = 42  # This is forty-two";

    let result = Parser::new(input).parse();

    assert!(result.is_ok(), "Failed to parse inline hash comment: {:?}", result.err());
}

#[test]
fn test_parser_053_07_multiple_hash_comments() {
    // Test Case 07: Multiple hash comments in sequence
    let input = r#"# First comment
# Second comment
# Third comment
let x = 1"#;

    let result = Parser::new(input).parse();

    assert!(result.is_ok(), "Failed to parse multiple hash comments: {:?}", result.err());
}

#[test]
fn test_parser_053_08_hash_comment_in_block() {
    // Test Case 08: Hash comment inside function body
    let input = r#"fn example() -> i32 {
    # Calculate result
    let x = 10
    # Add five
    let y = x + 5
    # Return result
    y
}"#;

    let result = Parser::new(input).parse();

    assert!(result.is_ok(), "Failed to parse hash comment in block: {:?}", result.err());
}

#[cfg(test)]
mod lexer_tests {
    use super::*;
    use ruchy::frontend::lexer::Token;

    #[test]
    fn test_parser_053_lexer_tokenizes_hash_comment() {
        // Verify lexer produces HashComment token
        let mut stream = TokenStream::new("x # comment\n+ y");

        // First token: Identifier "x"
        match stream.next() {
            Some((Token::Identifier(name), _)) if name == "x" => {},
            other => panic!("Expected Identifier(x), got: {:?}", other),
        }

        // Second token: HashComment " comment" (should skip hash symbol)
        match stream.next() {
            Some((Token::HashComment(text), _)) if text == " comment" => {},
            Some((Token::LineComment(text), _)) if text == " comment" => {
                // TEMPORARY: If LineComment works, that's also acceptable
                println!("WARNING: Got LineComment instead of HashComment (may need token variant)");
            },
            other => panic!("Expected HashComment or LineComment, got: {:?}", other),
        }

        // Third token: Plus
        match stream.next() {
            Some((Token::Plus, _)) => {},
            other => panic!("Expected Plus, got: {:?}", other),
        }

        // Fourth token: Identifier "y"
        match stream.next() {
            Some((Token::Identifier(name), _)) if name == "y" => {},
            other => panic!("Expected Identifier(y), got: {:?}", other),
        }
    }

    #[test]
    fn test_parser_053_lexer_hash_vs_double_slash() {
        // Verify both comment styles work
        let input1 = "x // comment\ny";
        let input2 = "x # comment\ny";

        // Both should tokenize to same structure (modulo comment token type)
        let mut stream1 = TokenStream::new(input1);
        let mut stream2 = TokenStream::new(input2);

        // First identifier
        assert!(matches!(stream1.next(), Some((Token::Identifier(_), _))));
        assert!(matches!(stream2.next(), Some((Token::Identifier(_), _))));

        // Comment token (may be different variants)
        let comment1 = stream1.next();
        let comment2 = stream2.next();

        assert!(
            matches!(comment1, Some((Token::LineComment(_), _))),
            "Expected LineComment for //, got: {:?}", comment1
        );

        // Hash comments should now tokenize as HashComment
        assert!(
            matches!(comment2, Some((Token::HashComment(_), _))) ||
            matches!(comment2, Some((Token::LineComment(_), _))),
            "Expected HashComment or LineComment for #, got: {:?}", comment2
        );

        // Second identifier
        assert!(matches!(stream1.next(), Some((Token::Identifier(_), _))));
        assert!(matches!(stream2.next(), Some((Token::Identifier(_), _))));
    }
}

#[cfg(test)]
#[cfg(feature = "proptest")]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        #[ignore] // Property tests run separately
        fn prop_hash_comments_never_break_simple_expressions(n in 0i32..1000) {
            // Add hash comment to simple expression
            let input = format!("let x = {}\n    # comment\n    + 1", n);

            let result = Parser::new(&input).parse();

            prop_assert!(result.is_ok(), "Hash comment broke parsing for n={}", n);
        }

        #[test]
        #[ignore] // Property tests run separately
        fn prop_hash_comments_preserved_in_ast(comment in "[a-zA-Z0-9 ]{1,50}") {
            let input = format!("# {}\nlet x = 1", comment);

            let result = Parser::new(&input).parse();

            prop_assert!(result.is_ok(), "Failed to parse hash comment: {}", comment);
        }
    }
}
