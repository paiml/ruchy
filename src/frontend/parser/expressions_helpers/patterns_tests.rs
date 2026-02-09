use super::*;


use crate::frontend::parser::Parser;

#[test]
fn test_identifier_pattern() {
    let code = "let x = 42";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Identifier pattern should parse");
}

#[test]
fn test_tuple_pattern() {
    let code = "let (x, y, z) = (1, 2, 3)";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Tuple pattern should parse");
}

#[test]
fn test_list_pattern_with_rest() {
    let code = "let [first, ...rest] = [1, 2, 3, 4]";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "List pattern with rest should parse");
}

#[test]
fn test_struct_pattern() {
    let code = "let Point { x, y } = point";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Struct pattern should parse");
}

#[test]
fn test_some_pattern() {
    let code = "let Some(x) = maybe_value";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Some pattern should parse");
}

#[test]
fn test_ok_pattern() {
    let code = "let Ok(val) = result";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Ok pattern should parse");
}

#[test]
fn test_err_pattern() {
    let code = "let Err(e) = result";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Err pattern should parse");
}

#[test]
fn test_none_pattern() {
    let code = "let None = maybe_value";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "None pattern should parse");
}

#[test]
fn test_wildcard_pattern() {
    let code = "let _ = value";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Wildcard pattern should parse");
}

#[test]
fn test_literal_pattern() {
    let code = "match x { 42 => true, _ => false }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Literal pattern in match should parse");
}

#[test]
fn test_range_pattern() {
    let code = "match x { 1..10 => \"low\", _ => \"high\" }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Range pattern should parse");
}

#[test]
fn test_or_pattern() {
    let code = "match x { Some(1) | Some(2) => true, _ => false }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Or pattern should parse");
}

// PARSER-082: Atom pattern tests
#[test]
fn test_parser_082_atom_pattern_simple() {
    let code = "match x { :ok => true, :error => false }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Atom pattern :ok should parse");
}

#[test]
fn test_parser_082_atom_pattern_with_wildcard() {
    let code = "match status { :ok => handle_ok(), :error => handle_error(), _ => default() }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Atom patterns with wildcard should parse");
}

#[test]
fn test_parser_082_atom_pattern_or() {
    let code = "match x { :ok | :success => true, _ => false }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Or patterns with atoms should parse");
}

#[test]
fn test_parser_082_atom_pattern_in_tuple() {
    let code = "match pair { (:ok, value) => value, (:error, msg) => panic(msg) }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Atom in tuple pattern should parse");
}

// COVERAGE: Additional pattern tests
#[test]
fn test_nested_tuple_pattern() {
    let code = "let ((a, b), c) = ((1, 2), 3)";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Nested tuple pattern should parse");
}

#[test]
fn test_list_pattern_without_rest() {
    let code = "let [a, b, c] = [1, 2, 3]";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "List pattern without rest should parse");
}

#[test]
fn test_tuple_variant_pattern() {
    let code = "match x { Point(a, b) => a + b }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Tuple variant pattern should parse");
}

#[test]
fn test_struct_pattern_with_rest() {
    let code = "let Point { x, .. } = point";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Struct pattern with rest should parse");
}

#[test]
fn test_match_with_guard() {
    let code = "match x { n if n > 0 => true, _ => false }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Match with guard should parse");
}

#[test]
fn test_match_inclusive_range() {
    let code = "match x { 1..=10 => \"in range\", _ => \"out\" }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Inclusive range pattern should parse");
}

#[test]
fn test_if_let_expression() {
    let code = "if let Some(x) = maybe { x } else { 0 }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "If-let expression should parse");
}

#[test]
fn test_string_literal_pattern() {
    let code = r#"match s { "hello" => true, _ => false }"#;
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "String literal pattern should parse");
}

#[test]
fn test_bool_literal_pattern() {
    let code = "match b { true => 1, false => 0 }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Bool literal pattern should parse");
}

#[test]
fn test_float_literal_pattern() {
    let code = "match f { 3.14 => \"pi\", _ => \"other\" }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Float literal pattern should parse");
}

#[test]
fn test_multiple_or_patterns() {
    let code = "match x { 1 | 2 | 3 => true, _ => false }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Multiple or patterns should parse");
}

