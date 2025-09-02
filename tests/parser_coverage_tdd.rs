// TDD Tests for 80% parser coverage goal
use ruchy::Parser;

// Error handling tests
#[test]
fn test_parser_empty_input() {
    let mut parser = Parser::new("");
    let result = parser.parse();
    assert!(result.is_err(), "Empty input should fail");
}

#[test]
fn test_parser_invalid_tokens() {
    let mut parser = Parser::new("@#$%^");
    let result = parser.parse();
    assert!(result.is_err(), "Invalid tokens should fail");
}

#[test]
fn test_parser_incomplete_expressions() {
    let test_cases = vec![
        "1 +",           // Missing operand
        "if true",       // Missing then clause
        "let x =",       // Missing value
        "[1, 2,",        // Incomplete list
        "{ a: 1",        // Incomplete object
        "fun foo(",      // Incomplete function
        "match x",       // Incomplete match
    ];
    
    for case in test_cases {
        let mut parser = Parser::new(case);
        let result = parser.parse();
        assert!(result.is_err(), "Case '{}' should fail parsing", case);
    }
}

// Edge case parsing tests
#[test]
fn test_deeply_nested_expressions() {
    let deeply_nested = "(((((1 + 2) * 3) - 4) / 5) % 6)";
    let mut parser = Parser::new(deeply_nested);
    let result = parser.parse();
    assert!(result.is_ok(), "Deep nesting should parse: {:?}", result.err());
}

#[test]
fn test_complex_list_literal() {
    let code = r#"[1, "hello", [2, 3], { name: "test" }, true, 3.14]"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Complex list should parse: {:?}", result.err());
}

#[test]
fn test_all_binary_operators() {
    let operators = vec![
        "1 + 2", "1 - 2", "1 * 2", "1 / 2", "1 % 2", "1 ** 2",
        "1 == 2", "1 != 2", "1 < 2", "1 <= 2", "1 > 2", "1 >= 2",
        "true && false", "true || false",
        "1 & 2", "1 | 2", "1 ^ 2", "1 << 2", "1 >> 2",
        "1 ?? 2",
    ];
    
    for op in operators {
        let mut parser = Parser::new(op);
        let result = parser.parse();
        assert!(result.is_ok(), "Operator '{}' should parse: {:?}", op, result.err());
    }
}

#[test]
fn test_all_unary_operators() {
    let operators = vec!["-1", "+1", "!true", "~1"];
    
    for op in operators {
        let mut parser = Parser::new(op);
        let result = parser.parse();
        assert!(result.is_ok(), "Unary '{}' should parse: {:?}", op, result.err());
    }
}

#[test]
fn test_string_interpolation_variants() {
    let interpolations = vec![
        r#"f"Hello {name}""#,
        r#"f"Value: {x + 1}""#,
        r#"f"Complex: {foo.bar(x)}""#,
    ];
    
    for interp in interpolations {
        let mut parser = Parser::new(interp);
        let result = parser.parse();
        assert!(result.is_ok(), "Interpolation '{}' should parse: {:?}", interp, result.err());
    }
}

#[test]
fn test_range_expressions() {
    let ranges = vec!["1..10", "1..=10", "..10", "1..", ".."];
    
    for range in ranges {
        let mut parser = Parser::new(range);
        let result = parser.parse();
        assert!(result.is_ok(), "Range '{}' should parse: {:?}", range, result.err());
    }
}

#[test]
fn test_pattern_matching_variants() {
    let patterns = vec![
        "match x { 1 => true, _ => false }",
        "match x { Some(y) => y, None => 0 }",
        "match x { [a, b] => a + b, _ => 0 }",
        "match x { Person { name, age } => age, _ => 0 }",
    ];
    
    for pattern in patterns {
        let mut parser = Parser::new(pattern);
        let result = parser.parse();
        assert!(result.is_ok(), "Pattern '{}' should parse: {:?}", pattern, result.err());
    }
}

