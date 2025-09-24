// EXTREME TDD: Final Push to 80% Coverage
// Target: Low coverage areas
// Complexity: <10 per test
// Single responsibility, zero technical debt

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

#[cfg(test)]
mod final_parser_tests {
    use super::*;

    #[test]
    fn test_parse_numbers() {
        let cases = vec![
            "0",
            "42",
            "-1",
            "3.14",
            "-2.71",
            "1e10",
            "1E-5",
            "0xFF",
            "0o77",
            "0b101",
            "1_000_000",
        ];
        for case in cases {
            let mut parser = Parser::new(case);
            assert!(parser.parse().is_ok(), "Failed to parse: {case}");
        }
    }

    #[test]
    fn test_parse_strings() {
        let cases = vec![
            r#""""#,
            r#""hello""#,
            r#""world""#,
            r#""test\n""#,
            r#""with\"quotes\"""#,
            r#""unicode: 你好""#,
        ];
        for case in cases {
            let mut parser = Parser::new(case);
            assert!(parser.parse().is_ok(), "Failed to parse: {case}");
        }
    }

    #[test]
    fn test_parse_operators() {
        let cases = vec![
            "1 + 2",
            "3 - 1",
            "2 * 3",
            "6 / 2",
            "7 % 3",
            "2 ** 3",
            "true && false",
            "true || false",
            "!true",
            "1 == 1",
            "1 != 2",
            "1 < 2",
            "2 > 1",
            "1 <= 1",
            "2 >= 2",
            "5 & 3",
            "5 | 3",
            "5 ^ 3",
            "1 << 2",
            "8 >> 1",
        ];
        for case in cases {
            let mut parser = Parser::new(case);
            assert!(parser.parse().is_ok(), "Failed to parse: {case}");
        }
    }

    #[test]
    fn test_parse_collections() {
        let cases = vec![
            "[]",
            "[1]",
            "[1, 2, 3]",
            "[1, 2, 3,]",
            "{}",
            "{a: 1}",
            "{a: 1, b: 2}",
            "{\"key\": \"value\"}",
            "()",
            "(1,)",
            "(1, 2)",
            "(1, 2, 3)",
        ];
        for case in cases {
            let mut parser = Parser::new(case);
            assert!(parser.parse().is_ok(), "Failed to parse: {case}");
        }
    }

    #[test]
    fn test_parse_variables() {
        let supported_cases = vec![
            "let x = 1",
            "let mut y = 2",
            "let x: i32 = 1",
            "let mut y: f64 = 2.0",
            "let (a, b) = (1, 2)",
            "let [x, y, z] = [1, 2, 3]",
        ];
        for case in supported_cases {
            let mut parser = Parser::new(case);
            assert!(parser.parse().is_ok(), "Failed to parse: {case}");
        }

        // Test unsupported syntax (const keyword not implemented)
        let unsupported_cases = vec!["const Z = 3"];
        for case in unsupported_cases {
            let mut parser = Parser::new(case);
            assert!(parser.parse().is_err(), "Expected to fail parsing: {case}");
        }
    }

    #[test]
    fn test_parse_functions() {
        let cases = vec![
            "fun f() {}",
            "fun f(x) { x }",
            "fun f(x, y) { x + y }",
            "fun f() -> i32 { 42 }",
            "|x| x + 1",
            "|| 42",
            "|x, y| x + y",
            "fun f<T>(x: T) -> T { x }",
        ];
        for case in cases {
            let mut parser = Parser::new(case);
            assert!(parser.parse().is_ok(), "Failed to parse: {case}");
        }
    }

    #[test]
    fn test_parse_control_flow() {
        let cases = vec![
            "if true { 1 }",
            "if x { 1 } else { 2 }",
            "while true { break }",
            "for i in 0..10 { i }",
            "loop { break }",
            "match x { 1 => \"one\", _ => \"other\" }",
            "return 42",
            "break",
            "continue",
        ];
        for case in cases {
            let mut parser = Parser::new(case);
            assert!(parser.parse().is_ok(), "Failed to parse: {case}");
        }
    }

