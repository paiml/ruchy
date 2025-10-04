// Shared test fixtures cached across tests

use once_cell::sync::Lazy;
use ruchy::frontend::ast::Ast;
use ruchy::frontend::parser::Parser;

/// Common sample programs for testing
pub static SAMPLE_PROGRAMS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        "let x = 42".into(),
        "fn add(a: i32, b: i32) -> i32 { a + b }".into(),
        "match x { Some(y) => y, None => 0 }".into(),
        "if true { 1 } else { 0 }".into(),
        "[1, 2, 3, 4, 5]".into(),
        "for i in 0..10 { println(i) }".into(),
        "struct Point { x: i32, y: i32 }".into(),
        "actor Counter { state { count: i32 } }".into(),
    ]
});

/// Pre-parsed ASTs for testing (avoids re-parsing)
pub static PARSED_ASTS: Lazy<Vec<Ast>> = Lazy::new(|| {
    SAMPLE_PROGRAMS
        .iter()
        .filter_map(|src| {
            let mut parser = Parser::new(src);
            parser.parse().ok()
        })
        .collect()
});

/// Common test expressions
pub static TEST_EXPRESSIONS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        "1 + 2".into(),
        "x * 3".into(),
        "true && false".into(),
        "!true".into(),
        "a == b".into(),
        "x > 0".into(),
        "arr[0]".into(),
        "obj.field".into(),
        "func(arg)".into(),
        "|x| x + 1".into(),
    ]
});

/// Common error cases for testing
pub static ERROR_PROGRAMS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        "let = 42".into(),
        "fn () {}".into(),
        "if { }".into(),
        "match { }".into(),
        "for in { }".into(),
        "1 + + 2".into(),
        "( ( )".into(),
        "[ [ ]".into(),
        "{ { }".into(),
        "unclosed string".into(),
    ]
});
