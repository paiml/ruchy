#![allow(missing_docs)]
//! #255 item 2: `ruchy fmt` must keep the `mut` of `let mut`.

use ruchy::quality::formatter::Formatter;
use ruchy::Parser;

fn fmt(code: &str) -> String {
    let ast = Parser::new(code).parse().expect("parse");
    Formatter::new().format(&ast).expect("format")
}

#[test]
fn let_mut_is_preserved_top_level() {
    let out = fmt("let mut counter = 0\ncounter = counter + 1\nprintln(counter)\n");
    assert!(out.contains("let mut counter = 0"), "lost mut: {out}");
}

#[test]
fn let_mut_is_preserved_in_function_body() {
    let out =
        fmt("fun f() {\n    let mut total = 1\n    total = total + 1\n    println(total)\n}\n");
    assert!(out.contains("let mut total = 1"), "lost mut: {out}");
}

#[test]
fn immutable_let_stays_immutable() {
    let out = fmt("let x = 1\nprintln(x)\n");
    assert!(!out.contains("let mut"), "added mut: {out}");
}
