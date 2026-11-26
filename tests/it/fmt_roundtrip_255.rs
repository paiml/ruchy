#![allow(missing_docs)]
//! #255 items 1-3: `ruchy fmt` output must re-parse, keep `mut`, and keep the trailing newline.

use ruchy::quality::formatter::Formatter;
use ruchy::Parser;

fn fmt(code: &str) -> String {
    let ast = Parser::new(code).parse().expect("parse input");
    Formatter::new().format(&ast).expect("format")
}

fn assert_reparses(out: &str) {
    Parser::new(out)
        .parse()
        .unwrap_or_else(|e| panic!("fmt output does not parse: {e:?}\n{out}"));
}

#[test]
fn item1_format_spec_debug_round_trips() {
    let out = fmt("fun main() {\n    let doubled = 2\n    println(f\"Doubled: {doubled:?}\")\n}\n");
    assert!(!out.contains("::?"), "doubled colon in spec: {out}");
    assert!(out.contains("{doubled:?}"), "spec lost: {out}");
    assert_reparses(&out);
}

#[test]
fn item1_format_spec_numeric_round_trips() {
    let out = fmt("let n = 7\nprintln(f\"{n:02}\")\n");
    assert!(!out.contains("::"), "doubled colon in spec: {out}");
    assert!(out.contains("{n:02}"), "spec lost: {out}");
    assert_reparses(&out);
}

#[test]
fn item2_let_mut_round_trips() {
    let out = fmt("let mut counter = 0\ncounter = counter + 1\nprintln(counter)\n");
    assert!(out.contains("let mut counter = 0"), "lost mut: {out}");
    assert_reparses(&out);
}

#[test]
fn item3_trailing_newline_kept() {
    let out = fmt("let mut counter = 0\nprintln(counter)\n");
    assert!(out.ends_with('\n'), "trailing newline dropped: {out:?}");
    assert_reparses(&out);
}

#[test]
fn fmt_is_idempotent_on_item_inputs() {
    let src = "fun main() {\n    let mut c = 0\n    println(f\"{c:?}\")\n}\n";
    let once = fmt(src);
    let twice = fmt(&once);
    assert_eq!(once, twice, "second fmt pass changed output");
}
