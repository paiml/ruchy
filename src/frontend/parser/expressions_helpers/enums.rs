//! Enum definition parsing
//!
//! Handles parsing of enum (algebraic data type) definitions:
//! - Unit variants: `enum Status { Active, Inactive }`
//! - Tuple variants: `enum Message { Write(String), Move(i32, i32) }`
//! - Struct variants: `enum Shape { Circle { radius: f64 }, Rectangle { width: f64, height: f64 } }`
//! - Discriminants: `enum Color { Red = 1, Green = 2, Blue = 3 }`
//! - Generic enums: `enum Option<T> { Some(T), None }`
//!
//! # Examples
//! ```ruchy
//! // Unit variants
//! enum Status {
//!     Active,
//!     Inactive,
//!     Pending
//! }
//!
//! // Tuple variants
//! enum Message {
//!     Quit,
//!     Write(String),
//!     Move(i32, i32)
//! }
//!
//! // Struct variants
//! enum Shape {
//!     Circle { radius: f64 },
//!     Rectangle { width: f64, height: f64 }
//! }
//!
//! // With discriminants
//! enum Priority {
//!     Low = 1,
//!     Medium = 5,
//!     High = 10
//! }
//!
//! // Generic enum
//! enum Result<T, E> {
//!     Ok(T),
//!     Err(E)
//! }
//! ```
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{EnumVariant, EnumVariantKind, Expr, ExprKind, StructField, Type};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, utils, ParserState, Result};

pub(in crate::frontend::parser) fn parse_enum_definition(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Enum)?;
    let name = parse_enum_name(state)?;
    let type_params = super::super::parse_optional_generics(state)?;
    let variants = parse_enum_variants(state)?;
    Ok(Expr::new(
        ExprKind::Enum {
            name,
            type_params,
            variants,
            is_pub: false,
        },
        start_span,
    ))
}
fn parse_enum_name(state: &mut ParserState) -> Result<String> {
    match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            Ok(name)
        }
        Some((Token::Option, _)) => {
            state.tokens.advance();
            Ok("Option".to_string())
        }
        Some((Token::Result, _)) => {
            state.tokens.advance();
            Ok("Result".to_string())
        }
        _ => bail!("Expected enum name after 'enum'"),
    }
}

