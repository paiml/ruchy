//! Additional transpiler tests for edge cases and coverage
#![allow(clippy::unwrap_used)]

use ruchy::compile;

#[test]
fn test_transpile_nested_blocks() {
    let code = r"
        {
            let x = 1
            {
                let y = 2
                x + y
            }
        }
    ";
    let result = compile(code).unwrap();
    assert!(result.contains('{'));
    assert!(result.contains("let x"));
    assert!(result.contains("let y"));
}

#[test]
fn test_transpile_if_expressions() {
    let code = "let result = if x > 0 { 1 } else { -1 }";
    let result = compile(code).unwrap();
    assert!(result.contains("if"));
    assert!(result.contains("else"));
}

#[test]
fn test_transpile_match_expressions() {
    let code = r#"
        match x {
            1 => "one",
            2 => "two",
            _ => "other"
        }
    "#;
    let result = compile(code).unwrap();
    assert!(result.contains("match"));
    assert!(result.contains("=>"));
}

#[test]
fn test_transpile_list_operations() {
    let code = "[1, 2, 3, 4, 5]";
    let result = compile(code).unwrap();
    assert!(result.contains('[') && result.contains(']'));
}

#[test]
fn test_transpile_binary_operations() {
    let cases = vec![
        ("x + y", "+"),
        ("x - y", "-"),
        ("x * y", "*"),
        ("x / y", "/"),
        ("x % y", "%"),
        ("x ** y", "pow"),
        ("x && y", "&&"),
        ("x || y", "||"),
        ("x == y", "=="),
        ("x != y", "!="),
        ("x < y", "<"),
        ("x > y", ">"),
        ("x <= y", "<="),
        ("x >= y", ">="),
        ("x & y", "&"),
        ("x | y", "|"),
        ("x ^ y", "^"),
        ("x << y", "<<"),
        ("x >> y", ">>"),
    ];

    for (code, expected) in cases {
        let result = compile(code).unwrap();
        assert!(result.contains(expected), "Failed for: {code}");
    }
}

#[test]
fn test_transpile_unary_operations() {
    let cases = vec![("-x", "-"), ("!x", "!"), ("~x", "!")];

    for (code, expected) in cases {
        let result = compile(code).unwrap();
        assert!(result.contains(expected), "Failed for: {code}");
    }
}

#[test]
fn test_transpile_for_loops() {
    let code = "for i in 0..10 { i * 2 }";
    let result = compile(code).unwrap();
    assert!(result.contains("for"));
    assert!(result.contains("in"));
    assert!(result.contains(".."));
}

#[test]
fn test_transpile_while_loops() {
    let code = "while x > 0 { x - 1 }";
    let result = compile(code).unwrap();
    assert!(result.contains("while"));
}

#[test]
fn test_transpile_struct_definitions() {
    let code = r"
        struct Person {
            name: String,
            age: i32
        }
    ";
    let result = compile(code).unwrap();
    assert!(result.contains("struct Person"));
    assert!(result.contains("name"));
    assert!(result.contains("age"));
}

#[test]
fn test_transpile_impl_blocks() {
    // Impl blocks with methods
    let code = r"
        impl Person {
            fun new() {
                42
            }
        }
    ";
    let result = compile(code).unwrap();
    assert!(result.contains("impl Person"));
    assert!(result.contains("fn new"));
}

#[test]
fn test_transpile_trait_definitions() {
    let code = "trait Display { }";
    let result = compile(code).unwrap();
    assert!(result.contains("trait Display"));
}

#[test]
fn test_transpile_string_literals() {
    let code = r#""Hello, World!""#;
    let result = compile(code).unwrap();
    assert!(result.contains("Hello, World!"));
}

#[test]
fn test_transpile_numeric_literals() {
    let cases = vec![("42", "42"), ("3.14", "3.14")];

    for (code, expected) in cases {
        let result = compile(code).unwrap();
        assert!(result.contains(expected), "Failed for: {code}");
    }
}

#[test]
fn test_transpile_bool_literals() {
    assert!(compile("true").unwrap().contains("true"));
    assert!(compile("false").unwrap().contains("false"));
}

#[test]
fn test_transpile_list_comprehensions() {
    let code = "[x * 2 for x in [1, 2, 3]]";
    let result = compile(code).unwrap();
    assert!(result.contains("map"));
    assert!(result.contains("collect"));
}

