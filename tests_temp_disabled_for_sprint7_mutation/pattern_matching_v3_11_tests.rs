//! TDD Tests for Pattern Matching Completeness
//! Sprint v3.11.0 - Fix all pattern matching edge cases

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::ast::{Expr, ExprKind, Pattern};
use ruchy::frontend::parser::Parser;

#[cfg(test)]
mod range_pattern_tests {
    use super::*;

    #[test]
    fn test_parse_inclusive_range_pattern() {
        let input = r#"
        match x {
            1..=5 => "in range",
            _ => "out of range"
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        // Currently fails with "Unexpected token in pattern: DotDot"
        assert!(result.is_ok(), "Should parse inclusive range pattern");
    }

    #[test]
    fn test_parse_exclusive_range_pattern() {
        let input = r#"
        match x {
            1..10 => "in range",
            _ => "out of range"
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse exclusive range pattern");
    }

    #[test]
    fn test_transpile_range_pattern() {
        let input = r#"
        match x {
            0..=9 => "digit",
            10..=99 => "two digits",
            _ => "many digits"
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        if result.is_ok() {
            let ast = result.unwrap();
            let transpiler = Transpiler::new();
            let transpiled = transpiler.transpile_to_string(&ast);
            assert!(transpiled.is_ok());
            let code = transpiled.unwrap();
            assert!(code.contains("0..=9") || code.contains("0...9"));
        }
    }

    #[test]
    fn test_char_range_pattern() {
        let input = r#"
        match ch {
            'a'..='z' => "lowercase",
            'A'..='Z' => "uppercase",
            _ => "other"
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse char range patterns");
    }
}

#[cfg(test)]
mod list_destructuring_tests {
    use super::*;

    #[test]
    fn test_parse_list_rest_pattern() {
        let input = r#"
        match list {
            [first, ..rest] => first,
            [] => 0
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse list with rest pattern");
    }

    #[test]
    fn test_parse_list_middle_rest() {
        let input = r#"
        match list {
            [first, ..middle, last] => first + last,
            _ => 0
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse list with middle rest");
    }

    #[test]
    fn test_transpile_list_rest_pattern() {
        let input = r#"
        match list {
            [head, ..tail] => process(head, tail),
            [] => default()
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        if result.is_ok() {
            let ast = result.unwrap();
            let transpiler = Transpiler::new();
            let transpiled = transpiler.transpile_to_string(&ast);
            assert!(transpiled.is_ok());
        }
    }

    #[test]
    fn test_nested_list_patterns() {
        let input = r#"
        match nested {
            [[a, b], [c, d]] => a + b + c + d,
            _ => 0
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse nested list patterns");
    }
}

#[cfg(test)]
mod pattern_guard_tests {
    use super::*;

    #[test]
    fn test_parse_simple_guard() {
        let input = r#"
        match x {
            n if n > 0 => "positive",
            n if n < 0 => "negative",
            _ => "zero"
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse pattern guards");
    }

    #[test]
    fn test_complex_guard_expression() {
        let input = r#"
        match point {
            (x, y) if x * x + y * y <= 100 => "inside circle",
            _ => "outside"
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse complex guard expressions");
    }

    #[test]
    fn test_guard_with_method_call() {
        let input = r#"
        match s {
            text if text.starts_with("http") => "url",
            text if text.contains("@") => "email",
            _ => "other"
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse guards with method calls");
    }

    #[test]
    fn test_transpile_pattern_guards() {
        let input = r#"
        match value {
            Some(x) if x > 10 => large(x),
            Some(x) => small(x),
            None => nothing()
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());

        let ast = result.unwrap();
        let transpiler = Transpiler::new();
        let transpiled = transpiler.transpile_to_string(&ast);
        assert!(transpiled.is_ok());
        let code = transpiled.unwrap();
        assert!(code.contains("if"));
    }
}

#[cfg(test)]
mod or_pattern_tests {
    use super::*;

    #[test]
    fn test_parse_simple_or_pattern() {
        let input = r#"
        match x {
            1 | 2 | 3 => "small",
            _ => "large"
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse or patterns");
    }

    #[test]
    fn test_parse_complex_or_pattern() {
        let input = r#"
        match value {
            Ok(1) | Ok(2) | Ok(3) => "small success",
            Err("timeout") | Err("cancelled") => "expected error",
            _ => "other"
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse complex or patterns");
    }

    #[test]
    fn test_transpile_or_pattern() {
        let input = r#"
        match ch {
            'a' | 'e' | 'i' | 'o' | 'u' => "vowel",
            _ => "consonant"
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());

        let ast = result.unwrap();
        let transpiler = Transpiler::new();
        let transpiled = transpiler.transpile_to_string(&ast);
        assert!(transpiled.is_ok());
        let code = transpiled.unwrap();
        assert!(code.contains("|"));
    }
}

#[cfg(test)]
mod at_binding_tests {
    use super::*;

    #[test]
    fn test_parse_at_binding() {
        let input = r#"
        match value {
            x @ 1..=5 => use_small(x),
            x @ _ => use_any(x)
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse @ bindings");
    }

    #[test]
    fn test_at_binding_with_struct() {
        let input = r#"
        match point {
            p @ Point { x: 0, y: 0 } => origin(p),
            p @ Point { x, y } if x == y => diagonal(p),
            _ => other()
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse @ bindings with structs");
    }

    #[test]
    fn test_transpile_at_binding() {
        let input = r#"
        match num {
            n @ 1..=10 => small(n),
            n => large(n)
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        if result.is_ok() {
            let ast = result.unwrap();
            let transpiler = Transpiler::new();
            let transpiled = transpiler.transpile_to_string(&ast);
            assert!(transpiled.is_ok());
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complex_pattern_combination() {
        let input = r#"
        match data {
            [x @ 1..=5, y, ..rest] if y > x => "pattern1",
            [a | b, c] => "pattern2",
            [] => "empty",
            _ => "other"
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        // This combines multiple advanced patterns
        assert!(result.is_ok(), "Should parse complex pattern combinations");
    }

    #[test]
    fn test_exhaustive_enum_matching() {
        let input = r#"
        match result {
            Ok(x @ 0..=100) => valid(x),
            Ok(x) if x > 100 => too_large(x),
            Err(e @ ("timeout" | "cancelled")) => expected_error(e),
            Err(other) => unexpected_error(other)
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse exhaustive enum patterns");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_range_pattern_never_panics(start in 0i32..100, end in 0i32..100) {
            let input = if start <= end {
                format!("match x {{ {}..={} => \"yes\", _ => \"no\" }}", start, end)
            } else {
                format!("match x {{ {}..={} => \"yes\", _ => \"no\" }}", end, start)
            };

            let mut parser = Parser::new(&input);
            let _ = parser.parse(); // Should not panic
        }

        #[test]
        fn test_or_pattern_never_panics(values: Vec<i32>) {
            if values.is_empty() {
                return Ok(());
            }

            let pattern = values.iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" | ");

            let input = format!("match x {{ {} => \"yes\", _ => \"no\" }}", pattern);
            let mut parser = Parser::new(&input);
            let _ = parser.parse(); // Should not panic
        }

        #[test]
        fn test_guard_pattern_never_panics(var in "[a-z]+", op in "(<|>|<=|>=|==|!=)", val in 0i32..100) {
            let input = format!("match x {{ {} if {} {} {} => \"yes\", _ => \"no\" }}",
                               var, var, op, val);
            let mut parser = Parser::new(&input);
            let _ = parser.parse(); // Should not panic
        }
    }
}