#[test]
fn test_control_flow_variants() {
    let control_flows = vec![
        "if x > 0 { 1 } else { 0 }",
        "if x > 0 { 1 } else if x < 0 { -1 } else { 0 }",
        "while x < 10 { x += 1 }",
        "for i in 0..10 { print(i) }",
        "loop { break }",
    ];
    
    for cf in control_flows {
        let mut parser = Parser::new(cf);
        let result = parser.parse();
        assert!(result.is_ok(), "Control flow '{}' should parse: {:?}", cf, result.err());
    }
}

#[test]
fn test_function_definitions() {
    let functions = vec![
        "fun add(x, y) { x + y }",
        "fun greet(name: str) -> str { f\"Hello {name}\" }",
        "fun factorial(n: i32) -> i32 { if n <= 1 { 1 } else { n * factorial(n - 1) } }",
        "pub fun public_func() {}",
    ];
    
    for func in functions {
        let mut parser = Parser::new(func);
        let result = parser.parse();
        assert!(result.is_ok(), "Function '{}' should parse: {:?}", func, result.err());
    }
}

#[test]
fn test_data_structure_definitions() {
    let structures = vec![
        "struct Point { x: f64, y: f64 }",
        "struct Person<T> { name: str, data: T }",
        "trait Display { fun show(self) -> str }",
        "impl Display for Point { fun show(self) -> str { f\"{self.x}, {self.y}\" } }",
    ];
    
    for structure in structures {
        let mut parser = Parser::new(structure);
        let result = parser.parse();
        assert!(result.is_ok(), "Structure '{}' should parse: {:?}", structure, result.err());
    }
}

#[test]
fn test_import_and_module_syntax() {
    let imports = vec![
        "import std.io",
        "import { println, print } from std.io",
        "use std::collections::HashMap",
        "from math import sin, cos, tan",
    ];
    
    for import in imports {
        let mut parser = Parser::new(import);
        let result = parser.parse();
        assert!(result.is_ok(), "Import '{}' should parse: {:?}", import, result.err());
    }
}

#[test]
fn test_macro_and_special_syntax() {
    let macros = vec![
        "println!(\"Hello\")",
        "vec![1, 2, 3]",
        "df![name => [\"Alice\", \"Bob\"], age => [25, 30]]",
        "assert!(true)",
    ];
    
    for mac in macros {
        let mut parser = Parser::new(mac);
        let result = parser.parse();
        assert!(result.is_ok(), "Macro '{}' should parse: {:?}", mac, result.err());
    }
}

#[test]
fn test_pipeline_operations() {
    let pipelines = vec![
        "1 |> add(2) |> multiply(3)",
        "[1, 2, 3] |> map(|x| x * 2) |> filter(|x| x > 2)",
        "\"hello\" |> upper() |> split(\"\")",
    ];
    
    for pipe in pipelines {
        let mut parser = Parser::new(pipe);
        let result = parser.parse();
        assert!(result.is_ok(), "Pipeline '{}' should parse: {:?}", pipe, result.err());
    }
}

#[test]
fn test_async_await_syntax() {
    let async_code = vec![
        "async fun fetch_data() { await http.get(\"url\") }",
        "async { await task1(); await task2() }",
    ];
    
    for code in async_code {
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Async '{}' should parse: {:?}", code, result.err());
    }
}

#[test]
fn test_try_catch_syntax() {
    let error_handling = vec![
        "try { risky_operation() } catch(e) { handle_error(e) }",
        "throw new Error(\"Something went wrong\")",
    ];
    
    for eh in error_handling {
        let mut parser = Parser::new(eh);
        let result = parser.parse();
        assert!(result.is_ok(), "Error handling '{}' should parse: {:?}", eh, result.err());
    }
}

#[test]
fn test_edge_case_literals() {
    let literals = vec![
        "0x1234",           // Hex
        "0b1010",           // Binary
        "0o755",            // Octal
        "1e10",             // Scientific notation
        "1.5e-10",          // Small scientific
        "r\"raw string\"",  // Raw string
        "'c'",              // Character
        "null",             // Null
    ];
    
    for lit in literals {
        let mut parser = Parser::new(lit);
        let result = parser.parse();
        assert!(result.is_ok(), "Literal '{}' should parse: {:?}", lit, result.err());
    }
}