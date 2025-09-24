// Integration tests for confirmed working features
// These test end-to-end compilation of real code patterns

use ruchy::compile;

#[cfg(test)]
mod integration_tests {
    use super::*;

    // List comprehension tests
    #[test]
    fn test_simple_list_comp_e2e() {
        let code = "fn main() { let squares = [x * x for x in 0..5]; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_filtered_list_comp_e2e() {
        let code = "fn main() { let evens = [x for x in 0..10 if x % 2 == 0]; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_mapped_list_comp_e2e() {
        let code = "fn main() { let doubled = [x * 2 for x in [1, 2, 3]]; }";
        assert!(compile(code).is_ok());
    }

    // Function tests
    #[test]
    fn test_simple_function_e2e() {
        let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_function_with_return_e2e() {
        let code = "fn square(x: i32) -> i32 { return x * x }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_recursive_function_e2e() {
        let code = "fn fact(n: i32) -> i32 { if n <= 1 { 1 } else { n * fact(n - 1) } }";
        assert!(compile(code).is_ok());
    }

    // Control flow tests
    #[test]
    fn test_if_else_e2e() {
        let code = "fn main() { let x = 5; if x > 0 { println(\"positive\") } else { println(\"negative\") } }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_match_expression_e2e() {
        let code = r#"
            fn main() {
                let x = 2;
                match x {
                    1 => println("one"),
                    2 => println("two"),
                    _ => println("other"),
                }
            }
        "#;
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_for_loop_e2e() {
        let code = "fn main() { for i in 0..10 { println(i) } }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_while_loop_e2e() {
        let code = "fn main() { let mut x = 0; while x < 10 { x = x + 1 } }";
        assert!(compile(code).is_ok());
    }

    // String tests
    #[test]
    fn test_string_literal_e2e() {
        let code = r#"fn main() { let s = "Hello, world!"; }"#;
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_string_concatenation_e2e() {
        let code = r#"fn main() { let s = "Hello" + ", " + "world!"; }"#;
        assert!(compile(code).is_ok());
    }

    // Array/List tests
    #[test]
    fn test_array_literal_e2e() {
        let code = "fn main() { let arr = [1, 2, 3, 4, 5]; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_array_indexing_e2e() {
        let code = "fn main() { let arr = [1, 2, 3]; let first = arr[0]; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_array_methods_e2e() {
        let code = "fn main() { let arr = [1, 2, 3]; let len = arr.len(); }";
        assert!(compile(code).is_ok());
    }

    // Tuple tests
    #[test]
    fn test_tuple_literal_e2e() {
        let code = "fn main() { let t = (1, \"hello\", true); }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_tuple_destructuring_e2e() {
        let code = "fn main() { let (x, y) = (1, 2); }";
        assert!(compile(code).is_ok());
    }

    // Object/Struct tests
    #[test]
    fn test_object_literal_e2e() {
        let code = "fn main() { let obj = { x: 10, y: 20 }; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_object_field_access_e2e() {
        let code = "fn main() { let obj = { x: 10, y: 20 }; let val = obj.x; }";
        assert!(compile(code).is_ok());
    }

    // Type annotation tests
    #[test]
    fn test_type_annotations_e2e() {
        let code = "fn main() { let x: i32 = 42; let s: String = \"hello\"; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    #[ignore = "vec! macro not yet implemented"]
    fn test_generic_types_e2e() {
        let code = "fn main() { let v: Vec<i32> = vec![1, 2, 3]; }";
        assert!(compile(code).is_ok());
    }

    // Operator tests
    #[test]
    fn test_arithmetic_operators_e2e() {
        let code = "fn main() { let x = 10 + 5 * 2 - 3 / 1 % 4; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_comparison_operators_e2e() {
        let code = "fn main() { let b = 5 > 3 && 2 < 4 || 1 == 1 && 2 != 3; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_bitwise_operators_e2e() {
        let code = "fn main() { let x = 5 & 3 | 2 ^ 1 << 2 >> 1; }";
        assert!(compile(code).is_ok());
    }

    // Method call tests
    #[test]
    fn test_method_chaining_e2e() {
        let code = "fn main() { let s = \"hello\".to_uppercase().trim().len(); }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_iterator_methods_e2e() {
        let code = "fn main() { let sum = [1, 2, 3].iter().sum(); }";
        assert!(compile(code).is_ok());
    }

    // Closure tests
    #[test]
    fn test_simple_closure_e2e() {
        let code = "fn main() { let add = |a, b| a + b; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_closure_with_capture_e2e() {
        let code = "fn main() { let x = 10; let add_x = |y| x + y; }";
        assert!(compile(code).is_ok());
    }

    // Range tests
    #[test]
    fn test_range_exclusive_e2e() {
        let code = "fn main() { let r = 0..10; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_range_inclusive_e2e() {
        let code = "fn main() { let r = 0..=10; }";
        assert!(compile(code).is_ok());
    }
}
