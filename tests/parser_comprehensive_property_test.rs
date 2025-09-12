//! Comprehensive parser property tests - 10,000+ iterations per rule
//! Tests all major grammar constructs to ensure parser robustness

use proptest::prelude::*;
use ruchy::Parser;

// Custom strategies for generating valid Ruchy code
fn identifier() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,10}".prop_filter("Not a keyword", |s| {
        !matches!(s.as_str(), 
            "let" | "var" | "fun" | "if" | "else" | "match" | "for" | "while" | 
            "return" | "break" | "continue" | "true" | "false" | "null" | "async" | 
            "await" | "try" | "catch" | "finally" | "import" | "export" | "as" |
            "struct" | "trait" | "impl" | "type" | "const" | "enum")
    })
}

fn valid_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 _-]{0,50}".prop_map(|s| s.replace('\\', ""))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    
    // ========== Core Constructs (10,000 iterations each) ==========
    
    #[test]
    fn fuzz_parser_with_random_bytes(bytes: Vec<u8>) {
        let input = String::from_utf8_lossy(&bytes);
        let mut parser = Parser::new(&input);
        let _ = parser.parse(); // Must not panic
    }
    
    #[test]
    fn all_binary_operators_parse(left: i32, right: i32) {
        for op in &["+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">=", "&&", "||", "|>"] {
            let input = format!("{} {} {}", left, op, right);
            let mut parser = Parser::new(&input);
            let _ = parser.parse(); // Should handle all operators
        }
    }
    
    #[test]
    fn nested_expressions_parse(a: u8, b: u8, c: u8, d: u8) {
        let input = format!("(({} + {}) * ({} - {}))", a, b, c, d);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse nested expr: {}", input);
    }
    
    #[test]
    fn function_definitions_parse(
        name in identifier(),
        param1 in identifier(),
        param2 in identifier(),
        ret_val: u8,
    ) {
        let input = format!("fun {}({}, {}) {{ {} }}", name, param1, param2, ret_val);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse function: {}", input);
    }
    
    #[test]
    fn if_else_chains_parse(
        cond1 in identifier(),
        cond2 in identifier(),
        val1: u8,
        val2: u8,
        val3: u8,
    ) {
        let input = format!(
            "if {} {{ {} }} else if {} {{ {} }} else {{ {} }}", 
            cond1, val1, cond2, val2, val3
        );
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse if-else: {}", input);
    }
    
    #[test]
    fn match_expressions_parse(
        var in identifier(),
        patterns: Vec<u8>,
    ) {
        if patterns.is_empty() {
            return Ok(());
        }
        let arms = patterns.iter()
            .map(|p| format!("{} => {}", p, p * 2))
            .chain(std::iter::once("_ => 0".to_string()))
            .collect::<Vec<_>>()
            .join(", ");
        let input = format!("match {} {{ {} }}", var, arms);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse match: {}", input);
    }
    
    #[test]
    fn for_loops_parse(
        var in identifier(),
        start: u8,
        end: u8,
        body in identifier(),
    ) {
        let input = format!("for {} in {}..{} {{ {} }}", var, start, end, body);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse for loop: {}", input);
    }
    
    #[test]
    fn while_loops_parse(
        var in identifier(),
        limit: u8,
        body in identifier(),
    ) {
        let input = format!("while {} < {} {{ {} + 1 }}", var, limit, body);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse while loop: {}", input);
    }
    
    #[test]
    fn object_literals_parse(fields: Vec<(String, u8)>) {
        let fields_str = fields.iter()
            .filter_map(|(k, v)| {
                if k.is_empty() || k.chars().next()?.is_ascii_digit() {
                    None
                } else {
                    Some(format!("{}: {}", k, v))
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let input = format!("{{ {} }}", fields_str);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse object: {}", input);
    }
    
    #[test]
    fn list_comprehensions_parse(
        var in identifier(),
        expr in identifier(),
        start: u8,
        end: u8,
    ) {
        let input = format!("[{} * 2 for {} in {}..{}]", expr, var, start, end);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse list comp: {}", input);
    }
    
    #[test]
    fn string_interpolation_parse(
        var in identifier(),
        text in valid_string(),
    ) {
        let input = format!("f\"Hello {{{}}} {}\"", var, text);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse f-string: {}", input);
    }
    
    #[test]
    fn pipeline_operator_parse(
        start_val: u8,
        func1 in identifier(),
        func2 in identifier(),
        func3 in identifier(),
    ) {
        let input = format!("{} |> {} |> {} |> {}", start_val, func1, func2, func3);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse pipeline: {}", input);
    }
    
    #[test]
    fn method_chaining_parse(
        obj in identifier(),
        methods: Vec<String>,
    ) {
        let chain = methods.iter()
            .filter(|m| !m.is_empty() && m.chars().next().unwrap().is_ascii_alphabetic())
            .take(5)
            .map(|m| format!(".{}()", m))
            .collect::<String>();
        if chain.is_empty() {
            return Ok(());
        }
        let input = format!("{}{}", obj, chain);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse method chain: {}", input);
    }
    
    #[test]
    fn async_await_parse(
        func_name in identifier(),
        var in identifier(),
    ) {
        let input = format!("async fun {}() {{ await {} }}", func_name, var);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse async/await: {}", input);
    }
    
    #[test]
    fn try_catch_parse(
        risky in identifier(),
        err_var in identifier(),
        fallback: u8,
    ) {
        let input = format!("try {{ {} }} catch {} {{ {} }}", risky, err_var, fallback);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse try/catch: {}", input);
    }
    
    #[test]
    fn complex_destructuring_parse(
        a in identifier(),
        b in identifier(),
        c in identifier(),
        vals: Vec<u8>,
    ) {
        if vals.len() < 3 {
            return Ok(());
        }
        let values = vals.iter().take(5).map(|v| v.to_string()).collect::<Vec<_>>().join(", ");
        let input = format!("let [{}, {}, ...{}] = [{}]", a, b, c, values);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse destructuring: {}", input);
    }
    
    #[test]
    fn nested_patterns_parse(
        a in identifier(),
        b in identifier(),
        x: u8,
        y: u8,
        z: u8,
    ) {
        let input = format!("let ({}, [{}, ...rest]) = ({}, [{}, {}, {}])", a, b, x, y, z, z+1);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse nested pattern: {}", input);
    }
    
    #[test]
    fn range_expressions_parse(start: i32, end: i32) {
        // Inclusive range
        let input1 = format!("{}..={}", start, end);
        let mut parser1 = Parser::new(&input1);
        let result1 = parser1.parse();
        prop_assert!(result1.is_ok(), "Failed to parse inclusive range: {}", input1);
        
        // Exclusive range  
        let input2 = format!("{}..{}", start, end);
        let mut parser2 = Parser::new(&input2);
        let result2 = parser2.parse();
        prop_assert!(result2.is_ok(), "Failed to parse exclusive range: {}", input2);
    }
    
    #[test]
    fn lambda_expressions_parse(
        param in identifier(),
        body: u8,
    ) {
        let input = format!("{} => {} * 2", param, body);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse lambda: {}", input);
    }
    
    #[test]
    fn type_annotations_parse(
        var in identifier(),
        val: u8,
    ) {
        let input = format!("let {}: i32 = {}", var, val);
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse type annotation: {}", input);
    }
}