#[test]
fn test_char_literal_pattern() {
    let code = "match c { 'a' => 1, 'b' => 2, _ => 0 }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Char literal pattern should parse");
}

#[test]
fn test_char_range_pattern() {
    let code = "match c { 'a'..'z' => true, _ => false }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Char range pattern should parse");
}

#[test]
fn test_match_with_wildcard_only() {
    // Match requires at least one arm
    let code = "match x { _ => 0 }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Match with wildcard arm should parse");
}

#[test]
fn test_match_multiple_arms() {
    let code = "match n { 0 => \"zero\", 1 => \"one\", 2 => \"two\", _ => \"many\" }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Match with multiple arms should parse");
}

#[test]
fn test_let_with_type_annotation() {
    let code = "let x: i32 = 42";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Let with type annotation should parse");
}

#[test]
fn test_mutable_pattern() {
    let code = "let mut x = 42";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Mutable let should parse");
}

#[test]
fn test_var_declaration() {
    let code = "var x = 42";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Var declaration should parse");
}

#[test]
fn test_constructor_pattern_no_args() {
    let code = "match x { Unit => 0 }";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Constructor pattern without args should parse"
    );
}

#[test]
fn test_constructor_pattern_with_args() {
    let code = "match x { Pair(a, b) => a + b }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Constructor pattern with args should parse");
}

#[test]
fn test_nested_struct_pattern() {
    let code = "let Line { start: Point { x, y }, end } = line";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Nested struct pattern should parse");
}

#[test]
fn test_struct_field_rename() {
    let code = "let Point { x: new_x, y: new_y } = point";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Struct field rename should parse");
}

#[test]
fn test_complex_match_expression() {
    let code = r#"match result {
        Ok(value) if value > 0 => value * 2,
        Ok(0) => 0,
        Err(e) => -1
    }"#;
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Complex match expression should parse");
}

#[test]
fn test_if_let_with_else_if() {
    let code = "if let Some(x) = a { x } else if let Some(y) = b { y } else { 0 }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Chained if-let should parse");
}

#[test]
fn test_let_else_clause() {
    let code = "let Some(x) = maybe else { return 0 }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Let-else should parse");
}

#[test]
fn test_pattern_with_default_value() {
    let code = "fun foo(x = 10) { x }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Pattern with default value should parse");
}

#[test]
fn test_rest_pattern_only() {
    let code = "let [...all] = [1, 2, 3]";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Rest pattern only should parse");
}

#[test]
fn test_large_integer_pattern() {
    // Use positive integer - negative literals in patterns may not be supported
    let code = "match x { 100 => \"hundred\", _ => \"other\" }";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Large integer pattern should parse");
}

// ============================================================
// Additional comprehensive tests for EXTREME TDD coverage
// ============================================================

use crate::frontend::ast::{Expr, ExprKind};
use crate::frontend::parser::Result;

fn parse(code: &str) -> Result<Expr> {
    Parser::new(code).parse()
}

fn get_block_exprs(expr: &Expr) -> Option<&Vec<Expr>> {
    match &expr.kind {
        ExprKind::Block(exprs) => Some(exprs),
        _ => None,
    }
}

// ============================================================
// Tuple pattern comprehensive tests
// ============================================================

#[test]
fn test_tuple_pattern_empty() {
    let result = parse("let () = ()");
    assert!(result.is_ok(), "Empty tuple pattern should parse");
}

#[test]
fn test_tuple_pattern_single() {
    let result = parse("let (x,) = (1,)");
    assert!(result.is_ok(), "Single element tuple should parse");
}

#[test]
fn test_tuple_pattern_four_elements() {
    let result = parse("let (a, b, c, d) = (1, 2, 3, 4)");
    assert!(result.is_ok(), "Four element tuple should parse");
}

#[test]
fn test_tuple_pattern_with_wildcards() {
    let result = parse("let (x, _, z) = (1, 2, 3)");
    assert!(result.is_ok(), "Tuple with wildcards should parse");
}

#[test]
fn test_tuple_pattern_nested_three_levels() {
    let result = parse("let (((a, b), c), d) = (((1, 2), 3), 4)");
    assert!(result.is_ok(), "Deeply nested tuple should parse");
}

