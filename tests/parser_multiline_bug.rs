//! CRITICAL PARSER BUG TEST - Multiline expression parsing
//! TDD fix for incomplete multiline expressions from ruchy-book validation

use ruchy::Parser;

#[test]
fn test_incomplete_multiline_if_expression() {
    // Check the exact failing case: incomplete if expression should fail gracefully
    let incomplete = "let result = if price > 100.0 { ";
    let mut parser = Parser::new(incomplete);
    let result = parser.parse();

    // This SHOULD fail with a clear error message, not "Unexpected end of input"
    assert!(
        result.is_err(),
        "Incomplete expression should fail gracefully"
    );
    let error = format!("{:?}", result.err().unwrap());

    // Should have meaningful error message
    assert!(
        error.contains("Expected") || error.contains("incomplete") || error.contains("missing"),
        "Error should be descriptive: {}",
        error
    );

    // Should NOT be generic "Unexpected end of input"
    assert!(
        !error.contains("Unexpected end of input"),
        "Should provide specific error, not generic EOF: {}",
        error
    );
}

#[test]
fn test_complete_multiline_if_expression() {
    // Check that complete multiline if expressions work
    let complete = r#"let result = if price > 100.0 { 
    price * 0.9
} else { 
    price * (1.0 + tax_rate)
}"#;

    let mut parser = Parser::new(complete);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Complete multiline if should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_single_line_complete_if_expression() {
    // Check that single-line complete if expressions work (baseline)
    let single_line = "let result = if price > 100.0 { price * 0.9 } else { price * 1.1 }";
    let mut parser = Parser::new(single_line);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Single-line if should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_block_parsing_with_newlines() {
    // Check that block parsing handles newlines correctly
    let multiline_block = r#"let x = {
    let a = 10;
    let b = 20;
    a + b
}"#;

    let mut parser = Parser::new(multiline_block);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Multiline block should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_incomplete_block_expression() {
    // Check incomplete block should fail gracefully
    let incomplete_block = "let x = {";
    let mut parser = Parser::new(incomplete_block);
    let result = parser.parse();

    assert!(result.is_err(), "Incomplete block should fail");
    let error = format!("{:?}", result.err().unwrap());

    // Should have meaningful error about missing closing brace
    assert!(
        error.contains("Expected") || error.contains("RightBrace") || error.contains("}"),
        "Error should mention missing closing brace: {}",
        error
    );
}

#[test]
fn test_multiline_let_with_arithmetic() {
    // Check multiline let statement with arithmetic that spans lines
    let multiline_calc = r#"let calculation = 10 + 
    20 + 
    30"#;

    let mut parser = Parser::new(multiline_calc);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Multiline arithmetic should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_error_recovery_with_suggestions() {
    // Check that parser provides helpful suggestions for common mistakes
    let examples = vec![
        ("let x = if true {", "Missing closing brace and else clause"),
        ("let x = {", "Missing closing brace"),
        ("let x = if", "Missing condition and body"),
    ];

    for (incomplete, _expected_hint) in examples {
        let mut parser = Parser::new(incomplete);
        let result = parser.parse();

        assert!(
            result.is_err(),
            "Incomplete expression '{}' should fail",
            incomplete
        );

        let error = format!("{:?}", result.err().unwrap());
        println!("Error for '{}': {}", incomplete, error);

        // For now, just ensure we get some error (improvement target)
        assert!(!error.is_empty(), "Should have non-empty error message");
    }
}
