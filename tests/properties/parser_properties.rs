// tests/properties/parser_properties.rs
// Property-based tests for parser robustness and correctness

#[cfg(feature = "testing")]
use quickcheck_macros::quickcheck;
use ruchy::utils::is_valid_identifier;

#[test]
#[cfg(feature = "testing")]
fn test_parser_properties() {
    use quickcheck::{quickcheck, TestResult};
    
    // Property: Valid identifiers should always parse
    fn prop_valid_identifiers_parse(name: String) -> TestResult {
        // Filter to valid identifiers
        if !is_valid_identifier(&name) {
            return TestResult::discard();
        }
        
        let code = format!("let {} = 42", name);
        TestResult::from_bool(super::can_parse(&code))
    }
    
    quickcheck(prop_valid_identifiers_parse as fn(String) -> TestResult);
    
    // Property: Balanced parentheses should parse correctly
    fn prop_balanced_parens(expr: String) -> TestResult {
        if !is_balanced_parens(&expr) {
            return TestResult::discard();
        }
        
        let code = format!("let x = ({})", expr);
        TestResult::from_bool(super::can_parse(&code))
    }
    
    quickcheck(prop_balanced_parens as fn(String) -> TestResult);
}

fn is_balanced_parens(s: &str) -> bool {
    let mut count = 0;
    for c in s.chars() {
        match c {
            '(' => count += 1,
            ')' => {
                count -= 1;
                if count < 0 { return false; }
            }
            _ => {}
        }
    }
    count == 0
}

#[test]
fn test_parser_regression_properties() {
    // Known good expressions that should always parse
    let known_good = vec![
        "42",
        "true",
        "false", 
        "\"hello\"",
        "[1, 2, 3]",
        "{\"key\": \"value\"}",
        "fn test() { 42 }",
    ];
    
    for expr in known_good {
        assert!(super::can_parse(expr), "Failed to parse known good expression: {}", expr);
    }
}