#[test]
fn test_tuple_pattern_with_mut_element() {
    let result = parse("let (mut x, y) = (1, 2)");
    assert!(result.is_ok(), "Tuple with mut element should parse");
}

// ============================================================
// List pattern comprehensive tests
// ============================================================

#[test]
fn test_list_pattern_empty() {
    let result = parse("let [] = []");
    assert!(result.is_ok(), "Empty list pattern should parse");
}

#[test]
fn test_list_pattern_single_element() {
    let result = parse("let [x] = [1]");
    assert!(result.is_ok(), "Single element list should parse");
}

#[test]
fn test_list_pattern_with_trailing_rest() {
    let result = parse("let [first, second, ...rest] = arr");
    assert!(result.is_ok(), "List with trailing rest should parse");
}

#[test]
fn test_list_pattern_with_two_dot_rest() {
    let result = parse("let [head, ..tail] = arr");
    assert!(result.is_ok(), "List with two-dot rest should parse");
}

#[test]
fn test_list_pattern_rest_only() {
    let result = parse("let [..all] = arr");
    assert!(result.is_ok(), "List with only rest should parse");
}

#[test]
fn test_list_pattern_with_wildcards() {
    let result = parse("let [first, _, third] = arr");
    assert!(result.is_ok(), "List with wildcards should parse");
}

// ============================================================
// Struct pattern comprehensive tests
// ============================================================

#[test]
fn test_struct_pattern_single_field() {
    let result = parse("let Point { x } = point");
    assert!(result.is_ok(), "Struct with single field should parse");
}

#[test]
fn test_struct_pattern_three_fields() {
    let result = parse("let Color { r, g, b } = color");
    assert!(result.is_ok(), "Struct with three fields should parse");
}

#[test]
fn test_struct_pattern_rest_only() {
    let result = parse("let Point { .. } = point");
    assert!(result.is_ok(), "Struct with only rest should parse");
}

#[test]
fn test_struct_pattern_field_with_nested() {
    let result = parse("let Line { start: Point { x, y }, end } = line");
    assert!(result.is_ok(), "Struct with nested pattern should parse");
}

#[test]
fn test_struct_pattern_anonymous() {
    let result = parse("let { name, age } = person");
    assert!(result.is_ok(), "Anonymous struct pattern should parse");
}

#[test]
fn test_struct_pattern_trailing_comma() {
    let result = parse("let Point { x, y, } = point");
    assert!(result.is_ok(), "Struct with trailing comma should parse");
}

// ============================================================
// Match expression comprehensive tests
// ============================================================

#[test]
fn test_match_with_block_body() {
    let result = parse("match x { 1 => { let a = 1; a + 1 }, _ => 0 }");
    assert!(result.is_ok(), "Match with block body should parse");
}

#[test]
fn test_match_arrow_syntax() {
    let result = parse("match x { 1 -> true, _ -> false }");
    assert!(result.is_ok(), "Match with arrow syntax should parse");
}

#[test]
fn test_match_five_arms() {
    let result = parse(
        "match x { 0 => \"zero\", 1 => \"one\", 2 => \"two\", 3 => \"three\", _ => \"many\" }",
    );
    assert!(result.is_ok(), "Match with five arms should parse");
}

#[test]
fn test_match_guard_with_function_call() {
    let result = parse("match x { n if is_valid(n) => true, _ => false }");
    assert!(
        result.is_ok(),
        "Match with function call guard should parse"
    );
}

#[test]
fn test_match_guard_with_comparison() {
    let result = parse("match x { n if n >= 0 && n < 100 => true, _ => false }");
    assert!(result.is_ok(), "Match with comparison guard should parse");
}

#[test]
fn test_match_nested() {
    let result =
        parse("match x { Some(y) => match y { 1 => true, _ => false }, None => false }");
    assert!(result.is_ok(), "Nested match should parse");
}

#[test]
fn test_match_with_trailing_comma() {
    let result = parse("match x { 1 => true, 2 => false, }");
    assert!(result.is_ok(), "Match with trailing comma should parse");
}

// ============================================================
// If-let comprehensive tests
// ============================================================

#[test]
fn test_if_let_without_else() {
    let result = parse("if let Some(x) = opt { print(x) }");
    assert!(result.is_ok(), "If-let without else should parse");
}

