// DEFECT-PARSER-006: Attributes inside block bodies
// Root cause: parse_next_block_expression() doesn't call parse_attributes()
// Impact: 9 book examples failing with "Unexpected token: AttributeStart"

#[cfg(test)]
mod defect_parser_006_tests {
    use ruchy::frontend::parser::Parser;

    #[test]
    fn test_defect_parser_006_attribute_in_macro_block() {
        // RED PHASE: This test SHOULD FAIL with current implementation
        // From interactive.paiml.com book - chapter2.md block 11
        let input = r#"
proptest! {
    #[test]
    fn test_example(n: i64) {
        let x = n;
    }
}
"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();

        assert!(
            result.is_ok(),
            "Parser should handle attributes inside macro blocks, but got: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_defect_parser_006_multiple_attributes_in_block() {
        let input = r#"
proptest! {
    #[test]
    fn test_one() {
        let x = 1;
    }

    #[test]
    fn test_two() {
        let y = 2;
    }
}
"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();

        assert!(
            result.is_ok(),
            "Parser should handle multiple attributes in macro block, but got: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_defect_parser_006_attribute_in_regular_block() {
        // Ensure attributes work inside any { } block, not just macros
        let input = r#"
{
    #[test]
    fn inner_test() {
        let x = 42;
    }
}
"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();

        assert!(
            result.is_ok(),
            "Parser should handle attributes in regular blocks, but got: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_defect_parser_006_top_level_attribute_still_works() {
        // Regression test - ensure top-level attributes still work
        let input = r#"
#[test]
fn test_example() {
    let x = 42;
}
"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();

        assert!(
            result.is_ok(),
            "Top-level attributes should still work, but got: {:?}",
            result.err()
        );
    }
}
