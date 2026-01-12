//! Struct definition parsing
//!
//! Handles parsing of struct (record type) definitions:
//! - Named structs: `struct Point { x: f64, y: f64 }`
//! - Tuple structs: `struct Color(u8, u8, u8)`
//! - Unit structs: `struct Marker`
//! - Generic structs: `struct Container<T> { value: T }`
//! - Field visibility: `pub`, `pub(crate)`, `private`
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{
    ClassMethod, Expr, ExprKind, SelfType, Span, StructField, Type, Visibility,
};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, parse_expr_recursive, utils, ParserState, Result};

pub(in crate::frontend::parser) fn parse_struct_variant(
    state: &mut ParserState,
    name: String,
    type_params: Vec<String>,
    start_span: Span,
) -> Result<Expr> {
    match state.tokens.peek() {
        Some((Token::LeftParen, _)) => {
            let fields = parse_tuple_struct_fields(state)?;
            Ok(Expr::new(
                ExprKind::TupleStruct {
                    name,
                    type_params,
                    fields,
                    derives: Vec::new(),
                    is_pub: false,
                },
                start_span,
            ))
        }
        Some((Token::LeftBrace, _)) => {
            let (fields, methods) = parse_struct_fields(state)?;
            Ok(Expr::new(
                ExprKind::Struct {
                    name,
                    type_params,
                    fields,
                    methods,
                    derives: Vec::new(),
                    is_pub: false,
                },
                start_span,
            ))
        }
        _ => Ok(Expr::new(
            ExprKind::Struct {
                name,
                type_params,
                fields: Vec::new(),
                methods: Vec::new(),
                derives: Vec::new(),
                is_pub: false,
            },
            start_span,
        )),
    }
}

pub(in crate::frontend::parser) fn parse_struct_name(state: &mut ParserState) -> Result<String> {
    if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        Ok(name)
    } else {
        bail!("Expected struct name after 'struct'");
    }
}

fn parse_tuple_struct_fields(state: &mut ParserState) -> Result<Vec<Type>> {
    state.tokens.expect(&Token::LeftParen)?;
    let mut fields = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        fields.push(utils::parse_type(state)?);

        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }

    state.tokens.expect(&Token::RightParen)?;
    Ok(fields)
}

fn parse_struct_fields(state: &mut ParserState) -> Result<(Vec<StructField>, Vec<ClassMethod>)> {
    state.tokens.expect(&Token::LeftBrace)?;
    let mut fields = Vec::new();
    let mut methods = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // DEFECT-PARSER-007: Skip comments before member declaration
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

        // PARSER-147: Check if this is a method definition (with or without pub)
        if is_method_definition(state) {
            let method = parse_struct_method_with_visibility(state)?;
            methods.push(method);
        } else {
            // Parse field
            let (visibility, is_mut) = parse_struct_field_modifiers(state)?;
            let (field_name, field_type, default_value) = parse_single_struct_field(state)?;

            fields.push(StructField {
                name: field_name,
                ty: field_type,
                visibility,
                is_mut,
                default_value,
                decorators: vec![],
            });
        }

        // DEFECT-PARSER-007: Skip any inline comments after member definition
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

            // Skip comments after comma (allows multiline definitions with comments)
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
    Ok((fields, methods))
}

fn parse_struct_field_modifiers(state: &mut ParserState) -> Result<(Visibility, bool)> {
    let visibility = if matches!(state.tokens.peek(), Some((Token::Pub, _))) {
        parse_pub_visibility(state)?
    } else if matches!(state.tokens.peek(), Some((Token::Private, _))) {
        parse_private_keyword(state);
        Visibility::Private
    } else {
        Visibility::Private
    };

    let is_mut = parse_mut_modifier(state);
    Ok((visibility, is_mut))
}

