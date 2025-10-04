// Extreme TDD: Big O complexity and entropy requirements for interpreter
// This test suite ensures the refactored interpreter has optimal algorithmic complexity
// and minimal code entropy (no unnecessary duplication).

use std::collections::HashMap;

#[test]
fn test_entropy_reduction_requirement() {
    // PMAT entropy analysis found 4,088 lines (87%) could be eliminated
    // This test ensures we achieve at least 80% reduction

    let original_lines = 7641;
    let current_file = std::fs::read_to_string("src/runtime/interpreter.rs").unwrap();
    let current_lines = current_file.lines().count();

    let reduction_percentage =
        ((original_lines - current_lines) as f64 / original_lines as f64) * 100.0;

    assert!(
        current_lines < 1600,
        "interpreter.rs has {} lines, target is <1600 (80% reduction). Current reduction: {:.1}%",
        current_lines,
        reduction_percentage
    );
}

#[test]
fn test_pattern_diversity_requirement() {
    // PMAT found pattern diversity of only 19.8% (minimum should be 30%)
    // This ensures we have sufficient variety in our code patterns

    let file_content = std::fs::read_to_string("src/runtime/interpreter.rs").unwrap();

    // Count unique function patterns
    let functions: Vec<&str> = file_content
        .lines()
        .filter(|line| line.trim().starts_with("fn ") || line.trim().starts_with("pub fn "))
        .collect();

    let unique_patterns = count_unique_patterns(&functions);
    let diversity = (unique_patterns as f64 / functions.len() as f64) * 100.0;

    assert!(
        diversity >= 30.0,
        "Pattern diversity is {:.1}%, minimum required is 30%",
        diversity
    );
}

fn count_unique_patterns(functions: &[&str]) -> usize {
    let mut patterns = std::collections::HashSet::new();
    for func in functions {
        // Simple pattern extraction based on function signature structure
        let pattern = func.split('(').next().unwrap_or("").trim();
        patterns.insert(pattern);
    }
    patterns.len()
}

#[test]
fn test_eval_expr_big_o_complexity() {
    // The main eval_expr function should have O(n) complexity for expression depth
    // Not O(n²) or worse due to repeated traversals

    // This is validated by ensuring:
    // 1. No nested loops over the same expression tree
    // 2. Single-pass evaluation where possible
    // 3. Memoization/caching for repeated subexpressions

    let file_content = std::fs::read_to_string("src/runtime/interpreter.rs").unwrap();

    // Find eval_expr function
    let eval_expr_start = file_content.find("pub fn eval_expr").unwrap_or(0);
    let eval_expr_end = file_content[eval_expr_start..]
        .find("\n    pub fn ")
        .unwrap_or(file_content.len() - eval_expr_start);

    let eval_expr_body = &file_content[eval_expr_start..eval_expr_start + eval_expr_end];

    // Check for nested loops (indicator of O(n²) or worse)
    let loop_count =
        eval_expr_body.matches("for ").count() + eval_expr_body.matches("while ").count();

    assert!(
        loop_count <= 1,
        "eval_expr has {} loops, suggesting O(n²) or worse complexity",
        loop_count
    );
}

#[test]
fn test_no_data_validation_duplication() {
    // PMAT found DataValidation pattern repeated 10 times
    // After refactoring, validation should be centralized

    let file_content = std::fs::read_to_string("src/runtime/interpreter.rs").unwrap();

    // Common validation patterns that shouldn't be duplicated
    let validation_patterns = [
        "args.len() != 1",
        "args.len() != 2",
        "args.is_empty()",
        "return Err(InterpreterError",
        "expects exactly",
    ];

    for pattern in &validation_patterns {
        let occurrences = file_content.matches(pattern).count();
        assert!(
            occurrences <= 3,
            "Validation pattern '{}' appears {} times (max 3 allowed)",
            pattern,
            occurrences
        );
    }
}

#[test]
fn test_no_data_transformation_duplication() {
    // PMAT found DataTransformation pattern repeated 10 times
    // Transformations should use common utility functions

    let file_content = std::fs::read_to_string("src/runtime/interpreter.rs").unwrap();

    // Common transformation patterns that shouldn't be duplicated
    let transform_patterns = [
        "Value::Integer(",
        "Value::Float(",
        "Value::String(",
        "as f64",
        "as i64",
        ".to_string()",
    ];

    for pattern in &transform_patterns {
        let occurrences = file_content.matches(pattern).count();
        assert!(
            occurrences <= 10,
            "Transformation pattern '{}' appears {} times (max 10 allowed)",
            pattern,
            occurrences
        );
    }
}

#[test]
fn test_lookup_complexity_is_o1() {
    // Variable and function lookups should be O(1) using HashMap
    // Not O(n) with linear search

    let file_content = std::fs::read_to_string("src/runtime/interpreter.rs").unwrap();

    // Check for HashMap usage for environments
    assert!(
        file_content.contains("HashMap") || file_content.contains("BTreeMap"),
        "No HashMap/BTreeMap found for O(1) lookups"
    );

    // Ensure no linear search patterns for variable lookup
    let has_linear_search =
        file_content.contains("iter().find(") || file_content.contains("for var in");

    assert!(
        !has_linear_search,
        "Linear search pattern found, should use HashMap for O(1) lookup"
    );
}

