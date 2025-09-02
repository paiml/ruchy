// TDD: Fix the first Let reference to understand the pattern
use ruchy::frontend::ast::{Pattern};

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
fn test_dispatcher_pattern_fix() {
    // Test the pattern we need for dispatcher.rs:312
    let simple_pattern = Pattern::Identifier("x".to_string());
    let extracted = extract_name_from_pattern(&simple_pattern);
    assert_eq!(extracted, "x");
    
    let tuple_pattern = Pattern::Tuple(vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Identifier("b".to_string())
    ]);
    let extracted = extract_name_from_pattern(&tuple_pattern);
    assert_eq!(extracted, "a");
}