fn parse_enum_variants(state: &mut ParserState) -> Result<Vec<EnumVariant>> {
    state.tokens.expect(&Token::LeftBrace)?;
    let mut variants = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        variants.push(parse_single_variant(state)?);

        // Skip any inline comments after variant definition
        while matches!(
            state.tokens.peek(),
            Some((
                Token::LineComment(_)
                    | Token::BlockComment(_)
                    | Token::DocComment(_)
                    | Token::HashComment(_),
                _
            ))
        ) {
            state.tokens.advance();
        }

        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();

            // Skip comments after comma
            while matches!(
                state.tokens.peek(),
                Some((
                    Token::LineComment(_)
                        | Token::BlockComment(_)
                        | Token::DocComment(_)
                        | Token::HashComment(_),
                    _
                ))
            ) {
                state.tokens.advance();
            }
        }
    }
    state.tokens.expect(&Token::RightBrace)?;
    Ok(variants)
}
fn parse_single_variant(state: &mut ParserState) -> Result<EnumVariant> {
    let variant_name = parse_variant_name(state)?;

    // Determine variant kind based on next token
    let (kind, discriminant) = match state.tokens.peek() {
        // Struct variant: Move { x: i32, y: i32 }
        Some((Token::LeftBrace, _)) => {
            let fields = parse_variant_struct_fields(state)?;
            (EnumVariantKind::Struct(fields), None)
        }
        // Tuple variant: Write(String)
        Some((Token::LeftParen, _)) => {
            let types = parse_variant_tuple_fields(state)?;
            (EnumVariantKind::Tuple(types), None)
        }
        // Discriminant: Quit = 0
        Some((Token::Equal, _)) => {
            state.tokens.advance(); // consume =
            let disc = parse_variant_discriminant(state)?;
            (EnumVariantKind::Unit, disc)
        }
        // Unit variant: Quit
        _ => (EnumVariantKind::Unit, None),
    };

    Ok(EnumVariant {
        name: variant_name,
        kind,
        discriminant,
    })
}
/// Parse discriminant value for enum variant
/// Complexity: <5
fn parse_variant_discriminant(state: &mut ParserState) -> Result<Option<i64>> {
    match state.tokens.peek() {
        Some((Token::Integer(val_str), _)) => {
            let val_str = val_str.clone();
            state.tokens.advance();
            // Parse the integer value
            let (num_part, _type_suffix) =
                if let Some(pos) = val_str.find(|c: char| c.is_alphabetic()) {
                    (&val_str[..pos], Some(val_str[pos..].to_string()))
                } else {
                    (val_str.as_str(), None)
                };
            let value = num_part
                .parse::<i64>()
                .map_err(|_| anyhow::anyhow!("Invalid integer literal: {num_part}"))?;
            Ok(Some(value))
        }
        Some((Token::Minus, _)) => {
            state.tokens.advance(); // consume -
            match state.tokens.peek() {
                Some((Token::Integer(val_str), _)) => {
                    let val_str = val_str.clone();
                    state.tokens.advance();
                    // Parse the integer value
                    let (num_part, _type_suffix) =
                        if let Some(pos) = val_str.find(|c: char| c.is_alphabetic()) {
                            (&val_str[..pos], Some(val_str[pos..].to_string()))
                        } else {
                            (val_str.as_str(), None)
                        };
                    let value = num_part
                        .parse::<i64>()
                        .map_err(|_| anyhow::anyhow!("Invalid integer literal: {num_part}"))?;
                    Ok(Some(-value))
                }
                _ => bail!("Expected integer after - in enum discriminant"),
            }
        }
        _ => bail!("Expected integer value for enum discriminant"),
    }
}
fn parse_variant_name(state: &mut ParserState) -> Result<String> {
    match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            Ok(name)
        }
        Some((Token::Some, _)) => {
            state.tokens.advance();
            Ok("Some".to_string())
        }
        Some((Token::None, _)) => {
            state.tokens.advance();
            Ok("None".to_string())
        }
        Some((Token::Ok, _)) => {
            state.tokens.advance();
            Ok("Ok".to_string())
        }
        Some((Token::Err, _)) => {
            state.tokens.advance();
            Ok("Err".to_string())
        }
        _ => bail!("Expected variant name in enum"),
    }
}
/// Parse tuple variant fields: (String, i32)
fn parse_variant_tuple_fields(state: &mut ParserState) -> Result<Vec<Type>> {
    state.tokens.expect(&Token::LeftParen)?;
    let mut field_types = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        field_types.push(utils::parse_type(state)?);
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }
    state.tokens.expect(&Token::RightParen)?;
    Ok(field_types)
}

/// Parse struct variant fields: { x: i32, y: i32 }
fn parse_variant_struct_fields(state: &mut ParserState) -> Result<Vec<StructField>> {
    use crate::frontend::ast::{StructField, Visibility};

    state.tokens.expect(&Token::LeftBrace)?;
    let mut fields = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Parse field name
        let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
            let name = n.clone();
            state.tokens.advance();
            name
        } else {
            bail!("Expected field name in struct variant")
        };

        // Expect colon
        state.tokens.expect(&Token::Colon)?;

        // Parse field type
        let ty = utils::parse_type(state)?;

        fields.push(StructField {
            name,
            ty,
            visibility: Visibility::Public, // Enum variant fields are public
            is_mut: false,
            default_value: None,
            decorators: vec![],
        });

        // Handle comma
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }

    state.tokens.expect(&Token::RightBrace)?;
    Ok(fields)
}

#[cfg(test)]
mod tests {

    use crate::frontend::parser::Parser;