#[test]
fn test_if_let_nested_pattern() {
    let result = parse("if let Some((a, b)) = opt { a + b } else { 0 }");
    assert!(result.is_ok(), "If-let with nested pattern should parse");
}

#[test]
fn test_if_let_ok_pattern() {
    let result = parse("if let Ok(val) = result { val } else { 0 }");
    assert!(result.is_ok(), "If-let with Ok should parse");
}

#[test]
fn test_if_let_err_pattern() {
    let result = parse("if let Err(e) = result { log(e) }");
    assert!(result.is_ok(), "If-let with Err should parse");
}

#[test]
fn test_if_let_struct_pattern() {
    let result = parse("if let Point { x, y } = point { x + y } else { 0 }");
    assert!(result.is_ok(), "If-let with struct should parse");
}

#[test]
fn test_if_let_chain() {
    let result = parse("if let Some(x) = a { x } else if let Some(y) = b { y } else { 0 }");
    assert!(result.is_ok(), "If-let chain should parse");
}

// ============================================================
// Or pattern comprehensive tests
// ============================================================

#[test]
fn test_or_pattern_three_alternatives() {
    let result = parse("match x { 1 | 2 | 3 => true, _ => false }");
    assert!(result.is_ok(), "Three-way or should parse");
}

#[test]
fn test_or_pattern_with_unit_variants() {
    // Or pattern with unit variants (no function calls in body)
    let result = parse("match x { A | B => 1, _ => 0 }");
    assert!(result.is_ok(), "Or with unit variants should parse");
}

#[test]
fn test_or_pattern_strings() {
    let result = parse(r#"match s { "yes" | "y" | "Y" => true, _ => false }"#);
    assert!(result.is_ok(), "Or with strings should parse");
}

#[test]
fn test_or_pattern_with_binding() {
    let result = parse("match x { Some(n) | Ok(n) => n, _ => 0 }");
    assert!(result.is_ok(), "Or with bindings should parse");
}

// ============================================================
// Range pattern comprehensive tests
// ============================================================

#[test]
fn test_range_pattern_exclusive() {
    let result = parse("match x { 0..10 => true, _ => false }");
    assert!(result.is_ok(), "Exclusive range should parse");
}

#[test]
fn test_range_pattern_inclusive() {
    let result = parse("match x { 0..=10 => true, _ => false }");
    assert!(result.is_ok(), "Inclusive range should parse");
}

#[test]
fn test_range_pattern_char_exclusive() {
    let result = parse("match c { 'a'..'z' => true, _ => false }");
    assert!(result.is_ok(), "Char exclusive range should parse");
}

#[test]
fn test_range_pattern_char_inclusive() {
    let result = parse("match c { 'a'..='z' => true, _ => false }");
    assert!(result.is_ok(), "Char inclusive range should parse");
}

// ============================================================
// At binding tests
// ============================================================

#[test]
fn test_at_binding_simple() {
    let result = parse("match x { n @ 1..10 => n, _ => 0 }");
    assert!(result.is_ok(), "At binding with range should parse");
}

#[test]
fn test_at_binding_with_pattern() {
    let result = parse("match opt { val @ Some(_) => val, None => None }");
    assert!(result.is_ok(), "At binding with Some should parse");
}

// ============================================================
// Qualified path patterns
// ============================================================

#[test]
fn test_qualified_enum_variant() {
    let result = parse("match x { Color::Red => 1, Color::Green => 2, _ => 0 }");
    assert!(result.is_ok(), "Qualified enum variant should parse");
}

#[test]
fn test_qualified_tuple_variant() {
    // Qualified with two alternatives - simpler form
    let result = parse("match x { Color::RGB(r, g, b) => r, _ => 0 }");
    assert!(result.is_ok(), "Qualified tuple variant should parse");
}

#[test]
fn test_qualified_struct_variant() {
    let result = parse("match x { Shape::Circle { radius } => radius, _ => 0.0 }");
    assert!(result.is_ok(), "Qualified struct variant should parse");
}

// ============================================================
// Let-else pattern tests
// ============================================================

#[test]
fn test_let_else_with_panic() {
    let result = parse("let Some(x) = opt else { panic(\"no value\") }");
    assert!(result.is_ok(), "Let-else with panic should parse");
}

#[test]
fn test_let_else_with_ok() {
    let result = parse("let Ok(val) = result else { return Err(e) }");
    assert!(result.is_ok(), "Let-else with Ok should parse");
}

#[test]
fn test_let_else_complex_block() {
    let result = parse(
        r#"let Some(x) = opt else {
        log("error")
        return 0
    }"#,
    );
    assert!(result.is_ok(), "Let-else with complex block should parse");
}

