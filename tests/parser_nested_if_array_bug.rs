// PARSER-XXX: Nested if-else with array returns causes "Expected RightBrace" error
// RED phase: All these tests should FAIL before the fix

use ruchy::frontend::parser::Parser;

#[test]
fn test_parser_xxx_nested_if_array_simple() {
    // Minimal reproduction: nested if-else returning arrays with strings
    let source = r#"
fun foo(x) {
    if x == 1 {
        [200, "ok"]
    } else {
        [404, "error"]
    }
}

fun bar() {
    let result = foo(1)
    result[0]
}
"#;

    let mut parser = Parser::new(source);
    let result = parser.parse();
    assert!(result.is_ok(), "Parser should accept nested if-else with arrays");
}

#[test]
fn test_parser_xxx_nested_if_three_branches() {
    // Three-branch nested if-else (the BENCH-010 pattern)
    let source = r#"
fun get_status(path) {
    if path == "/api/users" {
        [200, "users_list"]
    } else {
        if path == "/api/health" {
            [200, "ok"]
        } else {
            [404, "not_found"]
        }
    }
}

fun process(id, path) {
    let result = get_status(path)
    [id, result[0], result[1]]
}
"#;

    let mut parser = Parser::new(source);
    let result = parser.parse();
    assert!(result.is_ok(), "Parser should accept three-branch nested if-else");
}

#[test]
fn test_parser_xxx_without_strings_works() {
    // Control: Same pattern but without strings should work
    let source = r"
fun foo(x) {
    if x == 1 {
        [200, 201]
    } else {
        [404, 405]
    }
}

fun bar() {
    let result = foo(1)
    result[0]
}
";

    let mut parser = Parser::new(source);
    let result = parser.parse();
    assert!(result.is_ok(), "Parser should accept arrays with integers");
}

#[test]
fn test_parser_xxx_single_if_with_strings_works() {
    // Control: Single if-else with strings should work
    let source = r#"
fun foo(x) {
    if x == 1 {
        [200, "ok"]
    } else {
        [404, "error"]
    }
}
"#;

    let mut parser = Parser::new(source);
    let result = parser.parse();
    assert!(result.is_ok(), "Parser should accept single if-else with strings");
}

#[test]
fn test_parser_xxx_multiple_functions_with_let() {
    // Multiple functions where second uses let
    let source = r#"
fun first() {
    if true {
        [1, "one"]
    } else {
        if false {
            [2, "two"]
        } else {
            [3, "three"]
        }
    }
}

fun second() {
    let x = first()
    x
}

fun third() {
    let y = second()
    y
}
"#;

    let mut parser = Parser::new(source);
    let result = parser.parse();
    assert!(result.is_ok(), "Parser should handle multiple functions with let");
}

#[test]
fn test_parser_xxx_array_index_access() {
    // Array index access in second function
    let source = r#"
fun get_data() {
    if true {
        [100, "test", 42]
    } else {
        [0, "error", -1]
    }
}

fun use_data() {
    let arr = get_data()
    let first = arr[0]
    let second = arr[1]
    let third = arr[2]
    first
}
"#;

    let mut parser = Parser::new(source);
    let result = parser.parse();
    assert!(result.is_ok(), "Parser should handle array index access");
}

#[test]
fn test_parser_xxx_deeply_nested() {
    // Deeply nested if-else (4 levels)
    let source = r#"
fun deep(x) {
    if x == 1 {
        [1, "one"]
    } else {
        if x == 2 {
            [2, "two"]
        } else {
            if x == 3 {
                [3, "three"]
            } else {
                [4, "four"]
            }
        }
    }
}

fun caller() {
    let result = deep(2)
    result
}
"#;

    let mut parser = Parser::new(source);
    let result = parser.parse();
    assert!(result.is_ok(), "Parser should handle deeply nested if-else");
}

#[test]
fn test_parser_xxx_with_trailing_code() {
    // Ensure trailing code after functions doesn't break
    let source = r#"
fun foo(path) {
    if path == "/api/users" {
        [200, "users"]
    } else {
        if path == "/api/health" {
            [200, "ok"]
        } else {
            [404, "not_found"]
        }
    }
}

fun bar(id, path) {
    let result = foo(path)
    [id, result[0], result[1]]
}

println(bar(1, "/api/users"))
"#;

    let mut parser = Parser::new(source);
    let result = parser.parse();
    assert!(result.is_ok(), "Parser should handle trailing code after functions");
}