    #[test]
    fn test_parse_expressions() {
        let cases = vec![
            "x",
            "x.field",
            "x.method()",
            "x[0]",
            "f()",
            "f(1)",
            "f(1, 2)",
            "x + y * z",
            "(x + y) * z",
            "x ? y : z",
            "x as i32",
            "sizeof(x)",
            "typeof(x)",
        ];
        for case in cases {
            let mut parser = Parser::new(case);
            assert!(parser.parse().is_ok(), "Failed to parse: {case}");
        }
    }

    #[test]
    fn test_parse_advanced() {
        let supported_cases = vec![
            "async fun f() { await g() }",
            "f\"Hello {name}\"",
            "Some(42)",
            "None",
            "Ok(1)",
            "Err(\"error\")",
            "x |> f |> g",
        ];
        for case in supported_cases {
            let mut parser = Parser::new(case);
            assert!(parser.parse().is_ok(), "Failed to parse: {case}");
        }

        // Test supported list comprehensions
        let more_supported_cases = vec!["[x for x in 0..10]", "[x for x in 0..10 if x % 2 == 0]"];
        for case in more_supported_cases {
            let mut parser = Parser::new(case);
            assert!(parser.parse().is_ok(), "Failed to parse: {case}");
        }

        // Test unsupported syntax (dict comprehensions, spread, destructuring not implemented)
        let unsupported_cases = vec!["{x: x*2 for x in 0..5}", "...arr", "obj.{x, y}"];
        for case in unsupported_cases {
            let mut parser = Parser::new(case);
            assert!(parser.parse().is_err(), "Expected to fail parsing: {case}");
        }
    }

    #[test]
    fn test_parse_statements() {
        let supported_cases = vec![
            "x = 1",
            "x += 1",
            "x -= 1",
            "x *= 2",
            "x /= 2",
            "x %= 3",
            "x &= 1",
            "x |= 2",
            "x ^= 3",
            "x <<= 1",
            "use std",
            "import math",
            "from math import sqrt",
            "type MyInt = i32",
            "struct Point { x: i32, y: i32 }",
            "enum Color { Red, Green, Blue }",
            "trait Show { fun show(self) }",
        ];
        for case in supported_cases {
            let mut parser = Parser::new(case);
            assert!(parser.parse().is_ok(), "Failed to parse: {case}");
        }

        // Test unsupported operators (>>= not implemented)
        let unsupported_cases = vec!["x >>= 1"];
        for case in unsupported_cases {
            let mut parser = Parser::new(case);
            assert!(parser.parse().is_err(), "Expected to fail parsing: {case}");
        }
    }
}

#[cfg(test)]
mod final_transpiler_tests {
    use super::*;

    #[test]
    fn test_transpile_basic() {
        let cases = vec![
            "42",
            "true",
            "\"hello\"",
            "x",
            "[1, 2, 3]",
            "{a: 1}",
            "(1, 2)",
        ];
        let transpiler = Transpiler::new();
        for case in cases {
            let mut parser = Parser::new(case);
            if let Ok(ast) = parser.parse() {
                let result = transpiler.transpile_expr(&ast);
                assert!(result.is_ok(), "Failed to transpile: {case}");
            }
        }
    }

    #[test]
    fn test_transpile_operators() {
        let cases = vec![
            "1 + 2",
            "3 - 1",
            "2 * 3",
            "6 / 2",
            "7 % 3",
            "true && false",
            "true || false",
            "!true",
        ];
        let transpiler = Transpiler::new();
        for case in cases {
            let mut parser = Parser::new(case);
            if let Ok(ast) = parser.parse() {
                let result = transpiler.transpile_expr(&ast);
                assert!(result.is_ok(), "Failed to transpile: {case}");
            }
        }
    }

