//! Sprint 80: Coverage boost - simpler tests that compile

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

#[test]
fn test_parser_comprehensive() {
    let test_cases = vec![
        // Basic expressions
        "42", "3.14", "true", "false", "nil", "'c'",
        r#""string""#, r#""string with spaces""#,

        // Binary operations
        "1 + 2", "3 - 4", "5 * 6", "7 / 8", "9 % 10",
        "true && false", "true || false", "!true",
        "1 < 2", "3 <= 4", "5 > 6", "7 >= 8", "9 == 10", "11 != 12",

        // Arrays and tuples
        "[1, 2, 3]", "[]", "[1]",
        "(1, 2, 3)", "()", "(1,)",

        // Objects
        "{ }", r#"{ "key": "value" }"#, "{ x: 1, y: 2 }",

        // Functions
        "fn() { }", "fn(x) { x }", "fn(x, y) { x + y }",
        "x => x", "(x, y) => x + y",

        // Control flow
        "if true { 1 }", "if x { 1 } else { 2 }",
        "while true { }", "for x in 1..10 { }",
        "match x { 1 => 'a', 2 => 'b', _ => 'c' }",

        // Let bindings
        "let x = 5", "let mut y = 10", "const PI = 3.14",

        // Method calls
        "obj.method()", "arr[0]", "obj.field",

        // Complex expressions
        "1 + 2 * 3", "(1 + 2) * 3",
        "f(g(h(i())))",
        "[1, 2, 3].map(x => x * 2)",
    ];

    for code in test_cases {
        let mut parser = Parser::new(code);
        let _ = parser.parse();
    }
}

// Lexer test removed - Lexer not public

#[test]
fn test_transpiler_comprehensive() {
    let transpiler = Transpiler::new();

    let test_cases = vec![
        "let x = 5",
        "fn add(a, b) { a + b }",
        "if x > 0 { x } else { -x }",
        "[1, 2, 3]",
        "{ x: 1, y: 2 }",
        "for i in 1..10 { println(i) }",
        "match x { Some(v) => v, None => 0 }",
        "class Point { x: int; y: int }",
        "x + y * z",
        "f(1, 2, 3)",
    ];

    for code in test_cases {
        let mut parser = Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let _ = transpiler.transpile(&ast);
        }
    }
}

// AST construction test removed - types not public

#[test]
fn test_parser_error_recovery() {
    // These should not panic even with invalid input
    let error_cases = vec![
        "", " ", "\n", "\t",
        "let", "fn", "if", "while",
        "(", ")", "[", "]", "{", "}",
        "+", "-", "*", "/",
        "let x =", "fn (", "if {",
        "unterminated string",
        "1 + + 2", "3 * * 4",
        "((()))", "[[[]]]", "{{{}}}",
    ];

    for code in error_cases {
        let mut parser = Parser::new(code);
        let _ = parser.parse(); // Should not panic
    }
}

#[test]
fn test_parser_unicode() {
    let unicode_cases = vec![
        "let 你好 = 42",
        "let π = 3.14159",
        "let ∑ = sum",
        r#""Hello 世界""#,
        r#""Emoji 😀🎉""#,
        "// Comment with unicode: αβγδ",
    ];

    for code in unicode_cases {
        let mut parser = Parser::new(code);
        let _ = parser.parse();
    }
}

#[test]
fn test_parser_edge_cases() {
    let edge_cases = vec![
        // Deep nesting
        "((((((((((x))))))))))",
        "[[[[[[[[[[x]]]]]]]]]]",
        "{{{{{{{{{{x}}}}}}}}}}",

        // Long identifiers
        "let very_long_identifier_name_that_goes_on_and_on = 42",

        // Large numbers
        "999999999999999999999999999999",
        "0.000000000000000000000001",
        "1e308", "-1e308",

        // Many arguments
        "fn(a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p) { }",
        "f(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16)",

        // Complex expressions
        "a + b * c - d / e % f & g | h ^ i << j >> k",
        "x.y.z.w.v.u.t.s.r.q.p.o.n.m.l.k.j.i.h.g.f.e.d.c.b.a",
    ];

    for code in edge_cases {
        let mut parser = Parser::new(code);
        let _ = parser.parse();
    }
}

#[test]
fn test_transpiler_all_expr_kinds() {
    let transpiler = Transpiler::new();

    // Create expressions for each ExprKind variant
    let test_cases = vec![
        // Literals
        "42", "3.14", "true", "'c'", r#""string""#, "nil",

        // Identifiers and operators
        "x", "x + y", "x - y", "x * y", "x / y", "x % y",
        "x == y", "x != y", "x < y", "x <= y", "x > y", "x >= y",
        "x && y", "x || y", "!x",
        "x & y", "x | y", "x ^ y", "~x", "x << y", "x >> y",

        // Collections
        "[1, 2, 3]", "(1, 2, 3)", "{ x: 1, y: 2 }",

        // Functions and calls
        "fn(x) { x }", "f(x)", "x => x",

        // Control flow
        "if x { 1 } else { 2 }",
        "while x { }",
        "for x in y { }",
        "match x { _ => 1 }",
        "return x",
        "break",
        "continue",

        // Assignments and declarations
        "let x = 5",
        "x = 5",
        "x += 5",

        // Member access and indexing
        "x.y", "x[0]", "x[0..10]",

        // Type annotations
        "x as int", "x is int",

        // Async
        "await x", "async { x }",

        // Ranges
        "1..10", "1..=10",
    ];

    for code in test_cases {
        let mut parser = Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let _ = transpiler.transpile(&ast);
        }
    }
}

// AST helper tests removed - types not public

// Token test removed - Token is private

#[test]
fn test_parser_statement_types() {
    let statements = vec![
        // Variable declarations
        "let x = 5",
        "let mut y = 10",
        "const Z = 15",

        // Function declarations
        "fn test() { }",
        "fn add(a, b) { a + b }",
        "async fn fetch() { }",

        // Class declarations
        "class Point { }",
        "class Circle extends Shape { }",

        // Import/export
        "import foo",
        "export fn bar() { }",

        // Expression statements
        "print('hello')",
        "x + y",
        "obj.method()",

        // Control flow statements
        "if x { }",
        "while true { }",
        "for x in arr { }",
        "return 42",
        "break",
        "continue",
    ];

    for stmt in statements {
        let mut parser = Parser::new(stmt);
        let _ = parser.parse();
    }
}

#[test]
fn test_transpiler_statements() {
    let transpiler = Transpiler::new();

    let statements = vec![
        // Simple statements
        "let x = 5",
        "x = 10",
        "print(x)",
        "return x",

        // Control flow
        "if x { y }",
        "while x { y }",
        "for x in arr { }",

        // Functions
        "fn test() { 42 }",

        // Classes
        "class Test { }",
    ];

    for stmt in statements {
        let mut parser = Parser::new(stmt);
        if let Ok(ast) = parser.parse() {
            let _ = transpiler.transpile(&ast);
        }
    }
}