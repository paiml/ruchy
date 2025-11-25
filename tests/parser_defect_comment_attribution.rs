#![allow(missing_docs)]
#![allow(clippy::needless_range_loop, clippy::manual_let_else)]
// PARSER DEFECT: Standalone comments wrongly attributed as trailing comments
// Toyota Way: Stop the line, fix root cause with Extreme TDD
//
// BUG DESCRIPTION:
// When a comment appears on its own line (with blank line before it),
// the parser incorrectly attaches it as a trailing_comment to the previous
// expression instead of as a leading_comment to the next expression.
//
// IMPACT: Breaks ignore directives and comment preservation in formatter
//
// ROOT CAUSE: consume_trailing_comment() doesn't verify comment is on same line

use ruchy::frontend::ast::ExprKind;
use ruchy::Parser as RuchyParser;

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// PROPERTY: Standalone comments (with blank line before) are NEVER trailing comments
        #[test]
        #[ignore = "Property tests run with: cargo test property_tests -- --ignored"]
        fn prop_standalone_comments_never_trailing(code1 in "[a-z]+", code2 in "[a-z]+") {
            let source = format!("let {code1} = 1\n\n// comment\nlet {code2} = 2");
            let mut parser = RuchyParser::new(&source);

            if let Ok(ast) = parser.parse() {
                if let ExprKind::Block(exprs) = &ast.kind {
                    if exprs.len() >= 2 {
                        // First expression should have NO trailing comment
                        prop_assert!(exprs[0].trailing_comment.is_none(),
                            "Standalone comment (with blank line) should not be trailing");

                        // Second expression should have leading comment
                        prop_assert!(!exprs[1].leading_comments.is_empty(),
                            "Standalone comment should be leading of next expression");
                    }
                }
            }
        }

        /// PROPERTY: Comments on same line (no newline between) ARE trailing comments
        #[test]
        #[ignore = "Property tests run with: cargo test property_tests -- --ignored"]
        fn prop_inline_comments_are_trailing(var in "[a-z]+", val in 1..100i32, comment in "[a-zA-Z ]+") {
            let source = format!("let {var} = {val} // {comment}");
            let mut parser = RuchyParser::new(&source);

            if let Ok(ast) = parser.parse() {
                if let ExprKind::Let { value, .. } = &ast.kind {
                    // Value expression should have trailing comment
                    prop_assert!(value.trailing_comment.is_some(),
                        "Inline comment should be trailing comment on value");
                }
            }
        }

        /// PROPERTY: Multiple standalone comments are each leading comments for their following expression
        #[test]
        #[ignore = "Property tests run with: cargo test property_tests -- --ignored"]
        fn prop_multiple_standalone_comments(count in 2..5usize) {
            let mut source = String::new();
            for i in 0..count {
                if i > 0 {
                    source.push_str("\n\n// comment\n");
                }
                source.push_str(&format!("let v{i} = {i}"));
            }

            let mut parser = RuchyParser::new(&source);
            if let Ok(ast) = parser.parse() {
                if let ExprKind::Block(exprs) = &ast.kind {
                    if exprs.len() == count {
                        // First has no comments
                        prop_assert!(exprs[0].leading_comments.is_empty());
                        prop_assert!(exprs[0].trailing_comment.is_none());

                        // All others have leading comment
                        for i in 1..count {
                            prop_assert_eq!(exprs[i].leading_comments.len(), 1,
                                "Expression {} should have 1 leading comment", i);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_standalone_comment_should_be_leading_not_trailing() {
    let source = r"let a = 1

// This comment is standalone on its own line
let b = 2";

    let mut parser = RuchyParser::new(source);
    let ast = parser.parse().expect("Should parse successfully");

    // Top level should be a Block with 2 expressions
    if let ExprKind::Block(exprs) = &ast.kind {
        assert_eq!(exprs.len(), 2, "Should have 2 expressions");

        let first_let = &exprs[0];
        let second_let = &exprs[1];

        // CRITICAL: Standalone comment should be LEADING comment of second let
        assert!(
            second_let.leading_comments.len() == 1,
            "Second let should have 1 leading comment, got: {}",
            second_let.leading_comments.len()
        );

        // Standalone comment should NOT be trailing comment of first let
        assert!(
            first_let.trailing_comment.is_none(),
            "First let should NOT have trailing comment (comment is on separate line)"
        );

        // Verify the comment text
        let comment_text = match &second_let.leading_comments[0].kind {
            ruchy::frontend::ast::CommentKind::Line(text) => text,
            _ => panic!("Expected line comment"),
        };
        assert!(
            comment_text.contains("This comment is standalone"),
            "Comment text should be preserved"
        );
    } else {
        panic!("Expected Block at top level, got: {:?}", ast.kind);
    }
}

#[test]
fn test_trailing_comment_on_same_line_is_trailing() {
    let source = r"let a = 1 // trailing comment on same line
let b = 2";

    let mut parser = RuchyParser::new(source);
    let ast = parser.parse().expect("Should parse successfully");

    if let ExprKind::Block(exprs) = &ast.kind {
        let first_let = &exprs[0];
        let second_let = &exprs[1];

        // Comment on same line SHOULD be trailing on the VALUE expression
        // (not the Let expression itself)
        if let ExprKind::Let { value, .. } = &first_let.kind {
            assert!(
                value.trailing_comment.is_some(),
                "Comment on same line should be trailing comment on value"
            );
        } else {
            panic!("Expected Let expression");
        }

        // Second let should have NO leading comment
        assert!(
            second_let.leading_comments.is_empty(),
            "Second let should have no leading comments"
        );
    } else {
        panic!("Expected Block at top level");
    }
}

#[test]
fn test_multiple_standalone_comments() {
    let source = r"let a = 1

// First standalone comment
let b = 2

// Second standalone comment
let c = 3";

    let mut parser = RuchyParser::new(source);
    let ast = parser.parse().expect("Should parse successfully");

    if let ExprKind::Block(exprs) = &ast.kind {
        assert_eq!(exprs.len(), 3, "Should have 3 expressions");

        // First let: no comments
        assert!(exprs[0].leading_comments.is_empty());
        assert!(exprs[0].trailing_comment.is_none());

        // Second let: should have leading comment
        assert_eq!(exprs[1].leading_comments.len(), 1);
        assert!(exprs[1].trailing_comment.is_none());

        // Third let: should have leading comment
        assert_eq!(exprs[2].leading_comments.len(), 1);
        assert!(exprs[2].trailing_comment.is_none());
    } else {
        panic!("Expected Block at top level");
    }
}

#[test]
fn test_ignore_directive_as_standalone_comment() {
    // This is the actual use case that's broken
    let source = r"let a = 1

// ruchy-fmt-ignore
let b    =    2";

    let mut parser = RuchyParser::new(source);
    let ast = parser.parse().expect("Should parse successfully");

    if let ExprKind::Block(exprs) = &ast.kind {
        let second_let = &exprs[1];

        // Ignore directive MUST be leading comment to work
        assert!(
            !second_let.leading_comments.is_empty(),
            "Ignore directive must be leading comment for formatter to work"
        );

        let comment_text = match &second_let.leading_comments[0].kind {
            ruchy::frontend::ast::CommentKind::Line(text) => text,
            _ => panic!("Expected line comment"),
        };

        assert!(
            comment_text.trim() == "ruchy-fmt-ignore",
            "Should preserve ignore directive text"
        );
    } else {
        panic!("Expected Block at top level");
    }
}
