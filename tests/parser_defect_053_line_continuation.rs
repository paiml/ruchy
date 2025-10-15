// PARSER DEFECT [PARSER-053]: Line continuations with intervening comments
// Toyota Way: STOP THE LINE - No bug is out of scope
//
// BUG DESCRIPTION:
// Parser fails with "Unexpected token: Plus" when a binary operation
// spans multiple lines with comments between the lines.
//
// Example that SHOULD work but FAILS:
//   let x = 1 + 2   // comment
//       // inner comment
//       + 3
//
// IMPACT: Breaks formatter ignore directives and multi-line expressions
//
// ROOT CAUSE: Parser treats "+ 3" as a new statement instead of continuation

use ruchy::Parser as RuchyParser;
use ruchy::frontend::ast::ExprKind;

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// PROPERTY: Line continuations with comments should parse successfully
        #[test]
        #[ignore = "Property tests run with: cargo test property_tests -- --ignored"]
        fn prop_line_continuation_with_comment_parses(val1 in 1..100i32, val2 in 1..100i32) {
            let source = format!("let x = {} + {}\n    // comment\n    + {}", val1, val2, val1);
            let mut parser = RuchyParser::new(&source);
            let result = parser.parse();

            prop_assert!(result.is_ok(), "Should parse line continuation with comment: {:?}", result.err());
        }

        /// PROPERTY: Multiple line continuations should parse successfully
        #[test]
        #[ignore = "Property tests run with: cargo test property_tests -- --ignored"]
        fn prop_multiple_line_continuations_parse(count in 2..5usize) {
            let mut source = "let x = 1".to_string();
            for i in 2..=count {
                source.push_str(&format!("\n    + {}", i));
            }

            let mut parser = RuchyParser::new(&source);
            let result = parser.parse();

            prop_assert!(result.is_ok(), "Should parse {} line continuations: {:?}", count, result.err());
        }

        /// PROPERTY: Line continuations with multiple comments should parse
        #[test]
        #[ignore = "Property tests run with: cargo test property_tests -- --ignored"]
        fn prop_line_continuation_multiple_comments(val1 in 1..100i32, val2 in 1..100i32, val3 in 1..100i32) {
            let source = format!("let x = {}\n    // comment 1\n    + {}\n    // comment 2\n    + {}", val1, val2, val3);
            let mut parser = RuchyParser::new(&source);
            let result = parser.parse();

            prop_assert!(result.is_ok(), "Should parse line continuation with multiple comments: {:?}", result.err());
        }
    }
}

#[test]
fn test_parse_line_continuation_without_comment() {
    // This SHOULD work (baseline)
    let source = r#"let x = 1 + 2
    + 3"#;

    let mut parser = RuchyParser::new(source);
    let result = parser.parse();

    assert!(result.is_ok(), "Should parse line continuation without comment");
}

#[test]
fn test_parse_line_continuation_with_trailing_comment() {
    // This SHOULD work - trailing comment on first line
    let source = r#"let x = 1 + 2   // trailing comment
    + 3"#;

    let mut parser = RuchyParser::new(source);
    let result = parser.parse();

    assert!(result.is_ok(), "Should parse line continuation with trailing comment: {:?}", result.err());
}

#[test]
fn test_parse_line_continuation_with_intervening_comment() {
    // This CURRENTLY FAILS but SHOULD work
    let source = r#"let x = 1 + 2
    // inner comment
    + 3"#;

    let mut parser = RuchyParser::new(source);
    let result = parser.parse();

    assert!(result.is_ok(), "Should parse line continuation with intervening comment: {:?}", result.err());

    // Verify the AST is correct
    if let Ok(ast) = result {
        if let ExprKind::Let { value, .. } = &ast.kind {
            // Value should be: 1 + 2 + 3 (nested binary ops)
            match &value.kind {
                ExprKind::Binary { .. } => {
                    // Good - it's a binary operation
                }
                other => panic!("Expected binary operation, got: {:?}", other),
            }
        } else {
            panic!("Expected Let expression at top level");
        }
    }
}

#[test]
fn test_parse_line_continuation_with_both_comments() {
    // This CURRENTLY FAILS but SHOULD work
    let source = r#"let x = 1 + 2   // trailing comment
    // inner comment
    + 3"#;

    let mut parser = RuchyParser::new(source);
    let result = parser.parse();

    assert!(result.is_ok(), "Should parse line continuation with both comments: {:?}", result.err());
}

#[test]
fn test_parse_multiple_line_continuations() {
    // This SHOULD work - multiple continuations
    let source = r#"let x = 1 + 2
    + 3
    + 4"#;

    let mut parser = RuchyParser::new(source);
    let result = parser.parse();

    assert!(result.is_ok(), "Should parse multiple line continuations: {:?}", result.err());
}

#[test]
fn test_parse_line_continuation_with_multiple_comments() {
    // This CURRENTLY FAILS but SHOULD work
    let source = r#"let x = 1 + 2
    // comment 1
    + 3
    // comment 2
    + 4"#;

    let mut parser = RuchyParser::new(source);
    let result = parser.parse();

    assert!(result.is_ok(), "Should parse line continuation with multiple comments: {:?}", result.err());
}
