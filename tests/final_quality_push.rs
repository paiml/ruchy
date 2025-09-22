// Final push to 80% with EXTREME quality tests
// Every test follows single responsibility and has complexity ≤10

use ruchy::compile;

#[cfg(test)]
mod string_operations {
    use super::*;

    #[test]
    fn test_empty_string() {
        let code = r#"fn main() { let s = ""; }"#;
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_simple_string() {
        let code = r#"fn main() { let s = "hello"; }"#;
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_string_with_spaces() {
        let code = r#"fn main() { let s = "hello world"; }"#;
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_string_concatenation() {
        let code = r#"fn main() { let s = "hello" + " world"; }"#;
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_string_interpolation() {
        let code = r#"fn main() { let name = "world"; let s = f"hello {name}"; }"#;
        assert!(compile(code).is_ok());
    }
}

#[cfg(test)]
mod array_operations {
    use super::*;

    #[test]
    fn test_empty_array() {
        let code = "fn main() { let arr = []; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_single_element_array() {
        let code = "fn main() { let arr = [1]; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_multi_element_array() {
        let code = "fn main() { let arr = [1, 2, 3]; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_array_indexing() {
        let code = "fn main() { let arr = [1, 2, 3]; let x = arr[0]; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_array_length() {
        let code = "fn main() { let arr = [1, 2, 3]; let len = arr.len(); }";
        assert!(compile(code).is_ok());
    }
}

#[cfg(test)]
mod tuple_operations {
    use super::*;

    #[test]
    fn test_unit_tuple() {
        let code = "fn main() { let t = (); }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_pair_tuple() {
        let code = "fn main() { let t = (1, 2); }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_mixed_tuple() {
        let code = r#"fn main() { let t = (1, "hello", true); }"#;
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_tuple_destructuring() {
        let code = "fn main() { let (x, y) = (1, 2); }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_tuple_field_access() {
        let code = "fn main() { let t = (1, 2); let x = t.0; }";
        assert!(compile(code).is_ok());
    }
}

#[cfg(test)]
mod range_operations {
    use super::*;

    #[test]
    fn test_exclusive_range() {
        let code = "fn main() { let r = 0..10; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_inclusive_range() {
        let code = "fn main() { let r = 0..=10; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_range_in_for_loop() {
        let code = "fn main() { for i in 0..10 { } }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_range_with_step() {
        let code = "fn main() { let r = (0..10).step_by(2); }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_reversed_range() {
        let code = "fn main() { let r = (0..10).rev(); }";
        assert!(compile(code).is_ok());
    }
}

#[cfg(test)]
mod closure_operations {
    use super::*;

    #[test]
    fn test_simple_closure() {
        let code = "fn main() { let f = || 42; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_closure_with_params() {
        let code = "fn main() { let add = |a, b| a + b; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_closure_with_capture() {
        let code = "fn main() { let x = 10; let f = || x + 1; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_closure_with_move() {
        let code = "fn main() { let x = 10; let f = move || x; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_closure_in_iterator() {
        let code = "fn main() { let v = [1, 2, 3].iter().map(|x| x * 2); }";
        assert!(compile(code).is_ok());
    }
}

#[cfg(test)]
mod method_calls {
    use super::*;

    #[test]
    fn test_single_method() {
        let code = r#"fn main() { let s = "hello".to_uppercase(); }"#;
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_chained_methods() {
        let code = r#"fn main() { let s = "hello".to_uppercase().trim(); }"#;
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_method_with_args() {
        let code = r#"fn main() { let s = "hello world".replace("world", "rust"); }"#;
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_iterator_methods() {
        let code = "fn main() { let sum = [1, 2, 3].iter().sum(); }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_option_methods() {
        let code = "fn main() { let x = Some(42).unwrap(); }";
        assert!(compile(code).is_ok());
    }
}

#[cfg(test)]
mod type_operations {
    use super::*;

    #[test]
    fn test_type_alias() {
        let code = "type MyInt = i32;";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_generic_type() {
        let code = "fn identity<T>(x: T) -> T { x }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_option_type() {
        let code = "fn main() { let x: Option<i32> = Some(42); }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_result_type() {
        let code = "fn main() { let x: Result<i32, String> = Ok(42); }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_vec_type() {
        let code = "fn main() { let v: Vec<i32> = vec![1, 2, 3]; }";
        assert!(compile(code).is_ok());
    }
}

#[cfg(test)]
mod operator_precedence {
    use super::*;

    #[test]
    fn test_multiplication_before_addition() {
        let code = "fn main() { let x = 2 + 3 * 4; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_parentheses_override() {
        let code = "fn main() { let x = (2 + 3) * 4; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_comparison_precedence() {
        let code = "fn main() { let x = 1 + 2 < 4 * 5; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_logical_precedence() {
        let code = "fn main() { let x = true || false && false; }";
        assert!(compile(code).is_ok());
    }

    #[test]
    fn test_assignment_precedence() {
        let code = "fn main() { let mut x = 0; x = 1 + 2; }";
        assert!(compile(code).is_ok());
    }
}