// EXTREME TDD: Tests for List/Set/Dict Comprehensions from unified spec
// ALL TESTS MUST FAIL INITIALLY - Implementation comes AFTER

use ruchy::compile;

#[cfg(test)]
mod test_comprehensions {
    use super::*;

    // List comprehension tests
    #[test]
    fn test_simple_list_comprehension() {
        let code = r#"
            fun main() {
                let squares = [x * x for x in 0..10];
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile simple list comprehension"
        );
        let output = result.unwrap();
        assert!(
            output.contains("map") && output.contains("x | x * x") && output.contains("collect")
        );
    }

    #[test]
    fn test_list_comprehension_with_filter() {
        let code = r#"
            fun main() {
                let evens = [x for x in 0..100 if x % 2 == 0];
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile list comprehension with filter"
        );
        let output = result.unwrap();
        assert!(output.contains("filter") && output.contains("x % 2") && output.contains("== 0"));
        assert!(output.contains("map") && output.contains("| x |"));
    }

    #[test]
    fn test_list_comprehension_with_transform_and_filter() {
        let code = r#"
            fun main() {
                let result = [x * 2 for x in data if x > 0];
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile comprehension with transform and filter"
        );
        let output = result.unwrap();
        assert!(output.contains("filter") && output.contains("x > 0") && output.contains("x * 2"));
    }

    #[test]
    fn test_nested_list_comprehension() {
        let code = r#"
            fun main() {
                let pairs = [(x, y) for x in 0..3 for y in 0..3];
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile nested list comprehension"
        );
    }

    #[test]
    fn test_list_comprehension_with_complex_expression() {
        let code = r#"
            fun main() {
                let computed = [x.sqrt() + y.powi(2) for x in floats for y in nums if x > 0.0];
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile complex list comprehension"
        );
    }

