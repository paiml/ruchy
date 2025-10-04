// EXTREME TDD: Property tests for set literals with 10,000 iterations
// Testing invariants and edge cases with random inputs

use proptest::prelude::*;
use ruchy::compile;

// Property: Empty set always compiles to HashSet::new()
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_empty_set_always_compiles(_seed: u8) {
        let code = "fun main() { let s = {}; }";
        let result = compile(code);
        prop_assert!(result.is_ok(), "Empty set should always compile");
        let output = result.unwrap();
        prop_assert!(output.contains("HashSet"));
    }
}

// Property: Single element sets always compile
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_single_element_set(value: i32) {
        let code = format!("fun main() {{ let s = {{{value}}}; }}");
        let result = compile(&code);
        prop_assert!(result.is_ok(), "Single element set should compile");
        let output = result.unwrap();
        prop_assert!(output.contains("HashSet"));
        // Handle negative numbers which get formatted as "insert (- 1)" with spaces
        let expected1 = if value < 0 {
            format!("insert (- {})", value.abs())
        } else {
            format!("insert ({value})")
        };
        let expected2 = if value < 0 {
            format!("insert(- {})", value.abs())
        } else {
            format!("insert({value})")
        };
        prop_assert!(output.contains(&expected1) || output.contains(&expected2));
    }
}

// Property: Two element sets always compile and preserve both elements
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_two_element_set(a: i32, b: i32) {
        // Skip if elements are the same (set would have only one element)
        prop_assume!(a != b);

        let code = format!("fun main() {{ let s = {{{a}, {b}}}; }}");
        let result = compile(&code);
        prop_assert!(result.is_ok(), "Two element set should compile");
        let output = result.unwrap();
        prop_assert!(output.contains("HashSet"));
        let expected_a1 = if a < 0 {
            format!("insert (- {})", a.abs())
        } else {
            format!("insert ({a})")
        };
        let expected_a2 = if a < 0 {
            format!("insert(- {})", a.abs())
        } else {
            format!("insert({a})")
        };
        let expected_b1 = if b < 0 {
            format!("insert (- {})", b.abs())
        } else {
            format!("insert ({b})")
        };
        let expected_b2 = if b < 0 {
            format!("insert(- {})", b.abs())
        } else {
            format!("insert({b})")
        };
        prop_assert!(output.contains(&expected_a1) || output.contains(&expected_a2));
        prop_assert!(output.contains(&expected_b1) || output.contains(&expected_b2));
    }
}

// Property: Sets with many elements compile correctly
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_many_element_set(elements: Vec<i32>) {
        // Limit to reasonable size
        let elements = elements.into_iter().take(20).collect::<Vec<_>>();

        if elements.is_empty() {
            return Ok(()); // Skip empty case
        }

        let elements_str = elements.iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");

        let code = format!("fun main() {{ let s = {{{elements_str}}}; }}");
        let result = compile(&code);
        prop_assert!(result.is_ok(), "Multi-element set should compile");
        let output = result.unwrap();
        prop_assert!(output.contains("HashSet"));

        // Check that each unique element appears
        for elem in elements {
            let expected1 = if elem < 0 {
                format!("insert (- {})", elem.abs())
            } else {
                format!("insert ({elem})")
            };
            let expected2 = if elem < 0 {
                format!("insert(- {})", elem.abs())
            } else {
                format!("insert({elem})")
            };
            prop_assert!(output.contains(&expected1) || output.contains(&expected2));
        }
    }
}

// Property: Sets with string literals compile
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_string_set(strings: Vec<String>) {
        let strings = strings.into_iter()
            .take(5)
            .filter(|s| !s.contains('"') && !s.contains('\\'))
            .collect::<Vec<_>>();

        if strings.is_empty() {
            return Ok(());
        }

        let elements_str = strings.iter()
            .map(|s| format!("\"{s}\""))
            .collect::<Vec<_>>()
            .join(", ");

        let code = format!("fun main() {{ let s = {{{elements_str}}}; }}");
        let result = compile(&code);
        prop_assert!(result.is_ok(), "String set should compile");
        let output = result.unwrap();
        prop_assert!(output.contains("HashSet"));
    }
}

