#![allow(missing_docs)]
//! PARSER-ATTR-001: Rust-style `#[attribute]` parses per 5.0 spec Section 3.
//!
//! The 5.0 Sovereign Platform spec defines a unified decorator grammar:
//!   decorator ::= '@' IDENT ( '(' arg_list ')' )?
//!   attribute ::= '#[' IDENT ( '(' arg_list ')' )? ']'
//!
//! Both forms MUST parse and produce `Attribute` AST nodes. Prior to this
//! fix the parser bail!'d on `#[` with "Attributes are not supported",
//! which broke 5.0 sub-spec attributes (`#[prove(level)]`, `#[probar_test]`,
//! `#[playbook(name)]`, `#[brick_budget(ms)]`, `#[zero_js]`,
//! `#[infra_policy(p)]`) and every ruchy-book test file using `#[test]`.
//!
//! Discovered via Genchi Genbutsu during ruchy-cookbook validation on
//! 5.0.0-beta.1.
//!
//! Ticket: [PARSER-ATTR-001] Unified decorator grammar: accept #[attr].

use ruchy::Parser;

fn parse_ok(src: &str) {
    let mut p = Parser::new(src);
    match p.parse() {
        Ok(_) => {}
        Err(e) => panic!("parse must succeed for {src:?}, got: {e}"),
    }
}

#[test]
fn test_parser_attr_001_bare_attribute() {
    parse_ok("#[test]\nfun foo() { 1 }");
}

#[test]
fn test_parser_attr_001_attribute_with_arg() {
    parse_ok("#[playbook(smoke)]\nfun foo() { 1 }");
}

#[test]
fn test_parser_attr_001_attribute_with_multiple_args() {
    parse_ok("#[prove(silver)]\nfun foo() { 1 }");
}

#[test]
fn test_parser_attr_001_multiple_attributes_stacked() {
    parse_ok("#[test]\n#[probar_test]\nfun foo() { 1 }");
}

#[test]
fn test_parser_attr_001_mixed_decorator_and_attribute() {
    // Per spec, @decorator and #[attr] can be mixed on the same item.
    parse_ok("@verified\n#[prove(silver)]\nfun foo() { 1 }");
}

#[test]
fn test_parser_attr_001_all_pillar_attributes_parse() {
    // From spec Section 3 decorator/attribute map.
    for attr in &[
        "#[prove(silver)]",
        "#[probar_test]",
        "#[playbook(smoke)]",
        "#[brick_budget(100)]",
        "#[zero_js]",
        "#[infra_policy(strict)]",
    ] {
        parse_ok(&format!("{attr}\nfun foo() {{ 1 }}"));
    }
}