fn parse_pub_visibility(state: &mut ParserState) -> Result<Visibility> {
    state.tokens.expect(&Token::Pub)?;

    if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        parse_scoped_visibility(state)
    } else {
        Ok(Visibility::Public)
    }
}

fn parse_scoped_visibility(state: &mut ParserState) -> Result<Visibility> {
    state.tokens.expect(&Token::LeftParen)?;

    // PARSER-074: Match Token::Crate and Token::Super (not Identifier)
    let visibility = match state.tokens.peek() {
        Some((Token::Crate, _)) => {
            state.tokens.advance();
            Visibility::PubCrate
        }
        Some((Token::Super, _)) => {
            state.tokens.advance();
            Visibility::PubSuper
        }
        _ => Visibility::Public,
    };

    state.tokens.expect(&Token::RightParen)?;
    Ok(visibility)
}

fn parse_mut_modifier(state: &mut ParserState) -> bool {
    if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
        state.tokens.advance();
        true
    } else {
        false
    }
}

fn parse_private_keyword(state: &mut ParserState) {
    if matches!(state.tokens.peek(), Some((Token::Private, _))) {
        state.tokens.advance();
    }
}

pub(in crate::frontend::parser) fn parse_single_struct_field(
    state: &mut ParserState,
) -> Result<(String, Type, Option<Expr>)> {
    let field_name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected field name");
    };

    state.tokens.expect(&Token::Colon)?;
    let field_type = utils::parse_type(state)?;

    let default_value = if matches!(state.tokens.peek(), Some((Token::Equal, _))) {
        state.tokens.advance();
        Some(parse_expr_recursive(state)?)
    } else {
        None
    };

    Ok((field_name, field_type, default_value))
}

// PARSER-147: Helper to detect if next tokens are a method definition
fn is_method_definition(state: &mut ParserState) -> bool {
    // Check for: fun/fn OR pub fun/fn
    match state.tokens.peek() {
        Some((Token::Fun | Token::Fn, _)) => true,
        Some((Token::Pub, _)) => {
            // Lookahead: check if next token after pub is fun/fn
            matches!(
                state.tokens.peek_ahead(1),
                Some((Token::Fun | Token::Fn, _))
            )
        }
        _ => false,
    }
}

// PARSER-147: Parse method with optional pub visibility modifier
fn parse_struct_method_with_visibility(state: &mut ParserState) -> Result<ClassMethod> {
    // Parse optional pub keyword
    let is_pub = if matches!(state.tokens.peek(), Some((Token::Pub, _))) {
        state.tokens.advance();
        true
    } else {
        false
    };

    // Parse the method (reuse existing logic)
    let mut method = parse_struct_method(state)?;

    // Update visibility
    method.is_pub = is_pub;

    Ok(method)
}

fn parse_struct_method(state: &mut ParserState) -> Result<ClassMethod> {
    // Expect 'fun' or 'fn' keyword
    match state.tokens.peek() {
        Some((Token::Fun, _)) => {
            state.tokens.advance();
        }
        Some((Token::Fn, _)) => {
            state.tokens.advance();
        }
        _ => bail!("Expected 'fun' or 'fn' keyword for method definition"),
    }

    // Parse method name
    let method_name = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected method name after 'fun'");
    };

    // Parse parameter list (including self parameter)
    let params = utils::parse_params(state)?;

    // Determine self type from first parameter
    let self_type = determine_self_type(&params);

    // Parse optional return type
    let return_type = if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
        state.tokens.advance();
        Some(utils::parse_type(state)?)
    } else {
        None
    };

    // Parse method body
    let body = Box::new(parse_expr_recursive(state)?);

    Ok(ClassMethod {
        name: method_name,
        params,
        return_type,
        body,
        is_pub: false,
        is_static: matches!(self_type, SelfType::None),
        is_override: false,
        is_final: false,
        is_abstract: false,
        is_async: false,
        self_type,
    })
}