// ============================================================
// Edge cases and special patterns
// ============================================================

#[test]
fn test_pattern_in_function_param() {
    let result = parse("fun foo((x, y)) { x + y }");
    assert!(
        result.is_ok(),
        "Tuple pattern in function param should parse"
    );
}

#[test]
fn test_pattern_with_type_keyword_field() {
    let result = parse("let { type } = config");
    // May or may not parse - checking it doesn't crash
    let _ = result;
}

#[test]
fn test_constructor_empty_args() {
    let result = parse("match x { Unit() => true, _ => false }");
    assert!(result.is_ok(), "Empty constructor args should parse");
}

#[test]
fn test_constructor_three_args() {
    let result = parse("match x { Triple(a, b, c) => a + b + c }");
    assert!(result.is_ok(), "Constructor with three args should parse");
}

#[test]
fn test_match_complex_guard() {
    let result = parse("match (x, y) { (a, b) if a > 0 && b > 0 => true, _ => false }");
    assert!(result.is_ok(), "Match with complex guard should parse");
}

#[test]
fn test_pattern_result_identifier() {
    // Result is a keyword but can be used as identifier in some contexts
    let result = parse("let Result = compute()");
    assert!(result.is_ok(), "'Result' as identifier should parse");
}

#[test]
fn test_pattern_var_identifier() {
    let result = parse("let var = value");
    // May or may not work - var is a keyword
    let _ = result;
}

#[test]
fn test_empty_tuple_in_match() {
    let result = parse("match x { () => true }");
    assert!(result.is_ok(), "Empty tuple in match should parse");
}

#[test]
fn test_empty_list_in_match() {
    let result = parse("match arr { [] => true, _ => false }");
    assert!(result.is_ok(), "Empty list in match should parse");
}

// Property tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        #[ignore = "Property tests run with --ignored flag"] // Run with: cargo test property_tests -- --ignored
        fn prop_identifier_patterns_parse(name in "[a-z][a-z0-9_]*") {
            let code = format!("let {name} = 42");
            let result = Parser::new(&code).parse();
            prop_assert!(result.is_ok());
        }

        #[test]
        #[ignore = "Property tests run with --ignored flag"]
        fn prop_tuple_patterns_parse(a in "[a-z]+", b in "[a-z]+") {
            let code = format!("let ({a}, {b}) = (1, 2)");
            let result = Parser::new(&code).parse();
            prop_assert!(result.is_ok());
        }

        #[test]
        #[ignore = "Property tests run with --ignored flag"]
        fn prop_list_patterns_parse(name in "[a-z]+") {
            let code = format!("let [{name}, ...rest] = [1, 2, 3]");
            let result = Parser::new(&code).parse();
            prop_assert!(result.is_ok());
        }

        #[test]
        #[ignore = "Property tests run with --ignored flag"]
        fn prop_some_patterns_parse(inner in "[a-z]+") {
            let code = format!("let Some({inner}) = value");
            let result = Parser::new(&code).parse();
            prop_assert!(result.is_ok());
        }

        #[test]
        #[ignore = "Property tests run with --ignored flag"]
        fn prop_wildcard_always_parses(_seed in any::<u32>()) {
            let code = "let _ = 42";
            let result = Parser::new(code).parse();
            prop_assert!(result.is_ok());
        }

        #[test]
        #[ignore = "Property tests run with --ignored flag"]
        fn prop_literal_patterns_parse(n in 0i32..1000) {
            let code = format!("match x {{ {n} => true, _ => false }}");
            let result = Parser::new(&code).parse();
            prop_assert!(result.is_ok());
        }

        #[test]
        #[ignore = "Property tests run with --ignored flag"]
        fn prop_struct_patterns_parse(field in "[a-z]+") {
            let code = format!("let Point {{ {field} }} = p");
            let result = Parser::new(&code).parse();
            prop_assert!(result.is_ok());
        }
    }
}