#[test]
fn test_transpile_list_comprehension_with_filter() {
    let code = "[x for x in [1, 2, 3, 4, 5] if x > 2]";
    let result = compile(code).unwrap();
    assert!(result.contains("filter"));
    assert!(result.contains("collect"));
}

#[test]
fn test_transpile_pipeline() {
    let code = "x |> f |> g";
    let result = compile(code).unwrap();
    assert!(result.contains('g'));
    assert!(result.contains('f'));
}

#[test]
fn test_transpile_range() {
    let code = "1..10";
    let result = compile(code).unwrap();
    assert!(result.contains(".."));

    let code = "1..=10";
    let result = compile(code).unwrap();
    assert!(result.contains("..="));
}

#[test]
fn test_transpile_assignments() {
    let code = "x = 5";
    let result = compile(code).unwrap();
    assert!(result.contains('='));

    let code = "x += 5";
    let result = compile(code).unwrap();
    assert!(result.contains("+="));
}

#[test]
fn test_transpile_function_calls() {
    let code = "print(\"hello\")";
    let result = compile(code).unwrap();
    assert!(result.contains("print"));
    assert!(result.contains("hello"));
}

#[test]
fn test_transpile_method_calls() {
    let code = "x.foo()";
    let result = compile(code).unwrap();
    assert!(result.contains('.'));
    assert!(result.contains("foo"));
}

#[test]
fn test_transpile_imports() {
    let code = "import std::io";
    let result = compile(code).unwrap();
    assert!(result.contains("use"));
    assert!(result.contains("std"));
    assert!(result.contains("io"));
}

#[test]
fn test_transpile_exports() {
    let code = "export foo";
    let result = compile(code).unwrap();
    assert!(result.contains("pub"));
}

#[test]
fn test_transpile_modules() {
    let code = "module Math { }";
    let result = compile(code).unwrap();
    assert!(result.contains("mod Math"));
}

#[test]
fn test_transpile_spawn() {
    let code = "spawn Counter::new()";
    let result = compile(code).unwrap();
    assert!(result.contains("Counter"));
    assert!(result.contains("new"));
}

#[test]
fn test_transpile_try_operator() {
    let code = "f()?";
    let result = compile(code).unwrap();
    assert!(result.contains('?'));
}

#[test]
fn test_transpile_post_increment() {
    let code = "x++";
    let result = compile(code).unwrap();
    assert!(result.contains('x'));
    assert!(result.contains("+="));
    assert!(result.contains('1'));
}

#[test]
fn test_transpile_pre_increment() {
    let code = "++x";
    let result = compile(code).unwrap();
    assert!(result.contains('x'));
    assert!(result.contains("+="));
    assert!(result.contains('1'));
}

#[test]
fn test_transpile_dataframe_operations() {
    let code = "df![x => [1, 2, 3]]";
    let result = compile(code).unwrap();
    assert!(result.contains("DataFrame") || result.contains("df"));

    let code = "df![x => [1, 2, 3]].head(5)";
    let result = compile(code).unwrap();
    assert!(result.contains("head"));
}

#[test]
fn test_transpile_actor_definitions() {
    let code = r"
        actor Counter {
            state {
                count: i32
            }
            
            receive Inc() {
                1
            }
        }
    ";
    let result = compile(code).unwrap();
    assert!(result.contains("struct Counter"));
    assert!(result.contains("enum CounterMessage"));
}

#[test]
fn test_transpile_actor_send() {
    let code = "counter ! Inc()";
    let result = compile(code).unwrap();
    assert!(result.contains("send"));
    assert!(result.contains("Inc"));
}

#[test]
fn test_transpile_actor_ask() {
    let code = "counter ? GetCount()";
    let result = compile(code).unwrap();
    assert!(result.contains("ask"));
    assert!(result.contains("GetCount"));
}

#[test]
fn test_transpile_complex_expression() {
    let code = "(x + y) * z / (a - b)";
    let result = compile(code).unwrap();
    assert!(result.contains('+'));
    assert!(result.contains('*'));
    assert!(result.contains('/'));
    assert!(result.contains('-'));
}

#[test]
fn test_transpile_parenthesized() {
    let code = "(42)";
    let result = compile(code).unwrap();
    assert!(result.contains("42"));
}

#[test]
fn test_transpile_unit() {
    let code = "()";
    let result = compile(code).unwrap();
    assert!(result.contains("()"));
}
