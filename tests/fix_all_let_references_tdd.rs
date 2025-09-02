// This test will help us fix all references to the Let variant systematically
use ruchy::frontend::ast::{Expr, ExprKind, Pattern};

fn extract_name_from_pattern(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Identifier(name) => name.clone(),
        Pattern::Tuple(patterns) => {
            patterns.first()
                .and_then(|p| match p {
                    Pattern::Identifier(name) => Some(name.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| "__destructured".to_string())
        }
        Pattern::List(patterns) => {
            patterns.first()
                .and_then(|p| match p {
                    Pattern::Identifier(name) => Some(name.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| "__destructured".to_string())
        }
        _ => "__destructured".to_string(),
    }
}

#[test]
fn test_pattern_name_extraction() {
    let simple_pattern = Pattern::Identifier("x".to_string());
    assert_eq!(extract_name_from_pattern(&simple_pattern), "x");
    
    let tuple_pattern = Pattern::Tuple(vec![
        Pattern::Identifier("x".to_string()),
        Pattern::Identifier("y".to_string())
    ]);
    assert_eq!(extract_name_from_pattern(&tuple_pattern), "x");
}

// This test defines how Let should work with patterns
#[test]
fn test_let_with_pattern_structure() {
    // We need a helper to work with Let expressions that have patterns
    let pattern = Pattern::Tuple(vec![
        Pattern::Identifier("x".to_string()),
        Pattern::Identifier("y".to_string())
    ]);
    
    // Helper function to extract name for backwards compatibility
    let extracted_name = extract_name_from_pattern(&pattern);
    
    // The name should be the first identifier from the pattern
    assert_eq!(extracted_name, "x");
}