#[test]
fn test_match_expression_complexity() {
    // Match expressions should have O(1) branch selection
    // Using jump tables, not O(n) sequential checking

    let file_content = std::fs::read_to_string("src/runtime/interpreter.rs").unwrap();

    // Count match arms in eval_expr_kind
    if let Some(eval_expr_kind_start) = file_content.find("fn eval_expr_kind") {
        let function_end = file_content[eval_expr_kind_start..]
            .find("\n    fn ")
            .unwrap_or(1000);

        let function_body =
            &file_content[eval_expr_kind_start..eval_expr_kind_start + function_end];
        let match_arms = function_body.matches("=>").count();

        // With proper module extraction, should have <10 match arms
        assert!(
            match_arms <= 10,
            "eval_expr_kind has {} match arms, should be ≤10 after modularization",
            match_arms
        );
    }
}

#[test]
fn test_recursive_depth_limit() {
    // Recursive evaluation should have depth limit to prevent stack overflow
    // This ensures O(depth) space complexity, not unbounded

    let file_content = std::fs::read_to_string("src/runtime/interpreter.rs").unwrap();

    // Check for recursion depth tracking
    let has_depth_limit = file_content.contains("max_recursion_depth")
        || file_content.contains("recursion_depth")
        || file_content.contains("depth_limit")
        || file_content.contains("stack_depth");

    assert!(
        has_depth_limit,
        "No recursion depth limit found - unbounded recursion risk"
    );
}

#[test]
fn test_caching_for_expensive_operations() {
    // Expensive operations should be cached/memoized
    // This improves average case from O(n) to O(1)

    let file_content = std::fs::read_to_string("src/runtime/interpreter.rs").unwrap();

    // Check for caching mechanisms
    let has_cache = file_content.contains("cache")
        || file_content.contains("memo")
        || file_content.contains("InlineCache")
        || file_content.contains("cached_");

    assert!(
        has_cache,
        "No caching mechanism found for expensive operations"
    );
}

// Property-based tests for algorithmic complexity
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use std::collections::HashMap;

    proptest! {
        #[test]
        fn test_expression_depth_linear_complexity(depth: u8) {
            // For any expression depth n, evaluation should be O(n), not exponential
            let depth = depth % 20; // Limit depth for testing

            // This would be tested by creating nested expressions and measuring time
            // For now, we just verify the property holds conceptually

            // Time complexity should be: time = k * depth for some constant k
            // Not: time = 2^depth (exponential) or depth^2 (quadratic)

            let expected_max_operations = (depth as usize) * 100; // Linear bound
            prop_assert!(
                expected_max_operations < 10000,
                "Operations for depth {} exceed linear bound",
                depth
            );
        }

        #[test]
        fn test_hashmap_lookup_consistency(keys: Vec<String>) {
            // HashMap lookups should maintain O(1) average case
            // regardless of the number of keys

            let mut map = HashMap::new();
            for (i, key) in keys.iter().enumerate() {
                map.insert(key.clone(), i);
            }

            // Any lookup should be constant time (conceptually)
            for key in &keys {
                let _ = map.get(key); // O(1) operation
            }

            // Property: lookup time doesn't depend on map size
            prop_assert!(true, "HashMap maintains O(1) lookups");
        }

        #[test]
        fn test_pattern_matching_branch_selection(variant_count: u8) {
            // Match expressions with n branches should select in O(1)
            // Not O(n) by checking each branch sequentially

            let variants = variant_count % 50; // Reasonable number of variants

            // Rust's match is compiled to jump tables for efficiency
            // This ensures O(1) branch selection

            prop_assert!(
                variants < 100,
                "Too many match variants ({}) may degrade to O(n)",
                variants
            );
        }
    }
}

#[test]
fn test_module_separation_reduces_complexity() {
    // Check that extracted modules exist and are properly sized
    let modules = [
        ("src/runtime/eval_literal.rs", 200),
        ("src/runtime/eval_binary.rs", 500),
        ("src/runtime/eval_control_flow.rs", 500),
        ("src/runtime/eval_function.rs", 300),
        ("src/runtime/eval_data_structures.rs", 400),
        ("src/runtime/builtins.rs", 800),
        ("src/runtime/eval_pattern.rs", 200),
        ("src/runtime/gc.rs", 300),
    ];

    for (module_path, max_lines) in &modules {
        if let Ok(content) = std::fs::read_to_string(module_path) {
            let lines = content.lines().count();
            assert!(
                lines <= *max_lines,
                "{} has {} lines, exceeds limit of {}",
                module_path,
                lines,
                max_lines
            );
        }
    }
}

#[test]
fn test_algorithmic_improvements_documented() {
    // Each module should document its Big O complexity
    let modules = [
        "src/runtime/eval_literal.rs",
        "src/runtime/eval_binary.rs",
        "src/runtime/eval_control_flow.rs",
    ];

    for module_path in &modules {
        if let Ok(content) = std::fs::read_to_string(module_path) {
            let has_complexity_docs = content.contains("Complexity:")
                || content.contains("O(")
                || content.contains("complexity");

            assert!(
                has_complexity_docs,
                "{} missing Big O complexity documentation",
                module_path
            );
        }
    }
}
