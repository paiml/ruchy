#![allow(missing_docs)]
// PARSER DEFECT [PARSER-054]: Multiple leading comments in block not preserved
// Toyota Way: STOP THE LINE - No bug is out of scope
//
// BUG DESCRIPTION:
// Parser fails to preserve leading comments on statements after the first statement
// in a block. Only the FIRST statement's leading comments are captured.
//
// Example that SHOULD work but FAILS:
//   // comment 1
//   let a = 1
//
//   // comment 2
//   let b = 2
//
// ACTUAL: Only "comment 1" appears in AST
// EXPECTED: Both "comment 1" and "comment 2" in AST
//
// IMPACT: Breaks formatter ignore directives when used multiple times in a file
//
// ROOT CAUSE: Parser's consume_leading_comments() likely only called for first statement

use ruchy::Parser as RuchyParser;
use ruchy::frontend::ast::{ExprKind, CommentKind};

#[test]
fn test_parse_multiple_leading_comments_in_block() {
    let source = r"// comment 1
let a = 1

// comment 2
let b = 2";

    let mut parser = RuchyParser::new(source);
    let result = parser.parse();

    assert!(result.is_ok(), "Should parse multiple leading comments: {:?}", result.err());

    // Verify AST contains BOTH comments
    if let Ok(ast) = result {
        if let ExprKind::Block(stmts) = &ast.kind {
            assert_eq!(stmts.len(), 2, "Should have 2 statements");

            // First statement should have "comment 1"
            assert_eq!(stmts[0].leading_comments.len(), 1, "First statement should have 1 leading comment");
            if let CommentKind::Line(text) = &stmts[0].leading_comments[0].kind {
                assert!(text.contains("comment 1"), "First comment should contain 'comment 1'");
            } else {
                panic!("Expected Line comment");
            }

            // Second statement should have "comment 2" (THIS IS THE BUG)
            assert_eq!(stmts[1].leading_comments.len(), 1, "Second statement should have 1 leading comment");
            if let CommentKind::Line(text) = &stmts[1].leading_comments[0].kind {
                assert!(text.contains("comment 2"), "Second comment should contain 'comment 2'");
            } else {
                panic!("Expected Line comment");
            }
        } else {
            panic!("Expected Block expression at top level, got: {:?}", ast.kind);
        }
    }
}

#[test]
fn test_parse_three_leading_comments_in_block() {
    let source = r"// comment 1
let a = 1

// comment 2
let b = 2

// comment 3
let c = 3";

    let mut parser = RuchyParser::new(source);
    let result = parser.parse();

    assert!(result.is_ok(), "Should parse three leading comments: {:?}", result.err());

    // Verify AST contains ALL THREE comments
    if let Ok(ast) = result {
        if let ExprKind::Block(stmts) = &ast.kind {
            assert_eq!(stmts.len(), 3, "Should have 3 statements");

            assert_eq!(stmts[0].leading_comments.len(), 1, "First statement should have 1 leading comment");
            assert_eq!(stmts[1].leading_comments.len(), 1, "Second statement should have 1 leading comment");
            assert_eq!(stmts[2].leading_comments.len(), 1, "Third statement should have 1 leading comment");
        } else {
            panic!("Expected Block expression at top level");
        }
    }
}

#[test]
fn test_ignore_directives_preserved_in_ast() {
    // This is the ACTUAL failing case from test_fmt_ignore_multiple_expressions
    let source = r"// ruchy-fmt-ignore
let a    =    1

let b = 2

// ruchy-fmt-ignore
let c    =    3

let d = 4";

    let mut parser = RuchyParser::new(source);
    let result = parser.parse();

    assert!(result.is_ok(), "Should parse multiple ignore directives: {:?}", result.err());

    if let Ok(ast) = result {
        if let ExprKind::Block(stmts) = &ast.kind {
            assert_eq!(stmts.len(), 4, "Should have 4 statements");

            // First ignore directive (on let a)
            assert_eq!(stmts[0].leading_comments.len(), 1, "First let should have ignore directive");
            if let CommentKind::Line(text) = &stmts[0].leading_comments[0].kind {
                assert!(text.contains("ruchy-fmt-ignore"), "First comment should be ignore directive");
            } else {
                panic!("Expected Line comment");
            }

            // Second ignore directive (on let c) - THIS IS THE BUG
            assert_eq!(stmts[2].leading_comments.len(), 1, "Third let should have ignore directive");
            if let CommentKind::Line(text) = &stmts[2].leading_comments[0].kind {
                assert!(text.contains("ruchy-fmt-ignore"), "Third statement should have ignore directive");
            } else {
                panic!("Expected Line comment");
            }
        } else {
            panic!("Expected Block expression at top level");
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// PROPERTY: All leading comments should be preserved regardless of position in block
        #[test]
        #[ignore = "Property tests run with: cargo test property_tests -- --ignored"]
        fn prop_all_leading_comments_preserved(count in 2..10usize) {
            let mut source = String::new();
            for i in 0..count {
                source.push_str(&format!("// comment {i}\nlet x{i} = {i}\n\n"));
            }

            let mut parser = RuchyParser::new(&source);
            let result = parser.parse();

            prop_assert!(result.is_ok(), "Should parse {} statements with comments: {:?}", count, result.err());

            if let Ok(ast) = result {
                if let ExprKind::Block(stmts) = &ast.kind {
                    prop_assert_eq!(stmts.len(), count, "Should have {} statements", count);

                    // Every statement should have exactly 1 leading comment
                    for (i, stmt) in stmts.iter().enumerate() {
                        prop_assert_eq!(stmt.leading_comments.len(), 1,
                                      "Statement {} should have 1 leading comment", i);
                    }
                }
            }
        }
    }
}
