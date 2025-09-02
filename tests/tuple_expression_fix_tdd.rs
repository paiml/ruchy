// Test to fix tuple expression parsing

#[test]
fn test_tuple_expression_parsing_fix() {
    // The issue is that (x, y) should parse as a tuple expression
    // but currently LeftParen parsing only handles grouped expressions
    
    // Let's verify that tuple literals work differently
    let source = "(1, 2)";  // numeric tuple
    let mut parser = ruchy::frontend::parser::Parser::new(source);
    let result = parser.parse();
    
    match result {
        Ok(expr) => {
            println!("Successfully parsed numeric tuple: {:?}", expr.kind);
        }
        Err(e) => {
            println!("Failed to parse numeric tuple: {}", e);
            // This tells us if the issue is with comma parsing or identifier parsing
        }
    }
}

#[test] 
fn test_identifier_comma_sequence() {
    // Test if the issue is specific to identifiers in parentheses
    let source = "(a)";  // single identifier in parens
    let mut parser = ruchy::frontend::parser::Parser::new(source);
    let result = parser.parse();
    
    match result {
        Ok(_) => println!("Single identifier in parens works"),
        Err(e) => println!("Single identifier in parens fails: {}", e),
    }
}