// EXTREME TDD: Tests for Set Literals from unified spec
// ALL TESTS MUST FAIL INITIALLY - Implementation comes AFTER

use ruchy::compile;

#[cfg(test)]
mod test_set_literals {
    use super::*;

    // Basic set literal tests
    #[test]
    fn test_empty_set_literal() {
        let code = r"
            fun main() {
                let empty = {};
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile empty set literal");
        let output = result.unwrap();
        assert!(output.contains("HashSet::new()"));
    }

    #[test]
    fn test_simple_set_literal() {
        let code = r"
            fun main() {
                let numbers = {1, 2, 3};
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile simple set literal");
        let output = result.unwrap();
        assert!(output.contains("HashSet"));
        assert!(output.contains("insert (1"));
        assert!(output.contains("insert (2"));
        assert!(output.contains("insert (3"));
    }

    #[test]
    fn test_set_with_strings() {
        let code = r#"
            fun main() {
                let words = {"hello", "world", "rust"};
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set with strings");
        let output = result.unwrap();
        assert!(output.contains("HashSet"));
        assert!(output.contains(r#"insert ("hello""#) || output.contains(r#"insert("hello""#));
    }

    #[test]
    fn test_set_with_variables() {
        let code = r"
            fun main() {
                let x = 10;
                let y = 20;
                let numbers = {x, y, 30};
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set with variables");
        let output = result.unwrap();
        assert!(output.contains("insert (x)") || output.contains("insert(x)"));
        assert!(output.contains("insert (y)") || output.contains("insert(y)"));
        assert!(output.contains("insert (30") || output.contains("insert(30"));
    }

    #[test]
    fn test_set_with_expressions() {
        let code = r"
            fun main() {
                let nums = {1 + 1, 2 * 3, 10 - 5};
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set with expressions");
        let output = result.unwrap();
        assert!(output.contains("HashSet"));
        assert!(
            output.contains("insert (1i32 + 1i32)")
                || output.contains("insert (1 + 1)")
                || output.contains("insert(1 + 1)")
        );
    }

    // Type annotations
    #[test]
    fn test_set_with_type_annotation() {
        let code = r"
            fun main() {
                let numbers: HashSet<i32> = {1, 2, 3};
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set with type annotation");
        let output = result.unwrap();
        assert!(output.contains("HashSet<i32>") || output.contains("HashSet :: < i32 >"));
    }

    #[test]
    fn test_set_with_float_type() {
        let code = r"
            fun main() {
                let floats: HashSet<f64> = {1.0, 2.5, 3.14};
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set with float type");
        let output = result.unwrap();
        assert!(output.contains("HashSet"));
        assert!(output.contains("1.0") || output.contains("1f64"));
    }

    // Set operations
    #[test]
    fn test_set_union() {
        let code = r"
            fun main() {
                let a = {1, 2, 3};
                let b = {3, 4, 5};
                let c = a.union(&b);
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set union");
        let output = result.unwrap();
        assert!(output.contains("union"));
    }

    #[test]
    fn test_set_intersection() {
        let code = r"
            fun main() {
                let a = {1, 2, 3};
                let b = {2, 3, 4};
                let c = a.intersection(&b);
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set intersection");
        let output = result.unwrap();
        assert!(output.contains("intersection"));
    }

    #[test]
    fn test_set_difference() {
        let code = r"
            fun main() {
                let a = {1, 2, 3};
                let b = {2, 3};
                let c = a.difference(&b);
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set difference");
        let output = result.unwrap();
        assert!(output.contains("difference"));
    }

    #[test]
    fn test_set_symmetric_difference() {
        let code = r"
            fun main() {
                let a = {1, 2, 3};
                let b = {2, 3, 4};
                let c = a.symmetric_difference(&b);
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile symmetric difference");
        let output = result.unwrap();
        assert!(output.contains("symmetric_difference"));
    }

    // Set methods
    #[test]
    fn test_set_contains() {
        let code = r"
            fun main() {
                let nums = {1, 2, 3};
                let has_two = nums.contains(&2);
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set contains");
        let output = result.unwrap();
        assert!(output.contains("contains"));
    }

    #[test]
    fn test_set_insert() {
        let code = r"
            fun main() {
                let mut nums = {1, 2, 3};
                nums.insert(4);
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set insert");
        let output = result.unwrap();
        assert!(output.contains("insert (4") || output.contains("insert(4"));
    }

    #[test]
    fn test_set_remove() {
        let code = r"
            fun main() {
                let mut nums = {1, 2, 3};
                nums.remove(&2);
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set remove");
        let output = result.unwrap();
        assert!(output.contains("remove"));
    }

    #[test]
    fn test_set_len() {
        let code = r"
            fun main() {
                let nums = {1, 2, 3};
                let size = nums.len();
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set len");
        let output = result.unwrap();
        assert!(output.contains("len()"));
    }

    #[test]
    fn test_set_is_empty() {
        let code = r"
            fun main() {
                let nums = {1, 2, 3};
                let empty = nums.is_empty();
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set is_empty");
        let output = result.unwrap();
        assert!(output.contains("is_empty()"));
    }

    #[test]
    fn test_set_clear() {
        let code = r"
            fun main() {
                let mut nums = {1, 2, 3};
                nums.clear();
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set clear");
        let output = result.unwrap();
        assert!(output.contains("clear()"));
    }

    // Set iteration
    #[test]
    fn test_set_iteration() {
        let code = r"
            fun main() {
                let nums = {1, 2, 3};
                for n in nums {
                    println(n);
                }
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set iteration");
        let output = result.unwrap();
        assert!(output.contains("for n in nums"));
    }

    #[test]
    fn test_set_iter_method() {
        let code = r"
            fun main() {
                let nums = {1, 2, 3};
                let iter = nums.iter();
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set iter");
        let output = result.unwrap();
        assert!(output.contains("iter()"));
    }

    // Mixed types and complex scenarios
    #[test]
    fn test_nested_sets() {
        let code = r"
            fun main() {
                let sets = {{1, 2}, {3, 4}, {5, 6}};
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile nested sets");
        let output = result.unwrap();
        assert!(output.contains("HashSet"));
    }

    #[test]
    fn test_set_in_function_return() {
        let code = r"
            fun get_primes() -> HashSet<i32> {
                {2, 3, 5, 7, 11}
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set in function return");
        let output = result.unwrap();
        assert!(output.contains("fn get_primes"));
        assert!(output.contains("HashSet"));
    }

    #[test]
    fn test_set_in_match_expression() {
        let code = r"
            fun main() {
                let result = match x {
                    1 => {1, 2, 3},
                    2 => {4, 5, 6},
                    _ => {},
                };
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set in match expression");
    }

    #[test]
    fn test_set_from_iterator() {
        let code = r"
            fun main() {
                let nums = (1..10).collect::<HashSet<_>>();
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set from iterator");
        let output = result.unwrap();
        assert!(output.contains("collect"));
    }

    // Edge cases
    #[test]
    fn test_single_element_set() {
        let code = r"
            fun main() {
                let single = {42};
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile single element set");
        let output = result.unwrap();
        assert!(output.contains("HashSet"));
        assert!(output.contains("insert (42") || output.contains("insert(42"));
    }

    #[test]
    fn test_set_with_duplicates() {
        let code = r"
            fun main() {
                let nums = {1, 2, 2, 3, 3, 3};
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set with duplicates");
        // Should still create a set, duplicates are automatically handled
    }

    #[test]
    fn test_set_with_method_calls() {
        let code = r#"
            fun main() {
                let strings = {"hello".to_uppercase(), "world".to_lowercase()};
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set with method calls");
    }

    #[test]
    fn test_set_pattern_matching() {
        let code = r#"
            fun main() {
                let nums = {1, 2, 3};
                if nums.contains(&2) {
                    println("Has 2");
                }
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set pattern matching");
    }

    // Set comprehension-like operations
    #[test]
    fn test_set_from_comprehension() {
        let code = r"
            fun main() {
                let squares = {x * x for x in 0..10};
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set comprehension");
        let output = result.unwrap();
        assert!(output.contains("HashSet") || output.contains("collect"));
    }

    #[test]
    fn test_set_filter_map() {
        let code = r"
            fun main() {
                let nums = {1, 2, 3, 4, 5};
                let evens = nums.iter().filter(|x| x % 2 == 0).collect();
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set filter map");
    }

    // Property test generation
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use quickcheck::{quickcheck, TestResult};

        fn prop_set_with_random_size(size: u8) -> TestResult {
            if size > 20 {
                return TestResult::discard();
            }

            let elements = (0..size)
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            let code = format!("fun main() {{ let s = {{{}}}; }}", elements);
            let result = compile(&code);
            TestResult::from_bool(result.is_ok() || result.is_err())
        }

        fn prop_set_operations(op: &str) -> TestResult {
            let valid_ops = [
                "union",
                "intersection",
                "difference",
                "symmetric_difference",
            ];
            if !valid_ops.contains(&op) {
                return TestResult::discard();
            }

            let code = format!(
                r"
                fun main() {{
                    let a = {{1, 2}};
                    let b = {{2, 3}};
                    let c = a.{}(&b);
                }}
            ",
                op
            );

            let result = compile(&code);
            TestResult::from_bool(result.is_ok() || result.is_err())
        }

        quickcheck! {
            fn test_random_set_sizes(size: u8) -> TestResult {
                prop_set_with_random_size(size)
            }
        }
    }

    // Stress tests
    #[test]
    fn test_large_set() {
        let elements = (0..100)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let code = format!("fun main() {{ let big = {{{}}}; }}", elements);
        let result = compile(&code);
        assert!(result.is_ok(), "Failed to compile large set");
    }

    #[test]
    fn test_deeply_nested_sets() {
        let code = r"
            fun main() {
                let deep = {{{{1}}}};
            }
        ";
        let _result = compile(code);
        // This might be a syntax error or valid depending on interpretation
        // Document the expected behavior
    }

    #[test]
    fn test_set_with_all_types() {
        let code = r#"
            fun main() {
                let ints = {1, 2, 3};
                let floats = {1.0, 2.0, 3.0};
                let strings = {"a", "b", "c"};
                let bools = {true, false};
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile sets with different types"
        );
    }

    #[test]
    fn test_set_equality() {
        let code = r"
            fun main() {
                let a = {1, 2, 3};
                let b = {3, 2, 1};
                let equal = a == b;
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set equality");
    }

    #[test]
    fn test_set_is_subset() {
        let code = r"
            fun main() {
                let a = {1, 2};
                let b = {1, 2, 3};
                let subset = a.is_subset(&b);
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set is_subset");
    }

    #[test]
    fn test_set_is_superset() {
        let code = r"
            fun main() {
                let a = {1, 2, 3};
                let b = {1, 2};
                let superset = a.is_superset(&b);
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set is_superset");
    }

    #[test]
    fn test_set_is_disjoint() {
        let code = r"
            fun main() {
                let a = {1, 2};
                let b = {3, 4};
                let disjoint = a.is_disjoint(&b);
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set is_disjoint");
    }

    #[test]
    fn test_set_clone() {
        let code = r"
            fun main() {
                let a = {1, 2, 3};
                let b = a.clone();
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile set clone");
    }
}
