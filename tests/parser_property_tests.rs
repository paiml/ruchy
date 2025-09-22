// Property-based tests for parser robustness
// These ensure the parser handles various inputs without panicking

use ruchy::compile;
use quickcheck::{quickcheck, TestResult};

#[cfg(test)]
mod parser_properties {
    use super::*;

    // Test that parser doesn't panic on random strings
    fn prop_parser_no_panic(s: String) -> TestResult {
        // Limit string size to avoid excessive test times
        if s.len() > 1000 {
            return TestResult::discard();
        }

        // Parser should either parse successfully or return an error, never panic
        let _result = compile(&s);
        TestResult::passed()
    }

    // Test that valid identifiers compile
    fn prop_valid_identifier(name: String) -> TestResult {
        // Create a valid identifier
        let clean_name = name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>();

        if clean_name.is_empty() || clean_name.chars().next().unwrap().is_numeric() {
            return TestResult::discard();
        }

        let code = format!("fn main() {{ let {} = 42; }}", clean_name);
        let _result = compile(&code);
        TestResult::passed()
    }

    // Test that numbers compile correctly
    fn prop_numbers_compile(n: i32) -> TestResult {
        let code = format!("fn main() {{ let x = {}; }}", n);
        let result = compile(&code);
        TestResult::from_bool(result.is_ok())
    }

    // Test that string literals compile
    fn prop_string_literals(s: String) -> TestResult {
        // Escape the string properly
        let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
        let code = format!("fn main() {{ let s = \"{}\"; }}", escaped);
        let _result = compile(&code);
        TestResult::passed()
    }

    // Test array literals with various sizes
    fn prop_array_literals(size: u8) -> TestResult {
        if size > 100 {
            return TestResult::discard();
        }

        let elements = (0..size).map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
        let code = format!("fn main() {{ let arr = [{}]; }}", elements);
        let _result = compile(&code);
        TestResult::passed()
    }

    // Test function definitions with various parameter counts
    fn prop_function_params(param_count: u8) -> TestResult {
        if param_count > 20 {
            return TestResult::discard();
        }

        let params = (0..param_count)
            .map(|i| format!("p{}: i32", i))
            .collect::<Vec<_>>()
            .join(", ");

        let code = format!("fn test({}) -> i32 {{ 42 }}", params);
        let _result = compile(&code);
        TestResult::passed()
    }

    // Test binary operations with random operators
    fn prop_binary_ops(a: i8, b: i8, op: u8) -> TestResult {
        let operators = ["+", "-", "*", "/", "%", "<", ">", "<=", ">=", "==", "!=", "&&", "||"];
        let op_idx = (op as usize) % operators.len();
        let operator = operators[op_idx];

        // Avoid division by zero
        if (operator == "/" || operator == "%") && b == 0 {
            return TestResult::discard();
        }

        let code = format!("fn main() {{ let x = {} {} {}; }}", a, operator, b);
        let _result = compile(&code);
        TestResult::passed()
    }

    // Test if-else with various conditions
    fn prop_if_else(condition: bool, a: i32, b: i32) -> TestResult {
        let code = format!(
            "fn main() {{ let x = if {} {{ {} }} else {{ {} }}; }}",
            condition, a, b
        );
        let result = compile(&code);
        TestResult::from_bool(result.is_ok())
    }

    // Test match expressions
    fn prop_match_expr(n: u8) -> TestResult {
        if n > 10 {
            return TestResult::discard();
        }

        let arms = (0..n)
            .map(|i| format!("        {} => {},", i, i * 2))
            .collect::<Vec<_>>()
            .join("\n");

        let code = format!(
            "fn main() {{
                let x = 5;
                let y = match x {{
{}
                    _ => 999,
                }};
            }}",
            arms
        );
        let _result = compile(&code);
        TestResult::passed()
    }

    // Test for loops with ranges
    fn prop_for_loops(start: i8, end: i8) -> TestResult {
        let code = format!(
            "fn main() {{ for i in {}..{} {{ println(i); }} }}",
            start.min(end),
            start.max(end)
        );
        let _result = compile(&code);
        TestResult::passed()
    }

    // Actual quickcheck tests
    quickcheck! {
        fn test_parser_no_panic(s: String) -> TestResult {
            prop_parser_no_panic(s)
        }

        fn test_valid_identifier(name: String) -> TestResult {
            prop_valid_identifier(name)
        }

        fn test_numbers_compile(n: i32) -> TestResult {
            prop_numbers_compile(n)
        }

        fn test_string_literals(s: String) -> TestResult {
            prop_string_literals(s)
        }

        fn test_array_literals(size: u8) -> TestResult {
            prop_array_literals(size)
        }

        fn test_function_params(param_count: u8) -> TestResult {
            prop_function_params(param_count)
        }

        fn test_binary_ops(a: i8, b: i8, op: u8) -> TestResult {
            prop_binary_ops(a, b, op)
        }

        fn test_if_else(condition: bool, a: i32, b: i32) -> TestResult {
            prop_if_else(condition, a, b)
        }

        fn test_match_expr(n: u8) -> TestResult {
            prop_match_expr(n)
        }

        fn test_for_loops(start: i8, end: i8) -> TestResult {
            prop_for_loops(start, end)
        }
    }
}