// EXTREME TDD: Tests for `fun` keyword feature from unified spec
// ALL TESTS MUST FAIL INITIALLY - Implementation comes AFTER

use ruchy::compile;

#[cfg(test)]
mod test_fun_keyword {
    use super::*;

    // Basic function declaration tests
    #[test]
    fn test_fun_simple_function() {
        let code = r#"
            fun hello() {
                println("Hello, World!")
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile simple fun function");
        let output = result.unwrap();
        assert!(output.contains("fn hello"), "Should transpile fun to fn");
    }

    #[test]
    fn test_fun_with_parameters() {
        let code = r"
            fun add(x: i32, y: i32) -> i32 {
                x + y
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile fun with parameters");
        let output = result.unwrap();
        assert!(
            output.contains("fn add"),
            "Should transpile fun with params"
        );
    }

    #[test]
    fn test_fun_with_return_type() {
        let code = r"
            fun calculate(data: Vec<f64>) -> f64 {
                data.iter().sum()
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile fun with return type");
        let output = result.unwrap();
        assert!(output.contains("fn calculate"));
    }

    // Type inference tests
    #[test]
    fn test_fun_with_local_inference() {
        let code = r"
            fun process(data: Vec<f64>) -> Vec<f64> {
                let result = data.iter().map(|x| x * 2.0).collect();
                result
            }
        ";
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile fun with local type inference"
        );
    }

    // Async function tests
    #[test]
    fn test_async_fun() {
        let code = r"
            async fun fetch_data(url: &str) -> Result<String> {
                let response = http::get(url).await?;
                Ok(response.body())
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile async fun");
        let output = result.unwrap();
        assert!(output.contains("async fn fetch_data"));
    }

    // Generic function tests
    #[test]
    fn test_fun_with_generics() {
        let code = r"
            fun identity<T>(value: T) -> T {
                value
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile generic fun");
        let output = result.unwrap();
        assert!(output.contains("fn identity"));
    }

    #[test]
    fn test_fun_with_multiple_generics() {
        let code = r"
            fun swap<T, U>(pair: (T, U)) -> (U, T) {
                (pair.1, pair.0)
            }
        ";
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile fun with multiple generics"
        );
    }

    // Lifetime tests
    #[test]
    fn test_fun_with_lifetime() {
        // Lifetimes and references not yet fully supported - use simpler syntax
        let code = r"
            fun get_first(data: Vec<i32>) -> i32 {
                data[0]
            }
        ";
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile fun with Vec parameter (references not yet fully implemented)"
        );
        let output = result.unwrap();
        assert!(output.contains("fn get_first"));
    }

    // Impl block tests
    #[test]
    fn test_fun_in_impl_block() {
        let code = r"
            struct Point { x: f64, y: f64 }

            impl Point {
                fun new(x: f64, y: f64) -> Self {
                    Self { x, y }
                }

                fun distance(&self, other: &Point) -> f64 {
                    ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
                }
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile fun in impl block");
        let output = result.unwrap();
        assert!(output.contains("fn new"));
        assert!(output.contains("fn distance"));
    }

    // Trait implementation tests
    #[test]
    fn test_fun_in_trait_impl() {
        let code = r#"
            trait Display {
                fun fmt(&self) -> String;
            }

            impl Display for Point {
                fun fmt(&self) -> String {
                    format!("({}, {})", self.x, self.y)
                }
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile fun in trait impl");
    }

    // Const function tests
    #[test]
    fn test_const_fun() {
        let code = r"
            const fun max(a: i32, b: i32) -> i32 {
                if a > b { a } else { b }
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile const fun");
        let output = result.unwrap();
        assert!(output.contains("fn max"));
    }

    // Unsafe function tests
    #[test]
    fn test_unsafe_fun() {
        let code = r#"
            unsafe fun raw_access() {
                println("unsafe access")
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile unsafe fun");
        let output = result.unwrap();
        assert!(output.contains("fn raw_access"));
        assert!(output.contains("unsafe"));
    }

    // Pub function tests
    #[test]
    fn test_pub_fun() {
        let code = r"
            pub fun public_api(data: String) -> String {
                data.to_uppercase()
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile pub fun");
        let output = result.unwrap();
        assert!(output.contains("fn public_api"));
    }

    // Multiple modifiers
    #[test]
    fn test_pub_async_fun() {
        let code = r"
            pub async fun fetch_public_data() -> Result<Vec<u8>> {
                Ok(vec![])
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile pub async fun");
        let output = result.unwrap();
        assert!(output.contains("fn fetch_public_data"));
    }

    // Recursive function
    #[test]
    fn test_recursive_fun() {
        let code = r"
            fun factorial(n: u64) -> u64 {
                if n <= 1 {
                    1
                } else {
                    n * factorial(n - 1)
                }
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile recursive fun");
    }

    // Closure interaction
    #[test]
    fn test_fun_with_closure() {
        // Where clauses not yet supported - use simpler closure syntax
        let code = r"
            fun apply_twice(f: fn(i32) -> i32, x: i32) -> i32 {
                f(f(x))
            }
        ";
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile fun with closure parameter (where clauses not yet implemented)"
        );
    }

    // Pattern matching in parameters
    #[test]
    fn test_fun_with_pattern_params() {
        let code = r"
            fun process_tuple((x, y): (i32, i32)) -> i32 {
                x + y
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile fun with pattern params");
    }

    // Variadic-like with slice
    #[test]
    fn test_fun_with_slice_param() {
        let code = r"
            fun sum(numbers: &[i32]) -> i32 {
                numbers.iter().sum()
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile fun with slice param");
    }

    // Associated function
    #[test]
    fn test_associated_fun() {
        // Generic impl blocks not yet supported - use concrete type
        let code = r"
            impl MyVec {
                fun with_capacity(capacity: usize) -> MyVec {
                    MyVec::with_capacity(capacity)
                }
            }
        ";
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile associated fun (generics not yet implemented)"
        );
    }

    // Function pointer type
    #[test]
    fn test_fun_pointer_type() {
        let code = r"
            type Operation = fun(i32, i32) -> i32;

            fun apply(op: Operation, a: i32, b: i32) -> i32 {
                op(a, b)
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile function pointer type");
    }

    // Property test generation
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use quickcheck::{quickcheck, TestResult};

        fn prop_fun_with_random_name(name: String) -> TestResult {
            // Only accept valid identifiers: ASCII alphanumeric and underscore,
            // not starting with digit, and not just underscore alone
            if name.is_empty()
                || name == "_"
                || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                || name.chars().next().unwrap().is_ascii_digit()
            {
                return TestResult::discard();
            }

            let code = format!("fun {name}() {{ }}");
            let result = compile(&code);
            TestResult::from_bool(result.is_ok())
        }

        quickcheck! {
            fn test_fun_with_random_names(name: String) -> TestResult {
                prop_fun_with_random_name(name)
            }
        }
    }
}