fn determine_self_type(params: &[crate::frontend::ast::Param]) -> SelfType {
    if !params.is_empty() && params[0].name() == "self" {
        use crate::frontend::ast::TypeKind;
        match &params[0].ty.kind {
            TypeKind::Reference { is_mut: true, .. } => SelfType::MutBorrowed,
            TypeKind::Reference { is_mut: false, .. } => SelfType::Borrowed,
            _ => SelfType::Owned,
        }
    } else {
        SelfType::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::Parser;

    // Helper to parse code
    fn parse(code: &str) -> Result<Expr> {
        let mut parser = Parser::new(code);
        parser.parse()
    }

    // Helper to extract block expressions
    fn get_block_exprs(expr: &Expr) -> Option<&Vec<Expr>> {
        match &expr.kind {
            ExprKind::Block(exprs) => Some(exprs),
            _ => None,
        }
    }

    // ===== parse_struct_variant tests =====

    #[test]
    fn test_named_struct() {
        let code = "struct Point { x: f64, y: f64 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Named struct should parse");
    }

    #[test]
    fn test_named_struct_fields() {
        let expr = parse("struct Point { x: i32, y: i32 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Struct { name, fields, .. } = &exprs[0].kind {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
            }
        }
    }

    #[test]
    fn test_tuple_struct() {
        let code = "struct Color(u8, u8, u8)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Tuple struct should parse");
    }

    #[test]
    fn test_tuple_struct_fields() {
        let expr = parse("struct Pair(i32, i32)").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::TupleStruct { name, fields, .. } = &exprs[0].kind {
                assert_eq!(name, "Pair");
                assert_eq!(fields.len(), 2);
            }
        }
    }

    #[test]
    fn test_tuple_struct_single_field() {
        let expr = parse("struct Wrapper(String)").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::TupleStruct { fields, .. } = &exprs[0].kind {
                assert_eq!(fields.len(), 1);
            }
        }
    }

    #[test]
    fn test_unit_struct() {
        let code = "struct Marker";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Unit struct should parse");
    }

    #[test]
    fn test_unit_struct_empty_fields() {
        let expr = parse("struct Empty").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Struct { fields, .. } = &exprs[0].kind {
                assert!(fields.is_empty());
            }
        }
    }

    // ===== parse_struct_name tests =====

    #[test]
    fn test_struct_name_simple() {
        let result = parse("struct Foo { }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_name_with_numbers() {
        let result = parse("struct Point2D { x: i32, y: i32 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_name_snake_case() {
        let result = parse("struct my_struct { value: i32 }");
        assert!(result.is_ok());
    }

    // ===== Generic struct tests =====

    #[test]
    fn test_generic_struct() {
        let code = "struct Container<T> { value: T }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Generic struct should parse");
    }

    #[test]
    fn test_generic_struct_type_params() {
        let expr = parse("struct Box<T> { inner: T }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Struct { type_params, .. } = &exprs[0].kind {
                assert_eq!(type_params.len(), 1);
                assert_eq!(type_params[0], "T");
            }
        }
    }

    #[test]
    fn test_generic_struct_multiple_params() {
        let expr = parse("struct Pair<K, V> { key: K, value: V }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Struct { type_params, .. } = &exprs[0].kind {
                assert_eq!(type_params.len(), 2);
            }
        }
    }

    #[test]
    fn test_generic_tuple_struct() {
        let result = parse("struct Wrapper<T>(T)");
        assert!(result.is_ok());
    }

    // ===== Field visibility tests =====

    #[test]
    fn test_pub_field() {
        let code = "struct Point { pub x: f64, y: f64 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Struct with pub field should parse");
    }

    #[test]
    fn test_pub_crate_field() {
        let result = parse("struct Data { pub(crate) value: i32 }");
        assert!(result.is_ok(), "Struct with pub(crate) field should parse");
    }

    #[test]
    fn test_pub_super_field() {
        let result = parse("struct Data { pub(super) value: i32 }");
        assert!(result.is_ok(), "Struct with pub(super) field should parse");
    }

    #[test]
    fn test_private_field() {
        let result = parse("struct Secret { private data: String }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_mixed_visibility_fields() {
        let result = parse("struct Mixed { pub a: i32, private b: i32, c: i32 }");
        assert!(result.is_ok());
    }

    // ===== Mutable field tests =====

    #[test]
    fn test_mut_field() {
        let code = "struct Counter { mut count: i32 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Struct with mut field should parse");
    }

    #[test]
    fn test_pub_mut_field() {
        let result = parse("struct Counter { pub mut count: i32 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_mixed_mutability() {
        let result = parse("struct State { mut counter: i32, readonly: String }");
        assert!(result.is_ok());
    }

    // ===== Default value tests =====

    #[test]
    fn test_field_with_default() {
        let code = "struct Config { timeout: i32 = 30 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Struct with default value should parse");
    }

    #[test]
    fn test_field_with_string_default() {
        let result = parse("struct Config { name: String = \"default\" }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_field_with_expression_default() {
        let result = parse("struct Config { value: i32 = 10 + 20 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_fields_with_defaults() {
        let result = parse("struct Config { a: i32 = 1, b: i32 = 2, c: i32 = 3 }");
        assert!(result.is_ok());
    }

    // ===== Struct with methods tests (PARSER-147) =====

    #[test]
    fn test_struct_with_method() {
        let result = parse(
            "struct Counter { count: i32, fun inc(&mut self) { self.count = self.count + 1 } }",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_with_pub_method() {
        let result =
            parse("struct Counter { count: i32, pub fun get(&self) -> i32 { self.count } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_with_static_method() {
        let result = parse("struct Point { x: i32, y: i32, fun new(x: i32, y: i32) -> Point { Point { x: x, y: y } } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_with_multiple_methods() {
        let result =
            parse("struct Counter { count: i32, fun inc(&mut self) { } fun dec(&mut self) { } }");
        assert!(result.is_ok());
    }

    // ===== Field types tests =====

    #[test]
    fn test_field_with_array_type() {
        let result = parse("struct Data { items: Vec<i32> }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_field_with_option_type() {
        let result = parse("struct Config { value: Option<String> }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_field_with_result_type() {
        let result = parse("struct Response { result: Result<Data, Error> }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_field_with_reference_type() {
        let result = parse("struct View { data: &str }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_field_with_tuple_type() {
        let result = parse("struct Pair { value: (i32, i32) }");
        assert!(result.is_ok());
    }

    // ===== Comment handling tests (DEFECT-PARSER-007) =====

    #[test]
    fn test_struct_with_line_comments() {
        let result =
            parse("struct Point {\n  // x coordinate\n  x: i32,\n  // y coordinate\n  y: i32\n}");
        assert!(result.is_ok(), "Struct with line comments should parse");
    }

    #[test]
    fn test_struct_with_inline_comment() {
        let result = parse("struct Point { x: i32, // inline comment\n y: i32 }");
        assert!(result.is_ok());
    }

    // ===== Edge cases =====

    #[test]
    fn test_empty_struct_braces() {
        let result = parse("struct Empty { }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_single_field_struct() {
        let expr = parse("struct Single { value: i32 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Struct { fields, .. } = &exprs[0].kind {
                assert_eq!(fields.len(), 1);
            }
        }
    }

    #[test]
    fn test_trailing_comma() {
        let result = parse("struct Point { x: i32, y: i32, }");
        assert!(result.is_ok(), "Trailing comma should be allowed");
    }

    #[test]
    fn test_tuple_struct_trailing_comma() {
        let result = parse("struct Rgb(u8, u8, u8,)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_tuple_struct_empty() {
        let result = parse("struct Unit()");
        assert!(result.is_ok());
    }

    // ===== determine_self_type tests =====

    #[test]
    fn test_method_with_self() {
        let result = parse("struct Foo { fun bar(self) { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_method_with_ref_self() {
        let result = parse("struct Foo { fun bar(&self) { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_method_with_mut_ref_self() {
        let result = parse("struct Foo { fun bar(&mut self) { } }");
        assert!(result.is_ok());
    }

    // ===== Complex struct tests =====

    #[test]
    fn test_complex_struct() {
        let code = r#"
struct User {
    pub id: i64,
    pub(crate) name: String,
    private password_hash: String,
    mut login_count: i32 = 0,

    fun new(name: String) -> User {
        User { id: 0, name: name, password_hash: "", login_count: 0 }
    }

    pub fun get_name(&self) -> String {
        self.name
    }
}
"#;
        let result = parse(code);
        assert!(result.is_ok(), "Complex struct should parse");
    }

    #[test]
    fn test_nested_generic_struct() {
        let result = parse("struct Tree<T> { value: T, children: Vec<Tree<T>> }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_with_fn_keyword() {
        let result = parse("struct Foo { fn bar(&self) { } }");
        assert!(result.is_ok(), "'fn' should work like 'fun'");
    }

    // ============================================================
    // Additional EXTREME TDD tests for structs
    // ============================================================

    // ===== ExprKind verification =====

    #[test]
    fn test_struct_produces_struct_exprkind() {
        let expr = parse("struct Foo { }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Struct { .. }));
        }
    }

    #[test]
    fn test_tuple_struct_produces_tuple_struct_exprkind() {
        let expr = parse("struct Pair(i32, i32)").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::TupleStruct { .. }));
        }
    }

    // ===== Basic struct variations =====

    #[test]
    fn test_struct_single_char_name() {
        let result = parse("struct A { }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_long_name() {
        let result = parse("struct VeryLongStructNameForTesting { }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_name_with_underscore() {
        let result = parse("struct my_data_struct { value: i32 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_name_uppercase() {
        let result = parse("struct CONSTANTS { }");
        assert!(result.is_ok());
    }

    // ===== Field variations =====

    #[test]
    fn test_struct_many_fields() {
        let result = parse("struct Big { a: i32, b: i32, c: i32, d: i32, e: i32 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_field_with_underscore() {
        let result = parse("struct Data { my_field: i32 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_field_single_char() {
        let result = parse("struct Data { x: i32 }");
        assert!(result.is_ok());
    }

    // ===== Type variations =====

    #[test]
    fn test_struct_field_i8() {
        let result = parse("struct Byte { value: i8 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_field_i16() {
        let result = parse("struct Short { value: i16 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_field_i64() {
        let result = parse("struct Long { value: i64 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_field_f32() {
        let result = parse("struct Float { value: f32 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_field_f64() {
        let result = parse("struct Double { value: f64 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_field_bool() {
        let result = parse("struct Flag { enabled: bool }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_field_char() {
        let result = parse("struct Letter { ch: char }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_field_string() {
        let result = parse("struct Name { value: String }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_field_str_ref() {
        let result = parse("struct StrView { data: &str }");
        assert!(result.is_ok());
    }

    // ===== Generic type variations =====

    #[test]
    fn test_struct_generic_one_param() {
        let result = parse("struct Box<T> { value: T }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_generic_two_params() {
        let result = parse("struct Map<K, V> { key: K, value: V }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_generic_three_params() {
        let result = parse("struct Triple<A, B, C> { a: A, b: B, c: C }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_tuple_struct_generic() {
        let result = parse("struct Wrapper<T>(T)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_tuple_struct_generic_two() {
        let result = parse("struct Pair<A, B>(A, B)");
        assert!(result.is_ok());
    }

    // ===== Tuple struct variations =====

    #[test]
    fn test_tuple_struct_single() {
        let result = parse("struct Single(i32)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_tuple_struct_two() {
        let result = parse("struct Two(i32, String)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_tuple_struct_three() {
        let result = parse("struct Rgb(u8, u8, u8)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_tuple_struct_many() {
        let result = parse("struct Many(i32, i32, i32, i32, i32)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_tuple_struct_mixed_types() {
        let result = parse("struct Mixed(i32, String, bool, f64)");
        assert!(result.is_ok());
    }

    // ===== Visibility variations =====

    #[test]
    fn test_struct_all_pub_fields() {
        let result = parse("struct Public { pub a: i32, pub b: i32 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_all_private_fields() {
        let result = parse("struct Secret { private a: i32, private b: i32 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_pub_crate_multiple() {
        let result = parse("struct Internal { pub(crate) a: i32, pub(crate) b: i32 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_pub_super_multiple() {
        let result = parse("struct Parent { pub(super) a: i32, pub(super) b: i32 }");
        assert!(result.is_ok());
    }

    // ===== Mutability variations =====

    #[test]
    fn test_struct_all_mut_fields() {
        let result = parse("struct Mutable { mut a: i32, mut b: i32 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_pub_mut_multiple() {
        let result = parse("struct PubMut { pub mut a: i32, pub mut b: i32 }");
        assert!(result.is_ok());
    }

    // ===== Default value variations =====

    #[test]
    fn test_struct_default_int() {
        let result = parse("struct Config { value: i32 = 42 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_default_float() {
        let result = parse("struct Math { pi: f64 = 3.14159 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_default_bool_true() {
        let result = parse("struct Flags { enabled: bool = true }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_default_bool_false() {
        let result = parse("struct Flags { disabled: bool = false }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_default_negative() {
        let result = parse("struct Range { min: i32 = -100 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_mixed_defaults() {
        let result = parse("struct Config { a: i32 = 1, b: i32, c: i32 = 3 }");
        assert!(result.is_ok());
    }

    // ===== Method variations =====

    #[test]
    fn test_struct_method_no_params() {
        let result = parse("struct Foo { fun bar() { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_method_with_return() {
        let result = parse("struct Foo { fun get() -> i32 { 42 } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_method_owned_self() {
        let result = parse("struct Foo { fun consume(self) { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_method_many_params() {
        let result = parse("struct Math { fun add(a: i32, b: i32, c: i32) -> i32 { a + b + c } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_pub_fn_method() {
        let result = parse("struct Foo { pub fn bar(&self) { } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_multiple_pub_methods() {
        let result = parse("struct Foo { pub fun a(&self) { } pub fun b(&self) { } }");
        assert!(result.is_ok());
    }

    // ===== Collection field types =====

    #[test]
    fn test_struct_vec_field() {
        let result = parse("struct List { items: Vec<i32> }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_option_field() {
        let result = parse("struct Maybe { value: Option<String> }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_hashmap_field() {
        let result = parse("struct Cache { data: HashMap<String, i32> }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_nested_vec() {
        let result = parse("struct Matrix { rows: Vec<Vec<i32>> }");
        assert!(result.is_ok());
    }

    // ===== Multiple structs =====

    #[test]
    fn test_two_structs() {
        let result = parse("struct A { } struct B { }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_three_structs() {
        let result = parse("struct A { } struct B { } struct C { }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_referencing_another() {
        let result = parse("struct Inner { value: i32 } struct Outer { inner: Inner }");
        assert!(result.is_ok());
    }

    // ===== Complex scenarios =====

    #[test]
    fn test_struct_with_all_features() {
        let code = "struct Complex<T> { pub value: T, pub mut count: i32 = 0, fun new(v: T) -> Complex<T> { Complex { value: v, count: 0 } } }";
        let result = parse(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_with_self_type_field() {
        let result = parse("struct Node { value: i32, next: Option<Node> }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_deeply_nested_generic() {
        let result = parse("struct Deep { data: Option<Vec<HashMap<String, i32>>> }");
        assert!(result.is_ok());
    }
}
