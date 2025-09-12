//! Comprehensive property tests for all parser grammar rules
//! Target: 10,000+ iterations per test to ensure parser robustness

#[cfg(test)]
mod parser_property_tests {
    use proptest::prelude::*;
    use crate::frontend::Parser;
    
    // ========== Generators for valid Ruchy code ==========
    
    /// Generate valid identifiers
    fn identifier_strategy() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9_]{0,10}".prop_filter("Not a keyword", |s| {
            !matches!(s.as_str(), "let" | "fun" | "if" | "else" | "match" | 
                     "for" | "while" | "return" | "break" | "continue" |
                     "true" | "false" | "null" | "var" | "const")
        })
    }
    
    /// Generate valid integers
    fn integer_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            "[0-9]{1,10}",
            "-[1-9][0-9]{0,9}",
            "0x[0-9a-fA-F]{1,8}",
            "0o[0-7]{1,10}",
            "0b[01]{1,32}",
        ]
    }
    
    /// Generate valid floats
    fn float_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            "[0-9]{1,5}\\.[0-9]{1,5}",
            "-[0-9]{1,5}\\.[0-9]{1,5}",
            "[0-9]{1,5}\\.[0-9]{1,5}e[+-]?[0-9]{1,2}",
        ]
    }
    
    /// Generate valid string literals
    fn string_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            "\"[a-zA-Z0-9 ]{0,20}\"",
            "'[a-zA-Z0-9 ]{0,20}'",
            "\"\"",
            "''",
        ]
    }
    
    /// Generate valid character literals
    fn char_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            "'[a-zA-Z]'",
            "'[0-9]'",
            "'\\\\n'",
            "'\\\\t'",
            "'\\\\r'",
            "'\\\\0'",
        ]
    }
    
    /// Generate valid binary operators
    fn binop_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("+".to_string()),
            Just("-".to_string()),
            Just("*".to_string()),
            Just("/".to_string()),
            Just("%".to_string()),
            Just("==".to_string()),
            Just("!=".to_string()),
            Just("<".to_string()),
            Just(">".to_string()),
            Just("<=".to_string()),
            Just(">=".to_string()),
            Just("&&".to_string()),
            Just("||".to_string()),
            Just("|>".to_string()),
        ]
    }
    
    // ========== Property Tests for Core Grammar Rules ==========
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]
        
        /// Test that parser never panics on any input
        #[test]
        fn parser_never_panics(input: String) {
            let mut parser = Parser::new(&input);
            let _ = parser.parse(); // Should not panic
        }
        
        /// Test that valid literals always parse successfully
        #[test]
        fn valid_literals_parse(
            int in integer_strategy(),
            float in float_strategy(),
            string in string_strategy(),
            char_lit in char_strategy(),
        ) {
            // Integer literals
            let mut parser = Parser::new(&int);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse integer: {}", int);
            
            // Float literals
            let mut parser = Parser::new(&float);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse float: {}", float);
            
            // String literals
            let mut parser = Parser::new(&string);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse string: {}", string);
            
            // Character literals
            let mut parser = Parser::new(&char_lit);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse char: {}", char_lit);
        }
        
        /// Test variable declarations
        #[test]
        fn variable_declarations_parse(
            name in identifier_strategy(),
            value in integer_strategy(),
        ) {
            let code = format!("let {} = {}", name, value);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        }
        
        /// Test function declarations
        #[test]
        fn function_declarations_parse(
            name in identifier_strategy(),
            param1 in identifier_strategy(),
            param2 in identifier_strategy(),
            body_val in integer_strategy(),
        ) {
            let code = format!("fun {}({}, {}) {{ {} }}", name, param1, param2, body_val);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        }
        
        /// Test binary expressions
        #[test]
        fn binary_expressions_parse(
            left in integer_strategy(),
            op in binop_strategy(),
            right in integer_strategy(),
        ) {
            let code = format!("{} {} {}", left, op, right);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        }
        
        /// Test if expressions
        #[test]
        fn if_expressions_parse(
            cond_var in identifier_strategy(),
            then_val in integer_strategy(),
            else_val in integer_strategy(),
        ) {
            let code = format!("if {} {{ {} }} else {{ {} }}", cond_var, then_val, else_val);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        }
        
        /// Test match expressions
        #[test]
        fn match_expressions_parse(
            var in identifier_strategy(),
            pattern1 in integer_strategy(),
            result1 in string_strategy(),
            result2 in string_strategy(),
        ) {
            let code = format!("match {} {{ {} => {}, _ => {} }}", 
                             var, pattern1, result1, result2);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        }
        
        /// Test list literals
        #[test]
        fn list_literals_parse(values: Vec<u8>) {
            let values_str = values.iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let code = format!("[{}]", values_str);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        }
        
        /// Test tuple literals
        #[test]
        fn tuple_literals_parse(
            val1 in integer_strategy(),
            val2 in string_strategy(),
            val3 in identifier_strategy(),
        ) {
            let code = format!("({}, {}, {})", val1, val2, val3);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        }
        
        /// Test object literals
        #[test]
        fn object_literals_parse(
            key1 in identifier_strategy(),
            val1 in integer_strategy(),
            key2 in identifier_strategy(),
            val2 in string_strategy(),
        ) {
            let code = format!("{{ {}: {}, {}: {} }}", key1, val1, key2, val2);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        }
        
        /// Test for loops
        #[test]
        fn for_loops_parse(
            var in identifier_strategy(),
            start in "0|1|2",
            end in "10|20|30",
            body in integer_strategy(),
        ) {
            let code = format!("for {} in {}..{} {{ {} }}", var, start, end, body);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        }
        
        /// Test while loops
        #[test]
        fn while_loops_parse(
            var in identifier_strategy(),
            limit in "10|20|30",
            body in identifier_strategy(),
        ) {
            let code = format!("while {} < {} {{ {} }}", var, limit, body);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        }
        
        /// Test list comprehensions
        #[test]
        fn list_comprehensions_parse(
            var in identifier_strategy(),
            expr in identifier_strategy(),
            start in "0|1",
            end in "10|20",
        ) {
            let code = format!("[{} * 2 for {} in {}..{}]", expr, var, start, end);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        }
        
        /// Test rest patterns in destructuring
        #[test]
        fn rest_patterns_parse(
            first in identifier_strategy(),
            rest in identifier_strategy(),
            val1 in "1|2|3",
            val2 in "4|5|6",
            val3 in "7|8|9",
        ) {
            let code = format!("let [{}, ...{}] = [{}, {}, {}]", 
                             first, rest, val1, val2, val3);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        }
        
        /// Test tuple destructuring
        #[test]
        fn tuple_destructuring_parse(
            var1 in identifier_strategy(),
            var2 in identifier_strategy(),
            val1 in integer_strategy(),
            val2 in string_strategy(),
        ) {
            let code = format!("let ({}, {}) = ({}, {})", var1, var2, val1, val2);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        }
        
        /// Test pipeline operator
        #[test]
        fn pipeline_operator_parse(
            start_val in integer_strategy(),
            func1 in identifier_strategy(),
            func2 in identifier_strategy(),
        ) {
            let code = format!("{} |> {} |> {}", start_val, func1, func2);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        }
        
        /// Test string interpolation
        #[test]
        fn string_interpolation_parse(
            var in identifier_strategy(),
            text in "[a-zA-Z ]{0,10}",
        ) {
            let code = format!("f\"Hello {{{}}} {}\"", var, text);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        }
        
        /// Test async/await
        #[test]
        fn async_await_parse(
            func_name in identifier_strategy(),
            var in identifier_strategy(),
        ) {
            let code = format!("async fun {}() {{ await {} }}", func_name, var);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        }
        
        /// Test error handling with try/catch
        #[test]
        fn try_catch_parse(
            risky_expr in identifier_strategy(),
            err_var in identifier_strategy(),
            fallback in integer_strategy(),
        ) {
            let code = format!("try {{ {} }} catch {} {{ {} }}", 
                             risky_expr, err_var, fallback);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        }
        
        /// Test method chaining
        #[test]
        fn method_chaining_parse(
            obj in identifier_strategy(),
            method1 in identifier_strategy(),
            method2 in identifier_strategy(),
        ) {
            let code = format!("{}.{}().{}()", obj, method1, method2);
            let mut parser = Parser::new(&code);
            let result = parser.parse();
            prop_assert!(result.is_ok(), "Failed to parse: {}", code);
        }
    }
}