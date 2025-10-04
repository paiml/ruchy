//! EXTREME TDD: Property tests for Try/Catch with 10,000 iterations
//! Testing invariants and edge cases with random inputs

use proptest::prelude::*;
use ruchy::compile;

// Property: Try blocks always compile successfully
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_try_catch_never_panics(value: i32) {
        let code = format!(
            r"
            fun main() {{
                try {{
                    {value}
                }} catch (e) {{
                    -1
                }}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Try-catch should always compile");
    }
}

// Property: Throw statements always compile
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_throw_always_compiles(message: String) {
        // Filter out messages that could cause parsing issues
        let clean_message = message.chars()
            .filter(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace() || *c == '_')
            .take(50)
            .collect::<String>();

        if clean_message.is_empty() {
            return Ok(());
        }

        let code = format!(
            r#"
            fun main() {{
                throw "{}"
            }}
            "#,
            clean_message.replace('"', "'")
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Throw statements should always compile");

        if let Ok(output) = result {
            prop_assert!(output.contains("panic !"), "Throw should transpile to panic!");
        }
    }
}

// Property: Try-catch with different error types
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_try_catch_with_error_values(error_value: i32, recovery_value: i32) {
        let code = format!(
            r"
            fun main() {{
                try {{
                    throw {error_value}
                }} catch (e) {{
                    {recovery_value}
                }}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Try-catch with error values should compile");

        if let Ok(output) = result {
            prop_assert!(output.contains("Result"), "Should use Result type");
            prop_assert!(output.contains("match"), "Should use pattern matching");
        }
    }
}

// Property: Nested try-catch blocks
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_nested_try_catch(depth: u8, value: i32) {
        let depth = (depth % 3) + 1; // Limit depth to 1-3

        let mut code = format!("try {{ {value} }} catch (e) {{ -1 }}");
        for _ in 1..depth {
            code = format!("try {{ {code} }} catch (e) {{ -1 }}");
        }

        let full_code = format!(
            r"
            fun main() {{
                {code}
            }}
            "
        );

        let result = compile(&full_code);
        prop_assert!(result.is_ok(), "Nested try-catch should compile");

        if let Ok(output) = result {
            let result_count = output.matches("Result").count();
            prop_assert!(result_count >= depth as usize, "Should have nested Result types");
        }
    }
}

// Property: Try-catch-finally blocks
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_try_catch_finally(try_value: i32, catch_value: i32, finally_value: i32) {
        let code = format!(
            r"
            fun main() {{
                try {{
                    {try_value}
                }} catch (e) {{
                    {catch_value}
                }} finally {{
                    {finally_value}
                }}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Try-catch-finally should compile");
    }
}

// Property: Try blocks with expressions
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_try_with_expressions(a: i32, b: i32) {
        // Avoid division by zero
        let b = if b == 0 { 1 } else { b };

        let code = format!(
            r"
            fun main() {{
                try {{
                    {a} + {b} * {a} / {b}
                }} catch (e) {{
                    0
                }}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Try with expressions should compile");
    }
}

// Property: Multiple catch clauses
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_multiple_catch_clauses(values: Vec<i32>) {
        let values = values.into_iter().take(5).collect::<Vec<_>>();

        if values.len() < 2 {
            return Ok(());
        }

        let try_value = values[0];
        let catch_clauses = values[1..].iter()
            .enumerate()
            .map(|(i, &val)| format!("catch (e{i}) {{ {val} }}"))
            .collect::<Vec<_>>()
            .join(" ");

        let code = format!(
            r"
            fun main() {{
                try {{
                    {try_value}
                }} {catch_clauses}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Multiple catch clauses should compile");
    }
}

// Property: Try-catch in functions
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_try_catch_in_function(param: i32, error_val: i32, success_val: i32) {
        let code = format!(
            r"
            fun safe_operation(x) {{
                try {{
                    if x < 0 {{
                        throw {error_val}
                    }}
                    {success_val} + x
                }} catch (e) {{
                    {error_val}
                }}
            }}

            fun main() {{
                safe_operation({param})
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Try-catch in function should compile");
    }
}

// Property: Try-catch with variable bindings
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_try_catch_with_variables(var_name: String, value: i32) {
        // Generate valid identifier - must start with letter, only ASCII alphanumeric + underscore
        let cleaned = var_name.chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
            .take(20)
            .collect::<String>();

        let var_name = if cleaned.is_empty() || cleaned == "_" {
            "temp_var".to_string()
        } else if cleaned.chars().next().unwrap().is_ascii_digit() {
            format!("var_{cleaned}")
        } else {
            cleaned
        };

        // Additional safety check
        if var_name.is_empty() || var_name == "_" || var_name.chars().next().unwrap().is_ascii_digit() {
            return Ok(());
        }

        let code = format!(
            r"
            fun main() {{
                let {var_name} = {value};
                try {{
                    {var_name} + 1
                }} catch (e) {{
                    0
                }}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Try-catch with variables should compile");
    }
}

// Property: Empty try blocks
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_empty_try_block(catch_value: i32) {
        let code = format!(
            r"
            fun main() {{
                try {{
                    // Empty block
                }} catch (e) {{
                    {catch_value}
                }}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Empty try blocks should compile");
    }
}

// Property: Try-catch with complex expressions
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_complex_try_expressions(elements: Vec<i32>) {
        let elements = elements.into_iter().take(10).collect::<Vec<_>>();

        if elements.is_empty() {
            return Ok(());
        }

        let array_literal = format!("[{}]", elements.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", "));

        let code = format!(
            r"
            fun main() {{
                let arr = {array_literal};
                try {{
                    arr[0] + arr[1]
                }} catch (e) {{
                    -1
                }}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Complex try expressions should compile");
    }
}

// Property: String throw values
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_string_throw_values(msg_parts: Vec<String>) {
        let clean_parts: Vec<String> = msg_parts.into_iter()
            .take(3)
            .filter(|s| !s.is_empty())
            .map(|s| s.chars()
                .filter(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace())
                .take(20)
                .collect::<String>())
            .filter(|s| !s.trim().is_empty())
            .collect();

        if clean_parts.is_empty() {
            return Ok(());
        }

        let message = clean_parts.join(" ");
        let code = format!(
            r#"
            fun main() {{
                try {{
                    throw "{}"
                }} catch (e) {{
                    "error handled"
                }}
            }}
            "#,
            message.replace('"', "'")
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "String throw values should compile");
    }
}

// Property: Numeric throw values
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_numeric_throw_values(error_code: i32, recovery_code: i32) {
        let code = format!(
            r"
            fun main() {{
                try {{
                    throw {error_code}
                }} catch (e) {{
                    {recovery_code}
                }}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Numeric throw values should compile");
    }
}