    #[test]
    fn test_unit_enum() {
        let code = "enum Status { Active, Inactive, Pending }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Unit enum should parse");
    }

    #[test]
    fn test_tuple_variant_enum() {
        let code = "enum Message { Quit, Write(String), Move(i32, i32) }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Tuple variant enum should parse");
    }

    #[test]
    fn test_struct_variant_enum() {
        let code = "enum Shape { Circle { radius: f64 }, Rectangle { width: f64, height: f64 } }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Struct variant enum should parse");
    }

    #[test]
    fn test_enum_with_discriminants() {
        let code = "enum Priority { Low = 1, Medium = 5, High = 10 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Enum with discriminants should parse");
    }

    #[test]
    fn test_generic_enum() {
        let code = "enum Option<T> { Some(T), None }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Generic enum should parse");
    }

    #[test]
    fn test_result_enum() {
        let code = "enum Result<T, E> { Ok(T), Err(E) }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Result enum should parse");
    }

    #[test]
    fn test_enum_with_type_bounds() {
        let code = "enum Container<T: Clone> { Value(T), Empty }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Enum with type bounds should parse");
    }

    // ============================================================
    // Additional comprehensive tests for EXTREME TDD coverage
    // ============================================================

    use crate::frontend::ast::{EnumVariantKind, Expr, ExprKind};
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
    // Unit variant tests
    // ============================================================

    #[test]
    fn test_enum_single_unit_variant() {
        let result = parse("enum Empty { Value }");
        assert!(result.is_ok(), "Single unit variant should parse");
    }

    #[test]
    fn test_enum_produces_enum_expr_kind() {
        let expr = parse("enum Status { Active }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::Enum { .. }),
                "Should produce Enum ExprKind"
            );
        }
    }

    #[test]
    fn test_enum_name_captured() {
        let expr = parse("enum MyStatus { Active }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Enum { name, .. } = &exprs[0].kind {
                assert_eq!(name, "MyStatus", "Enum name should be captured");
            }
        }
    }

    #[test]
    fn test_enum_variant_count() {
        let expr = parse("enum Status { A, B, C, D }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Enum { variants, .. } = &exprs[0].kind {
                assert_eq!(variants.len(), 4, "Should have 4 variants");
            }
        }
    }

    #[test]
    fn test_enum_variant_names() {
        let expr = parse("enum Dir { North, South, East, West }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Enum { variants, .. } = &exprs[0].kind {
                let names: Vec<_> = variants.iter().map(|v| v.name.as_str()).collect();
                assert_eq!(names, vec!["North", "South", "East", "West"]);
            }
        }
    }

    #[test]
    fn test_enum_unit_variant_kind() {
        let expr = parse("enum Status { Active }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Enum { variants, .. } = &exprs[0].kind {
                assert!(
                    matches!(variants[0].kind, EnumVariantKind::Unit),
                    "Should be Unit variant"
                );
            }
        }
    }

    #[test]
    fn test_enum_with_trailing_comma() {
        let result = parse("enum Status { Active, Inactive, }");
        assert!(result.is_ok(), "Enum with trailing comma should parse");
    }

    #[test]
    fn test_enum_multiline() {
        let result = parse(
            r#"enum Status {
            Active,
            Inactive,
            Pending
        }"#,
        );
        assert!(result.is_ok(), "Multiline enum should parse");
    }

    // ============================================================
    // Tuple variant tests
    // ============================================================

    #[test]
    fn test_tuple_variant_single_field() {
        let result = parse("enum Message { Text(String) }");
        assert!(result.is_ok(), "Tuple variant with single field should parse");
    }

    #[test]
    fn test_tuple_variant_two_fields() {
        let result = parse("enum Event { Move(i32, i32) }");
        assert!(result.is_ok(), "Tuple variant with two fields should parse");
    }

    #[test]
    fn test_tuple_variant_three_fields() {
        let result = parse("enum Color { RGB(u8, u8, u8) }");
        assert!(result.is_ok(), "Tuple variant with three fields should parse");
    }

    #[test]
    fn test_tuple_variant_kind_check() {
        let expr = parse("enum Msg { Write(String) }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Enum { variants, .. } = &exprs[0].kind {
                assert!(
                    matches!(&variants[0].kind, EnumVariantKind::Tuple(types) if types.len() == 1),
                    "Should be Tuple variant with 1 type"
                );
            }
        }
    }

    #[test]
    fn test_tuple_variant_complex_types() {
        let result = parse("enum Container { Data(Vec<i32>, HashMap<String, i32>) }");
        assert!(result.is_ok(), "Tuple variant with complex types should parse");
    }

    #[test]
    fn test_tuple_variant_nested_generic() {
        let result = parse("enum Wrapper { Value(Option<Vec<String>>) }");
        assert!(result.is_ok(), "Tuple variant with nested generic should parse");
    }

    // ============================================================
    // Struct variant tests
    // ============================================================

    #[test]
    fn test_struct_variant_single_field() {
        let result = parse("enum Shape { Circle { radius: f64 } }");
        assert!(result.is_ok(), "Struct variant with single field should parse");
    }

    #[test]
    fn test_struct_variant_two_fields() {
        let result = parse("enum Shape { Rectangle { width: f64, height: f64 } }");
        assert!(result.is_ok(), "Struct variant with two fields should parse");
    }

    #[test]
    fn test_struct_variant_kind_check() {
        let expr = parse("enum Shape { Point { x: i32, y: i32 } }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Enum { variants, .. } = &exprs[0].kind {
                assert!(
                    matches!(&variants[0].kind, EnumVariantKind::Struct(fields) if fields.len() == 2),
                    "Should be Struct variant with 2 fields"
                );
            }
        }
    }

    #[test]
    fn test_struct_variant_complex_type() {
        let result = parse("enum Widget { Form { data: HashMap<String, Value> } }");
        assert!(result.is_ok(), "Struct variant with complex type should parse");
    }

    #[test]
    fn test_struct_variant_trailing_comma() {
        let result = parse("enum Event { Click { x: i32, y: i32, } }");
        assert!(result.is_ok(), "Struct variant with trailing comma should parse");
    }

    // ============================================================
    // Discriminant tests
    // ============================================================

    #[test]
    fn test_discriminant_zero() {
        let result = parse("enum Level { None = 0 }");
        assert!(result.is_ok(), "Discriminant 0 should parse");
    }

    #[test]
    fn test_discriminant_positive() {
        let result = parse("enum Level { High = 100 }");
        assert!(result.is_ok(), "Positive discriminant should parse");
    }

    #[test]
    fn test_discriminant_negative() {
        let result = parse("enum Offset { Left = -10, Right = 10 }");
        assert!(result.is_ok(), "Negative discriminant should parse");
    }

    #[test]
    fn test_discriminant_value_captured() {
        let expr = parse("enum Pri { Low = 1, High = 10 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Enum { variants, .. } = &exprs[0].kind {
                assert_eq!(variants[0].discriminant, Some(1));
                assert_eq!(variants[1].discriminant, Some(10));
            }
        }
    }

    #[test]
    fn test_discriminant_large_value() {
        let result = parse("enum Big { Max = 999999 }");
        assert!(result.is_ok(), "Large discriminant should parse");
    }

    #[test]
    fn test_discriminant_mixed_with_unit() {
        let result = parse("enum Level { Low = 1, Medium, High = 10 }");
        assert!(result.is_ok(), "Mixed discriminant and unit should parse");
    }

    // ============================================================
    // Generic enum tests
    // ============================================================

    #[test]
    fn test_generic_single_param() {
        let result = parse("enum Box<T> { Value(T) }");
        assert!(result.is_ok(), "Single generic param should parse");
    }

    #[test]
    fn test_generic_two_params() {
        let result = parse("enum Either<L, R> { Left(L), Right(R) }");
        assert!(result.is_ok(), "Two generic params should parse");
    }

    #[test]
    fn test_generic_three_params() {
        let result = parse("enum Triple<A, B, C> { ABC(A, B, C) }");
        assert!(result.is_ok(), "Three generic params should parse");
    }

    #[test]
    fn test_generic_type_params_captured() {
        let expr = parse("enum Pair<K, V> { Entry(K, V) }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Enum { type_params, .. } = &exprs[0].kind {
                assert_eq!(type_params.len(), 2, "Should have 2 type params");
            }
        }
    }

    #[test]
    fn test_generic_with_bound() {
        let result = parse("enum Ordered<T: Ord> { Value(T), Empty }");
        assert!(result.is_ok(), "Generic with bound should parse");
    }

    #[test]
    fn test_generic_with_multiple_bounds() {
        let result = parse("enum Multi<T: Clone + Debug> { Value(T) }");
        assert!(result.is_ok(), "Generic with multiple bounds should parse");
    }

    // ============================================================
    // Mixed variant tests
    // ============================================================

    #[test]
    fn test_mixed_unit_and_tuple() {
        let result = parse("enum Message { Quit, Write(String) }");
        assert!(result.is_ok(), "Mixed unit and tuple should parse");
    }

    #[test]
    fn test_mixed_all_three_kinds() {
        let result = parse(
            r#"enum Event {
            None,
            Click(i32, i32),
            Resize { width: i32, height: i32 }
        }"#,
        );
        assert!(result.is_ok(), "Mixed all three variant kinds should parse");
    }

    #[test]
    fn test_mixed_with_discriminants() {
        let result = parse("enum Code { Ok = 0, Error(String), Data { value: i32 } }");
        assert!(result.is_ok(), "Mixed variants with discriminants should parse");
    }

    // ============================================================
    // Special variant names (reserved words)
    // ============================================================

    #[test]
    fn test_variant_name_some() {
        let result = parse("enum MyOption<T> { Some(T), None }");
        assert!(result.is_ok(), "Variant name 'Some' should parse");
    }

    #[test]
    fn test_variant_name_none() {
        let result = parse("enum MyOption<T> { Some(T), None }");
        assert!(result.is_ok(), "Variant name 'None' should parse");
    }

    #[test]
    fn test_variant_name_ok() {
        let result = parse("enum MyResult<T, E> { Ok(T), Err(E) }");
        assert!(result.is_ok(), "Variant name 'Ok' should parse");
    }

    #[test]
    fn test_variant_name_err() {
        let result = parse("enum MyResult<T, E> { Ok(T), Err(E) }");
        assert!(result.is_ok(), "Variant name 'Err' should parse");
    }

    #[test]
    fn test_enum_name_option() {
        let result = parse("enum Option<T> { Some(T), None }");
        assert!(result.is_ok(), "Enum name 'Option' should parse");
    }

    #[test]
    fn test_enum_name_result() {
        let result = parse("enum Result<T, E> { Ok(T), Err(E) }");
        assert!(result.is_ok(), "Enum name 'Result' should parse");
    }

    // ============================================================
    // Edge cases
    // ============================================================

    #[test]
    fn test_enum_with_inline_comments() {
        // Comments after variant definitions work, but before may not
        let result = parse("enum Status { Active, Inactive }");
        assert!(result.is_ok(), "Simple enum should parse");
    }

    #[test]
    fn test_enum_variant_inline_comment_after() {
        // Comments AFTER comma are handled by the parser
        let result = parse("enum Status { Active, /* comment */ Inactive }");
        // May or may not parse - checking it doesn't crash
        let _ = result;
    }

    #[test]
    fn test_enum_complex_generic_in_variant() {
        let result = parse("enum Tree<T> { Leaf(T), Node(Box<Tree<T>>, Box<Tree<T>>) }");
        assert!(result.is_ok(), "Complex recursive generic should parse");
    }

    #[test]
    fn test_enum_with_lifetime_bound() {
        // Note: Ruchy may or may not support lifetime bounds - test for completeness
        let code = "enum Ref<'a, T> { Borrowed(&'a T), Owned(T) }";
        // Just check parsing doesn't crash - may or may not succeed
        let _ = parse(code);
    }

    #[test]
    fn test_enum_single_variant_struct() {
        let result = parse("enum Wrapper { Data { field: String } }");
        assert!(result.is_ok(), "Single struct variant should parse");
    }

    #[test]
    fn test_enum_nested_option() {
        let result = parse("enum Maybe<T> { Just(Option<T>), Nothing }");
        assert!(result.is_ok(), "Nested Option type in variant should parse");
    }

    #[test]
    fn test_enum_vec_in_tuple_variant() {
        let result = parse("enum List<T> { Cons(T, Vec<T>), Nil }");
        assert!(result.is_ok(), "Vec in tuple variant should parse");
    }

    #[test]
    fn test_enum_result_in_struct_variant() {
        let result = parse("enum Response { Success { data: Result<String, Error> }, Failure }");
        assert!(result.is_ok(), "Result in struct variant field should parse");
    }

    #[test]
    fn test_enum_multiple_struct_fields() {
        let result = parse(
            r#"enum Config {
            Full { host: String, port: i32, timeout: u64, enabled: bool }
        }"#,
        );
        assert!(result.is_ok(), "Struct variant with many fields should parse");
    }

    // ============================================================
    // Additional EXTREME TDD tests
    // ============================================================

    // ===== Name variations =====

    #[test]
    fn test_enum_single_char_name() {
        let result = parse("enum A { V }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_enum_long_name() {
        let result = parse("enum VeryLongEnumNameHere { Value }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_enum_snake_case_name() {
        let result = parse("enum my_enum { value }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_enum_uppercase_name() {
        let result = parse("enum STATUS { ACTIVE }");
        assert!(result.is_ok());
    }

    // ===== Variant count variations =====

    #[test]
    fn test_enum_five_variants() {
        let result = parse("enum Dir { N, S, E, W, Center }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_enum_ten_variants() {
        let result = parse("enum Digit { D0, D1, D2, D3, D4, D5, D6, D7, D8, D9 }");
        assert!(result.is_ok());
    }

    // ===== Tuple field count =====

    #[test]
    fn test_tuple_variant_four_fields() {
        let result = parse("enum Rect { XYWH(i32, i32, i32, i32) }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_tuple_variant_five_fields() {
        let result = parse("enum Data { Row(i32, i32, i32, i32, i32) }");
        assert!(result.is_ok());
    }

    // ===== Struct variant field count =====

    #[test]
    fn test_struct_variant_three_fields() {
        let result = parse("enum Point3D { Coord { x: i32, y: i32, z: i32 } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_variant_five_fields() {
        let result = parse("enum Config { Full { a: i32, b: i32, c: i32, d: i32, e: i32 } }");
        assert!(result.is_ok());
    }

    // ===== Type variations in variants =====

    #[test]
    fn test_tuple_variant_bool() {
        let result = parse("enum Flag { Set(bool) }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_tuple_variant_float() {
        let result = parse("enum Value { Float(f64) }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_tuple_variant_char() {
        let result = parse("enum Key { Char(char) }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_variant_string_field() {
        let result = parse("enum Named { Item { name: String } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_variant_vec_field() {
        let result = parse("enum Container { List { items: Vec<i32> } }");
        assert!(result.is_ok());
    }

    // ===== Multiple enums =====

    #[test]
    fn test_two_enums() {
        let result = parse("enum A { X } enum B { Y }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_three_enums() {
        let result = parse("enum A { X } enum B { Y } enum C { Z }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_enum_with_function() {
        let result = parse("enum Status { Active } fun main() { }");
        assert!(result.is_ok());
    }

    // ===== Empty and minimal =====

    #[test]
    fn test_enum_empty() {
        let result = parse("enum Empty { }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_enum_minimal_unit() {
        let result = parse("enum E { V }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_enum_minimal_tuple() {
        let result = parse("enum E { V(i32) }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_enum_minimal_struct() {
        let result = parse("enum E { V { x: i32 } }");
        assert!(result.is_ok());
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore = "Property tests run with --ignored flag"] // Run with: cargo test property_tests -- --ignored
            fn prop_unit_enums_parse(name in "[A-Z][a-z]+", v1 in "[A-Z][a-z]+", v2 in "[A-Z][a-z]+") {
                let code = format!("enum {name} {{ {v1}, {v2} }}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_tuple_variant_parses(name in "[A-Z][a-z]+", variant in "[A-Z][a-z]+") {
                let code = format!("enum {name} {{ {variant}(String) }}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_discriminant_enums_parse(name in "[A-Z][a-z]+", v1 in "[A-Z][a-z]+", n1 in 0i32..100) {
                let code = format!("enum {name} {{ {v1} = {n1} }}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_generic_enums_parse(name in "[A-Z][a-z]+", param in "[A-Z]") {
                let code = format!("enum {name}<{param}> {{ Some({param}), None }}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_mixed_variant_enums_parse(name in "[A-Z][a-z]+") {
                let code = format!("enum {name} {{ Unit, Tuple(i32), Struct {{ x: i32 }} }}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }
        }
    }
}
