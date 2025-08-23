//! Property-based tests for module path syntax

use proptest::prelude::*;
use ruchy::frontend::parser::Parser;

/// Generate valid module path segments
fn arb_identifier() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,15}".prop_map(|s| s.to_string())
}

/// Generate qualified module paths like a::b::c
fn arb_module_path() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_identifier(), 1..5)
        .prop_map(|segments| segments.join("::"))
}

/// Generate type names with module paths
fn arb_qualified_type() -> impl Strategy<Value = String> {
    (arb_module_path(), "[A-Z][a-zA-Z0-9]{0,15}")
        .prop_map(|(path, ty)| format!("{}::{}", path, ty))
}

proptest! {
    #[test]
    fn test_parse_qualified_types(qualified_type in arb_qualified_type()) {
        let input = format!("fn test(x: {}) {{ x }}", qualified_type);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        
        prop_assert!(
            result.is_ok(),
            "Failed to parse qualified type: {} - Error: {:?}",
            qualified_type,
            result.err()
        );
    }

    #[test]
    fn test_parse_qualified_function_calls(module_path in arb_module_path()) {
        let input = format!("{}::function()", module_path);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        
        prop_assert!(
            result.is_ok(),
            "Failed to parse qualified function call: {} - Error: {:?}",
            module_path,
            result.err()
        );
    }

    #[test]
    fn test_parse_nested_qualified_types(
        outer_path in arb_module_path(),
        inner_path in arb_qualified_type()
    ) {
        let input = format!(
            "fn test(x: {}::Result<{}, Error>) {{ x }}",
            outer_path, inner_path
        );
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        
        // This should parse successfully
        prop_assert!(
            result.is_ok(),
            "Failed to parse nested qualified type - Error: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_module_path_length(segments in prop::collection::vec(arb_identifier(), 1..20)) {
        let module_path = segments.join("::");
        let input = format!("{}::function()", module_path);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        
        // Should handle arbitrarily long module paths
        prop_assert!(
            result.is_ok(),
            "Failed to parse long module path with {} segments - Error: {:?}",
            segments.len(),
            result.err()
        );
    }

    #[test]
    fn test_mixed_qualified_and_generics(
        module_path in arb_module_path(),
        type1 in arb_qualified_type(),
        type2 in arb_qualified_type()
    ) {
        let input = format!(
            "fn test(x: {}::HashMap<{}, {}>) {{ x }}",
            module_path, type1, type2
        );
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        
        prop_assert!(
            result.is_ok(),
            "Failed to parse qualified generic type - Error: {:?}",
            result.err()
        );
    }
}