// Property: Sets with variables compile
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_variable_set(var_names: Vec<String>) {
        // Generate only valid Ruchy identifiers: must start with letter, contain only alphanumeric + underscore
        let var_names = var_names.into_iter()
            .filter_map(|s| {
                if s.is_empty() {
                    return None;
                }
                let first_char = s.chars().next().unwrap();
                if !first_char.is_ascii_alphabetic() {
                    return None;
                }
                // Clean up to make valid identifier
                let cleaned = s.chars()
                    .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect::<String>();
                if cleaned.is_empty() || cleaned == "_" || cleaned.len() > 20 {
                    None
                } else {
                    // Check not reserved keyword
                    if matches!(cleaned.as_str(), "let" | "fun" | "if" | "else" | "while" | "for" | "return" | "true" | "false") {
                        None
                    } else {
                        Some(cleaned)
                    }
                }
            })
            .take(3)
            .collect::<Vec<_>>();

        if var_names.len() < 2 {
            return Ok(());
        }

        // Initialize variables first
        let var_inits = var_names.iter()
            .enumerate()
            .map(|(i, name)| format!("let {name} = {i};"))
            .collect::<Vec<_>>()
            .join(" ");

        let elements_str = var_names.join(", ");

        let code = format!("fun main() {{ {var_inits} let s = {{{elements_str}}}; }}");
        let result = compile(&code);
        prop_assert!(result.is_ok());
    }
}

// Property: Nested sets compile (sets of sets)
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_nested_sets(depth: u8) {
        let depth = (depth % 3) + 1; // Limit depth to 1-3

        let mut expr = "1".to_string();
        for _ in 0..depth {
            expr = format!("{{{expr}}}");
        }

        let code = format!("fun main() {{ let s = {expr}; }}");
        let result = compile(&code);

        // Nested sets might not be supported yet, so we just check it doesn't crash
        let _ = result; // Don't assert success, just ensure no panic
    }
}

// Property: Sets with expressions compile
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_expression_set(a: i32, b: i32) {
        let code = format!("fun main() {{ let s = {{{a} + {b}, {a} * {b}}}; }}");
        let result = compile(&code);
        prop_assert!(result.is_ok(), "Expression set should compile");
        let output = result.unwrap();
        prop_assert!(output.contains("HashSet"));
    }
}

// Property: Set with trailing comma compiles
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_trailing_comma_set(elements: Vec<i32>) {
        let elements = elements.into_iter().take(5).collect::<Vec<_>>();

        if elements.is_empty() {
            return Ok(());
        }

        let elements_str = elements.iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");

        // Add trailing comma
        let code = format!("fun main() {{ let s = {{{elements_str},}}; }}");
        let result = compile(&code);
        prop_assert!(result.is_ok(), "Set with trailing comma should compile");
    }
}

// Property: Very large sets compile without stack overflow
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))] // Fewer cases for expensive test

    #[test]
    fn prop_large_set(size: usize) {
        let size = size % 1000; // Limit to 1000 elements

        let elements_str = (0..size)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let code = format!("fun main() {{ let s = {{{elements_str}}}; }}");
        let result = compile(&code);
        prop_assert!(result.is_ok(), "Large set should compile without stack overflow");
    }
}

// Property: Sets preserve uniqueness (duplicates handled correctly)
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_duplicate_elements_set(elem: i32, count: u8) {
        let count = (count % 10) + 1; // 1 to 10 duplicates

        let elements_str = vec![elem.to_string(); count as usize].join(", ");

        let code = format!("fun main() {{ let s = {{{elements_str}}}; }}");
        let result = compile(&code);
        prop_assert!(result.is_ok(), "Set with duplicates should compile");
        let output = result.unwrap();
        prop_assert!(output.contains("HashSet"));

        // Should still contain insert for the element
        let expected1 = if elem < 0 {
            format!("insert (- {})", elem.abs())
        } else {
            format!("insert ({elem})")
        };
        let expected2 = if elem < 0 {
            format!("insert(- {})", elem.abs())
        } else {
            format!("insert({elem})")
        };
        prop_assert!(output.contains(&expected1) || output.contains(&expected2));
    }
}

// Property: Mixed type sets (should fail or coerce types)
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_mixed_type_set(int_val: i32, float_val: f64) {
        let code = format!("fun main() {{ let s = {{{int_val}, {float_val}}}; }}");
        let result = compile(&code);

        // Mixed types might fail or succeed with coercion
        // We just ensure no panic
        let _ = result;
    }
}

// Property: Set comprehension-like syntax
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_set_with_range(start: i32, end: i32) {
        let (start, end) = if start <= end { (start, end) } else { (end, start) };
        let start = start.clamp(-100, 100);
        let end = end.clamp(-100, 100);

        // Try to create a set from range elements
        let elements = (start..=end.min(start + 10))
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let code = format!("fun main() {{ let s = {{{elements}}}; }}");
        let result = compile(&code);
        prop_assert!(result.is_ok(), "Set from range elements should compile");
    }
}