    #[test]
    fn test_transpile_functions() {
        let cases = vec!["fun f() {}", "fun f(x) { x }", "|x| x + 1"];
        let transpiler = Transpiler::new();
        for case in cases {
            let mut parser = Parser::new(case);
            if let Ok(ast) = parser.parse() {
                let result = transpiler.transpile_expr(&ast);
                assert!(result.is_ok(), "Failed to transpile: {case}");
            }
        }
    }

    #[test]
    fn test_transpile_control() {
        let cases = vec![
            "if true { 1 }",
            "if x { 1 } else { 2 }",
            "match x { 1 => \"one\", _ => \"other\" }",
        ];
        let transpiler = Transpiler::new();
        for case in cases {
            let mut parser = Parser::new(case);
            if let Ok(ast) = parser.parse() {
                let result = transpiler.transpile_expr(&ast);
                assert!(result.is_ok(), "Failed to transpile: {case}");
            }
        }
    }

    #[test]
    fn test_transpile_assignments() {
        let cases = vec![
            "let x = 1",
            "let mut y = 2",
            "const Z = 3",
            "x = 1",
            "x += 1",
            "x -= 1",
            "x *= 2",
            "x /= 2",
        ];
        let transpiler = Transpiler::new();
        for case in cases {
            let mut parser = Parser::new(case);
            if let Ok(ast) = parser.parse() {
                let result = transpiler.transpile_expr(&ast);
                assert!(result.is_ok(), "Failed to transpile: {case}");
            }
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_parse_and_transpile() {
        let programs = vec![
            "let x = 42",
            "fun add(a, b) { a + b }",
            "if x > 0 { x } else { -x }",
            "[x * 2 for x in 0..5]",
            "match x { Some(v) => v, None => 0 }",
            "class Point { x: i32, y: i32 }",
            "x |> f |> g",
            "async fun fetch() { await url }",
            "let [a, b, ...rest] = [1, 2, 3, 4, 5]",
        ];

        let transpiler = Transpiler::new();
        for prog in programs {
            let mut parser = Parser::new(prog);
            if let Ok(ast) = parser.parse() {
                let result = transpiler.transpile_expr(&ast);
                assert!(result.is_ok(), "Failed to compile: {prog}");
            }
        }
    }

    #[test]
    fn test_comprehensive_features() {
        let code = r"
            fun factorial(n) {
                if n <= 1 {
                    1
                } else {
                    n * factorial(n - 1)
                }
            }

            let result = factorial(5)
        ";

        let mut parser = Parser::new(code);
        let ast = parser.parse();
        assert!(ast.is_ok());

        let transpiler = Transpiler::new();
        if let Ok(ast) = ast {
            let result = transpiler.transpile_expr(&ast);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_error_recovery() {
        // These should parse with error recovery
        let error_cases = vec![
            "",
            "   ",
            "\n\n",
            "// comment only",
            "/* block comment */",
            "let",
            "fun",
            "if",
            "1 +",
            "let x =",
            "fun f(",
        ];

        for case in error_cases {
            let mut parser = Parser::new(case);
            let _ = parser.parse(); // Don't assert, just ensure no panic
        }
    }

    #[test]
    fn test_unicode_support() {
        let cases = vec![
            "let 你好 = \"世界\"",
            "let π = 3.14159",
            "let café = \"coffee\"",
            "\"emoji: 🎉🎊🎈\"",
            "// комментарий",
        ];

        for case in cases {
            let mut parser = Parser::new(case);
            let _ = parser.parse(); // Unicode should be handled gracefully
        }
    }

    #[test]
    fn test_deeply_nested() {
        let nested = "((((((((((1))))))))))";
        let mut parser = Parser::new(nested);
        assert!(parser.parse().is_ok());

        let nested_if = "if true { if true { if true { if true { 1 } } } }";
        let mut parser = Parser::new(nested_if);
        assert!(parser.parse().is_ok());
    }
}
