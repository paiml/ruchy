// PARSER-054: Inline comments after semicolons cause parse failures
// GitHub Issue: https://github.com/paiml/ruchy/issues/TBD
// Priority: HIGH - Blocks 20% of book content (49+ blocks)

#[cfg(test)]
mod parser_054_tests {
    use ruchy::frontend::parser::Parser;

    #[test]
    fn test_parser_054_inline_comment_after_statement() {
        // RED PHASE: This test SHOULD FAIL with current implementation
        // From interactive.paiml.com book - chapter02_00.md block 5
        let input = r"
fun main() {
    let x = 10;
    println(x);  // Output: 10
}
";

        let mut parser = Parser::new(input);
        let result = parser.parse();

        assert!(
            result.is_ok(),
            "Parser should handle inline comments after semicolons, but got: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parser_054_multiple_statements_with_comments() {
        let input = r"
fun main() {
    let value1 = 10;  // First value
    let value2 = 20;  // Second value
    let result = value1 + value2;  // Compute sum
    println(result);  // Output: 30
}
";

        let mut parser = Parser::new(input);
        let result = parser.parse();

        assert!(
            result.is_ok(),
            "Parser should handle multiple inline comments, but got: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parser_054_block_comment_after_semicolon() {
        let input = r"
fun main() {
    let x = 10; /* block comment */
    println(x);
}
";

        let mut parser = Parser::new(input);
        let result = parser.parse();

        assert!(
            result.is_ok(),
            "Parser should handle block comments after semicolons, but got: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parser_054_no_comment_still_works() {
        // Regression test - ensure fix doesn't break existing behavior
        let input = r"
fun main() {
    let x = 10;
    let y = 20;
    println(x + y);
}
";

        let mut parser = Parser::new(input);
        let result = parser.parse();

        assert!(
            result.is_ok(),
            "Parser should still work without comments, but got: {:?}",
            result.err()
        );
    }
}