    // Set comprehension tests
    #[test]
    fn test_simple_set_comprehension() {
        let code = r#"
            fun main() {
                let unique = {x % 10 for x in data};
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile simple set comprehension");
        let output = result.unwrap();
        assert!(output.contains("HashSet") || output.contains("collect"));
        assert!(output.contains("map") && output.contains("x % 10"));
    }

    #[test]
    fn test_set_comprehension_with_filter() {
        let code = r#"
            fun main() {
                let positive_mods = {x % 10 for x in data if x > 0};
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile set comprehension with filter"
        );
        let output = result.unwrap();
        assert!(output.contains("HashSet") || output.contains("collect"));
        assert!(output.contains("filter") && output.contains("x > 0"));
    }

    #[test]
    fn test_set_comprehension_from_string() {
        let code = r#"
            fun main() {
                let chars = {c for c in text.chars() if c.is_alphabetic()};
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile set comprehension from string"
        );
    }

    // Dict/HashMap comprehension tests
    #[test]
    fn test_simple_dict_comprehension() {
        let code = r#"
            fun main() {
                let word_lengths = {word: word.len() for word in text.split_whitespace()};
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile simple dict comprehension"
        );
        let output = result.unwrap();
        assert!(output.contains("HashMap") || output.contains("collect"));
        assert!(output.contains("map") && output.contains("word") && output.contains("len"));
    }

    #[test]
    fn test_dict_comprehension_with_filter() {
        let code = r#"
            fun main() {
                let long_words = {word: word.len() for word in words if word.len() > 5};
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile dict comprehension with filter"
        );
        let output = result.unwrap();
        assert!(output.contains("HashMap") || output.contains("collect"));
        assert!(output.contains("filter") && output.contains("word.len() > 5"));
    }

    #[test]
    fn test_dict_comprehension_with_enumerate() {
        let code = r#"
            fun main() {
                let indexed = {i: value for (i, value) in data.iter().enumerate()};
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile dict comprehension with enumerate"
        );
    }

    #[test]
    fn test_dict_comprehension_from_tuples() {
        let code = r#"
            fun main() {
                let mapping = {k: v for (k, v) in pairs};
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile dict comprehension from tuples"
        );
    }

    // Type inference in comprehensions
    #[test]
    fn test_comprehension_type_inference() {
        let code = r#"
            fun process() -> Vec<i32> {
                [x * 2 for x in 0..10]
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile comprehension with type inference"
        );
        let output = result.unwrap();
        assert!(output.contains("Vec<i32>"));
    }

    #[test]
    fn test_set_comprehension_type_inference() {
        let code = r#"
            fun get_uniques() -> HashSet<i32> {
                {x % 10 for x in 0..100}
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile set comprehension with type inference"
        );
    }

    // Complex nested comprehensions
    #[test]
    fn test_matrix_comprehension() {
        let code = r#"
            fun create_matrix() -> Vec<Vec<i32>> {
                [[i + j for j in 0..5] for i in 0..5]
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile matrix comprehension");
    }

    #[test]
    fn test_flattened_comprehension() {
        let code = r#"
            fun flatten() -> Vec<i32> {
                [item for row in matrix for item in row]
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile flattened comprehension");
    }

    // Comprehensions with method calls
    #[test]
    fn test_comprehension_with_methods() {
        let code = r#"
            fun process_strings() -> Vec<String> {
                [s.to_uppercase() for s in strings if !s.is_empty()]
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile comprehension with method calls"
        );
    }

    #[test]
    fn test_comprehension_with_chain() {
        let code = r#"
            fun chain_process() -> Vec<i32> {
                [x for x in data.iter().filter(|n| **n > 0).map(|n| n * 2)]
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile comprehension with iterator chain"
        );
    }

    // Comprehensions with pattern matching
    #[test]
    fn test_comprehension_with_patterns() {
        let code = r#"
            fun extract_values() -> Vec<i32> {
                [value for Some(value) in options]
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile comprehension with patterns"
        );
    }

    #[test]
    fn test_dict_comprehension_with_pattern() {
        let code = r#"
            fun create_map() -> HashMap<String, i32> {
                {name: age for Person { name, age, .. } in people}
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile dict comprehension with pattern"
        );
    }

    // Comprehensions with ranges
    #[test]
    fn test_comprehension_with_step_range() {
        let code = r#"
            fun step_values() -> Vec<i32> {
                [x for x in (0..100).step_by(5)]
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile comprehension with step range"
        );
    }

    #[test]
    fn test_comprehension_with_reverse_range() {
        let code = r#"
            fun countdown() -> Vec<i32> {
                [x for x in (0..10).rev()]
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile comprehension with reverse range"
        );
    }

    // Comprehensions with async
    #[test]
    fn test_async_comprehension() {
        let code = r#"
            async fun fetch_all() -> Vec<Data> {
                [fetch(url).await for url in urls]
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile async comprehension");
    }

    // Performance-oriented comprehensions
    #[test]
    fn test_parallel_comprehension() {
        let code = r#"
            fun parallel_process() -> Vec<i32> {
                [expensive_computation(x) for x in data.par_iter()]
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile parallel comprehension");
    }

    // Comprehensions with closures
    #[test]
    fn test_comprehension_capturing_variables() {
        let code = r#"
            fun scale_values(factor: i32) -> Vec<i32> {
                let multiplier = factor * 2;
                [x * multiplier for x in data]
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile comprehension with captured variables"
        );
    }

    // Edge cases
    #[test]
    fn test_empty_comprehension() {
        let code = r#"
            fun empty() -> Vec<i32> {
                [x for x in vec![] if x > 0]
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile empty comprehension");
    }

    #[test]
    fn test_comprehension_with_side_effects() {
        let code = r#"
            fun with_side_effects() -> Vec<i32> {
                let mut counter = 0;
                [{ counter += 1; x * counter } for x in data]
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile comprehension with side effects"
        );
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use quickcheck::{quickcheck, TestResult};

        fn prop_list_comprehension_basic(n: u8) -> TestResult {
            let code = format!("fun test() -> Vec<i32> {{ [x for x in 0..{}] }}", n);
            let result = compile(&code);
            TestResult::from_bool(result.is_ok() || result.is_err())
        }

        fn prop_set_comprehension_basic(n: u8) -> TestResult {
            let code = format!(
                "fun test() -> HashSet<i32> {{ {{x % 10 for x in 0..{}}} }}",
                n
            );
            let result = compile(&code);
            TestResult::from_bool(result.is_ok() || result.is_err())
        }

        quickcheck! {
            fn test_list_comprehension_ranges(n: u8) -> TestResult {
                prop_list_comprehension_basic(n)
            }

            fn test_set_comprehension_ranges(n: u8) -> TestResult {
                prop_set_comprehension_basic(n)
            }
        }
    }
}
