use ruchy::frontend::ast::Pattern;

#[test]
fn test_extract_name_from_pattern() {
    // Test helper function to extract name from pattern
    let pattern = Pattern::Identifier("x".to_string());
    let name = extract_name_from_pattern(&pattern);
    assert_eq!(name, "x");
    
    let tuple_pattern = Pattern::Tuple(vec![
        Pattern::Identifier("x".to_string()),
        Pattern::Identifier("y".to_string())
    ]);
    let name = extract_name_from_pattern(&tuple_pattern);
    assert_eq!(name, "x"); // Should extract first identifier
}

fn extract_name_from_pattern(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Identifier(name) => name.clone(),
        Pattern::Tuple(patterns) => {
            // Extract the first identifier from tuple destructuring
            patterns.first()
                .and_then(|p| match p {
                    Pattern::Identifier(name) => Some(name.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| "__destructured".to_string())
        }
        Pattern::List(patterns) => {
            // Extract the first identifier from list destructuring  
            patterns.first()
                .and_then(|p| match p {
                    Pattern::Identifier(name) => Some(name.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| "__destructured".to_string())
        }
        _ => "__destructured".to_string(), // Placeholder for complex patterns
    }
}