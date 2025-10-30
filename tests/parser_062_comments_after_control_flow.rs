#![allow(missing_docs)]
//! Tests for PARSER-062: Comments after control flow statements
//!
//! Bug: Inline comments after break/continue/return cause parser failures
//! Root cause: Comment tokens not skipped when checking for terminators
//!
//! Example failure:
//! ```ruchy
//! for n in [1, 2, 3] {
//!     if n > 1 {
//!         break  // Comment here
//!     }
//! }
//! ```
//! Error: "Expected body after for iterator: Expected `RightBrace`, found If"

use ruchy::Parser;

#[test]
fn test_parser_062_break_with_inline_comment() {
    let input = r"
for n in [1, 2, 3] {
    if n > 1 {
        break  // Exit loop
    }
}
";
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Parser should handle inline comments after break: {:?}",
        result.err()
    );
}

#[test]
fn test_parser_062_continue_with_inline_comment() {
    let input = r"
for n in [1, 2, 3] {
    if n == 2 {
        continue  // Skip this iteration
    }
    print(n)
}
";
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Parser should handle inline comments after continue: {:?}",
        result.err()
    );
}

#[test]
fn test_parser_062_return_with_inline_comment() {
    let input = r#"
fun test() {
    if true {
        return  // Early exit
    }
    print("unreachable")
}
"#;
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Parser should handle inline comments after return: {:?}",
        result.err()
    );
}

#[test]
fn test_parser_062_full_book_example() {
    let input = r"
let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
let has_large = false

for n in numbers {
  if n > 100 {
    has_large = true
    break  // Exit early, no need to check rest
  }
}

has_large  // Returns: false
";
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Parser should handle full book example with comments: {:?}",
        result.err()
    );
}

#[test]
fn test_parser_062_multiple_comments_in_nested_blocks() {
    let input = r"
for i in 0..10 {
    for j in 0..10 {
        if i == j {
            continue  // Skip diagonal
        }
        if i + j > 15 {
            break  // Exit inner loop
        }
        print(i, j)
    }
}
";
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Parser should handle multiple comments in nested blocks: {:?}",
        result.err()
    );
}
