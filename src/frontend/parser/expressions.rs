//! Basic expression parsing - minimal version with only used functions
use super::{
    bail, Expr, ExprKind, ParserState, Pattern,
    Result, Span, Token,
};

// Helper modules for improved maintainability (TDG Structural improvement)
// PARSER-069: Make expressions_helpers accessible within parser module for turbofish parsing
#[path = "expressions_helpers/mod.rs"]
pub(in crate::frontend::parser) mod expressions_helpers;
pub fn parse_prefix(state: &mut ParserState) -> Result<Expr> {
    let Some((token, span)) = state.tokens.peek() else {
        bail!("Unexpected end of input - expected expression");
    };
    let token = token.clone();
    let span = *span;

    dispatch_prefix_token(state, token, span)
}

fn dispatch_prefix_token(state: &mut ParserState, token: Token, span: Span) -> Result<Expr> {
    match token {
        // Literals
        Token::Integer(_)
        | Token::Float(_)
        | Token::String(_)
        | Token::RawString(_)
        | Token::FString(_)
        | Token::Char(_)
        | Token::Byte(_)
        | Token::Bool(_)
        | Token::Null
        | Token::None
        | Token::Some => parse_literal_prefix(state, token, span),

        // Unary operators
        Token::Minus
        | Token::Plus
        | Token::Bang
        | Token::Star
        | Token::Ampersand
        | Token::Power
        | Token::Await
        | Token::Tilde
        | Token::Spawn => expressions_helpers::unary_operators::parse_unary_prefix(state, token, span),

        // Identifiers and special keywords
        Token::Identifier(_) | Token::Underscore | Token::Self_ | Token::Super | Token::Default | Token::Result => {
            parse_identifier_prefix(state, token, span)
        }

        // Keywords and declarations
        Token::Fun
        | Token::Fn
        | Token::LeftBrace
        | Token::Let
        | Token::Var
        | Token::Mod
        | Token::Module
        | Token::At => parse_declaration_prefix(state, token, span),

        // Control flow and structures
        Token::If
        | Token::Match
        | Token::While
        | Token::For
        | Token::Try
        | Token::Loop
        | Token::Lifetime(_) => parse_control_prefix(state, token, span),

        // Data structures and definitions
        Token::Struct
        | Token::Class
        | Token::Trait
        | Token::Interface
        | Token::Impl
        | Token::Type
        | Token::DataFrame
        | Token::Actor => parse_structure_prefix(state, token, span),

        // Imports, modifiers, and specials
        Token::Import
        | Token::From
        | Token::Use
        | Token::Pub
        | Token::Const
        | Token::Sealed
        | Token::Final
        | Token::Abstract
        | Token::Unsafe
        | Token::Break
        | Token::Continue
        | Token::Return
        | Token::Throw
        | Token::Export
        | Token::Async
        | Token::Increment
        | Token::Decrement => parse_modifier_prefix(state, token, span),

        // Collections and constructors
        Token::Pipe
        | Token::OrOr
        | Token::Backslash
        | Token::LeftParen
        | Token::LeftBracket
        | Token::Enum
        | Token::Ok
        | Token::Err
        | Token::Option => parse_collection_prefix(state, token, span),

        _ => bail!("Unexpected token: {token:?}"),
    }
}

// All literal parsing moved to expressions_helpers/literals.rs module
fn parse_literal_prefix(state: &mut ParserState, token: Token, span: Span) -> Result<Expr> {
    match token {
        // Basic literals - delegated to literals module
        Token::Integer(_)
        | Token::Float(_)
        | Token::String(_)
        | Token::RawString(_)
        | Token::FString(_)
        | Token::Char(_)
        | Token::Byte(_)
        | Token::Bool(_) => expressions_helpers::literals::parse_literal_token(state, &token, span),

        // Special literals also in literals module
        Token::Null => expressions_helpers::literals::parse_null(state, span),
        Token::None => expressions_helpers::literals::parse_none(state, span),
        Token::Some => expressions_helpers::literals::parse_some_constructor(state, span),
        _ => unreachable!(),
    }
}

// Unary operator parsing moved to expressions_helpers/unary_operators.rs module

fn parse_identifier_prefix(state: &mut ParserState, token: Token, span: Span) -> Result<Expr> {
    match token {
        Token::Identifier(_) | Token::Underscore | Token::Self_ | Token::Super => {
            expressions_helpers::identifiers::parse_identifier_token(state, &token, span)
        }
        Token::Default => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Identifier("default".to_string()), span))
        }
        Token::Result => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Identifier("Result".to_string()), span))
        }
        _ => unreachable!(),
    }
}

fn parse_declaration_prefix(state: &mut ParserState, token: Token, _span: Span) -> Result<Expr> {
    match token {
        Token::Fun | Token::Fn | Token::LeftBrace => parse_function_block_token(state, token),
        Token::Let | Token::Var => parse_variable_declaration_token(state, token),
        Token::Mod | Token::Module => parse_module_declaration(state),
        Token::At => parse_decorator_prefix(state),
        _ => unreachable!(),
    }
}

fn parse_decorator_prefix(state: &mut ParserState) -> Result<Expr> {
    let decorators = expressions_helpers::classes::parse_decorators(state)?;
    let mut expr = parse_prefix(state)?;
    if let ExprKind::Class {
        decorators: ref mut class_decorators,
        ..
    } = &mut expr.kind
    {
        *class_decorators = decorators;
    }
    Ok(expr)
}

fn parse_control_prefix(state: &mut ParserState, token: Token, _span: Span) -> Result<Expr> {
    match token {
        Token::If | Token::Match | Token::While | Token::For | Token::Try | Token::Loop => {
            parse_control_flow_token(state, token)
        }
        Token::Lifetime(label_name) => parse_loop_label(state, label_name),
        _ => unreachable!(),
    }
}

// Loop label parsing moved to expressions_helpers/loops.rs module
fn parse_loop_label(state: &mut ParserState, label_name: String) -> Result<Expr> {
    expressions_helpers::loops::parse_loop_label(state, label_name)
}

fn parse_structure_prefix(state: &mut ParserState, token: Token, span: Span) -> Result<Expr> {
    match token {
        Token::Struct
        | Token::Class
        | Token::Trait
        | Token::Interface
        | Token::Impl
        | Token::Type => parse_data_structure_token(state, token),
        Token::DataFrame | Token::Actor => parse_special_definition_token(state, token, span),
        _ => unreachable!(),
    }
}

fn parse_modifier_prefix(state: &mut ParserState, token: Token, span: Span) -> Result<Expr> {
    match token {
        Token::Import | Token::From | Token::Use => parse_import_token(state, token),
        _ => parse_control_statement_token(state, token, span),
    }
}

fn parse_collection_prefix(state: &mut ParserState, token: Token, span: Span) -> Result<Expr> {
    match token {
        Token::Pipe | Token::OrOr | Token::Backslash => parse_lambda_token(state, token),
        Token::LeftParen => expressions_helpers::tuples::parse_parentheses_token(state, span),
        Token::LeftBracket | Token::Enum => parse_collection_enum_token(state, token),
        Token::Ok | Token::Err | Token::Result | Token::Option => {
            parse_constructor_token(state, token, span)
        }
        _ => unreachable!(),
    }
}

// Literal parsing moved to expressions_helpers/literals.rs module
// Identifier and path parsing moved to expressions_helpers/identifiers.rs module

/// Parse unary operator tokens (Minus, Bang)
/// Extracted from `parse_prefix` to reduce complexity
/// Parse parentheses tokens - either unit type (), grouped expression (expr), or tuple (a, b, c)
/// Extracted from `parse_prefix` to reduce complexity
// Tuple parsing moved to expressions_helpers/tuples.rs module

// Visibility and modifier functions moved to expressions_helpers/visibility_modifiers.rs (TDG improvement)
// - parse_pub_token, parse_const_token, parse_sealed_token, parse_final_token
// - parse_abstract_token, parse_unsafe_token + 11 helper functions
// Control flow functions moved to expressions_helpers/control_flow.rs (TDG improvement)

// Constructor token parsing moved to expressions_helpers/increment_decrement.rs module
fn parse_constructor_token(state: &mut ParserState, token: Token, span: Span) -> Result<Expr> {
    expressions_helpers::increment_decrement::parse_constructor_token(state, token, span)
}
/// Parse control flow tokens (If, Match, While, For, Try)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_control_flow_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::If => parse_if_expression(state),
        Token::Match => parse_match_expression(state),
        Token::While => parse_while_loop(state),
        Token::For => parse_for_loop(state),
        Token::Try => parse_try_catch(state),
        Token::Loop => parse_loop(state),
        _ => bail!("Expected control flow token, got: {token:?}"),
    }
}
// Try-catch-finally parsing moved to expressions_helpers/error_handling.rs module
fn parse_try_catch(state: &mut ParserState) -> Result<Expr> {
    expressions_helpers::error_handling::parse_try_catch(state)
}
// Module declaration parsing moved to expressions_helpers/modules.rs module
fn parse_module_declaration(state: &mut ParserState) -> Result<Expr> {
    expressions_helpers::modules::parse_module_declaration(state)
}
/// Parse data structure definition tokens (Struct, Trait, Impl)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_data_structure_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::Struct => parse_struct_definition(state),
        Token::Class => parse_struct_definition(state), // Class transpiles to struct
        Token::Trait => parse_trait_definition(state),
        Token::Interface => parse_trait_definition(state), // Interface is just a trait
        Token::Impl => parse_impl_block(state),
        Token::Type => parse_type_alias(state),
        _ => bail!("Expected data structure token, got: {token:?}"),
    }
}
/// Parse import/module tokens (Import, Use)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_import_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::Import => {
            // Consume the Import token first
            state.tokens.advance();
            // Check if this is JS-style import { ... } from
            if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
                super::imports::parse_js_style_import(state)
            } else {
                super::imports::parse_import_statement(state)
            }
        }
        Token::From => {
            // Consume the From token
            state.tokens.advance();
            super::imports::parse_from_import_statement(state)
        }
        Token::Use => parse_use_statement(state),
        _ => bail!("Expected import token, got: {token:?}"),
    }
}

/// Parse lambda expression tokens (Pipe, `OrOr`)\
/// Extracted from `parse_prefix` to reduce complexity
fn parse_lambda_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::Pipe => parse_lambda_expression(state),
        Token::OrOr => parse_lambda_no_params(state),
        Token::Backslash => super::functions::parse_lambda(state),
        _ => bail!("Expected lambda token, got: {token:?}"),
    }
}
/// Parse function/block tokens (Fun, Fn, `LeftBrace`)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_function_block_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::Fun | Token::Fn => super::functions::parse_function(state),
        Token::LeftBrace => super::collections::parse_block(state),
        _ => bail!("Expected function/block token, got: {token:?}"),
    }
}
/// Parse variable declaration tokens (Let, Var)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_variable_declaration_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::Let => parse_let_statement(state),
        Token::Var => parse_var_statement(state),
        _ => bail!("Expected variable declaration token, got: {token:?}"),
    }
}
/// Parse special definition tokens (`DataFrame`, Actor)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_special_definition_token(state: &mut ParserState, token: Token, span: Span) -> Result<Expr> {
    match token {
        // DataFrame literal (df![...]) or identifier (df) - delegated to dataframes module
        Token::DataFrame => expressions_helpers::dataframes::parse_dataframe_token(state, span),
        Token::Actor => parse_actor_definition(state),
        _ => bail!("Expected special definition token, got: {token:?}"),
    }
}
/// Parse control statement tokens (Pub, Break, Continue, Return)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_control_statement_token(
    state: &mut ParserState,
    token: Token,
    span: Span,
) -> Result<Expr> {
    match token {
        Token::Pub => expressions_helpers::visibility_modifiers::parse_pub_token(state, span),
        Token::Const => expressions_helpers::visibility_modifiers::parse_const_token(state, span),
        Token::Sealed => expressions_helpers::visibility_modifiers::parse_sealed_token(state, span),
        Token::Final => expressions_helpers::visibility_modifiers::parse_final_token(state, span),
        Token::Abstract => expressions_helpers::visibility_modifiers::parse_abstract_token(state, span),
        Token::Unsafe => expressions_helpers::visibility_modifiers::parse_unsafe_token(state, span),
        Token::Break => expressions_helpers::control_flow::parse_break_token(state, span),
        Token::Continue => expressions_helpers::control_flow::parse_continue_token(state, span),
        Token::Return => expressions_helpers::control_flow::parse_return_token(state, span),
        Token::Throw => expressions_helpers::control_flow::parse_throw_token(state, span),
        Token::Export => parse_export_token(state),
        Token::Async => parse_async_token(state),
        Token::Increment => parse_increment_token(state, span),
        Token::Decrement => parse_decrement_token(state, span),
        _ => bail!("Expected control statement token, got: {token:?}"),
    }
}
/// Parse collection/enum definition tokens (`LeftBracket`, Enum)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_collection_enum_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::LeftBracket => expressions_helpers::arrays::parse_list_literal(state),
        Token::Enum => parse_enum_definition(state),
        _ => bail!("Expected collection/enum token, got: {token:?}"),
    }
}
/// Parse let statement: let [mut] name [: type] = value [in body]
// Let statement parsing moved to expressions_helpers/variable_declarations.rs module
fn parse_let_statement(state: &mut ParserState) -> Result<Expr> {
    expressions_helpers::variable_declarations::parse_let_statement(state)
}
/// Parse mutability for let statement
/// Extracted from `parse_let_statement` to reduce complexity
fn parse_let_mutability(state: &mut ParserState) -> bool {
    if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
        state.tokens.advance();
        true
    } else {
        false
    }
}

// Pattern parsing moved to expressions_helpers/patterns.rs module
fn parse_let_pattern(state: &mut ParserState, is_mutable: bool) -> Result<Pattern> {
    expressions_helpers::patterns::parse_let_pattern(state, is_mutable)
}

// Pattern parsing moved to expressions_helpers/patterns.rs module
fn parse_var_pattern(state: &mut ParserState) -> Result<Pattern> {
    expressions_helpers::patterns::parse_var_pattern(state)
}

// Pattern parsing moved to expressions_helpers/patterns.rs module
pub fn parse_tuple_pattern(state: &mut ParserState) -> Result<Pattern> {
    expressions_helpers::patterns::parse_tuple_pattern(state)
}

// Pattern parsing moved to expressions_helpers/patterns.rs module
pub fn parse_struct_pattern(state: &mut ParserState) -> Result<Pattern> {
    expressions_helpers::patterns::parse_struct_pattern(state)
}

// Pattern parsing moved to expressions_helpers/patterns.rs module
pub fn parse_list_pattern(state: &mut ParserState) -> Result<Pattern> {
    expressions_helpers::patterns::parse_list_pattern(state)
}

// Pattern parsing moved to expressions_helpers/patterns.rs module
fn parse_match_pattern(state: &mut ParserState) -> Result<Pattern> {
    expressions_helpers::patterns::parse_match_pattern(state)
}

// Expression parsing moved to expressions_helpers/patterns.rs module
fn parse_if_expression(state: &mut ParserState) -> Result<Expr> {
    expressions_helpers::patterns::parse_if_expression(state)
}

fn parse_match_expression(state: &mut ParserState) -> Result<Expr> {
    expressions_helpers::patterns::parse_match_expression(state)
}

fn parse_var_statement(state: &mut ParserState) -> Result<Expr> {
    expressions_helpers::patterns::parse_var_statement(state)
}

/// Parse while loop: while condition { body }
/// Complexity: <5 (simple structure)
// While loop parsing moved to expressions_helpers/loops.rs module
fn parse_while_loop(state: &mut ParserState) -> Result<Expr> {
    expressions_helpers::loops::parse_while_loop(state)
}
// For loop parsing moved to expressions_helpers/loops.rs module
fn parse_for_loop(state: &mut ParserState) -> Result<Expr> {
    expressions_helpers::loops::parse_for_loop(state)
}

// Infinite loop parsing moved to expressions_helpers/loops.rs module
fn parse_loop(state: &mut ParserState) -> Result<Expr> {
    expressions_helpers::loops::parse_loop(state)
}

// Array/list parsing moved to expressions_helpers/arrays.rs module

// No-parameter lambda parsing moved to expressions_helpers/lambdas.rs module
fn parse_lambda_no_params(state: &mut ParserState) -> Result<Expr> {
    expressions_helpers::lambdas::parse_lambda_no_params(state)
}
// Arrow lambda parsing moved to expressions_helpers/lambdas.rs module
fn parse_lambda_from_expr(state: &mut ParserState, expr: Expr, start_span: Span) -> Result<Expr> {
    expressions_helpers::lambdas::parse_lambda_from_expr(state, expr, start_span)
}
// Pipe-delimited lambda parsing moved to expressions_helpers/lambdas.rs module
fn parse_lambda_expression(state: &mut ParserState) -> Result<Expr> {
    expressions_helpers::lambdas::parse_lambda_expression(state)
}
// Type alias and generic parameter parsing moved to expressions_helpers/type_aliases.rs module
fn parse_type_alias(state: &mut ParserState) -> Result<Expr> {
    expressions_helpers::type_aliases::parse_type_alias(state)
}

pub(super) fn parse_optional_generics(state: &mut ParserState) -> Result<Vec<String>> {
    expressions_helpers::type_aliases::parse_optional_generics(state)
}

fn parse_struct_definition(state: &mut ParserState) -> Result<Expr> {
    let (is_class, start_span) = match state.tokens.peek().map(|(t, s)| (t.clone(), *s)) {
        Some((Token::Struct, span)) => {
            state.tokens.advance();
            (false, span)
        }
        Some((Token::Class, span)) => {
            state.tokens.advance();
            (true, span)
        }
        _ => bail!("Expected 'struct' or 'class' keyword"),
    };

    let name = expressions_helpers::structs::parse_struct_name(state)?;
    let type_params = parse_optional_generics(state)?;

    if is_class {
        expressions_helpers::classes::parse_class_definition(state, name, type_params, start_span)
    } else {
        expressions_helpers::structs::parse_struct_variant(state, name, type_params, start_span)
    }
}

/// Delegate trait parsing to traits module
fn parse_trait_definition(state: &mut ParserState) -> Result<Expr> {
    expressions_helpers::traits::parse_trait_definition(state)
}

/// Delegate impl block parsing to impls module
fn parse_impl_block(state: &mut ParserState) -> Result<Expr> {
    expressions_helpers::impls::parse_impl_block(state)
}

// Use statement parsing moved to expressions_helpers/use_statements.rs module
fn parse_use_statement(state: &mut ParserState) -> Result<Expr> {
    expressions_helpers::use_statements::parse_use_statement(state)
}

// Legacy delegation for from imports (delegates to imports.rs)
pub(super) fn parse_from_import_statement(state: &mut ParserState) -> Result<Expr> {
    super::imports::parse_from_import_statement(state)
}

// Legacy delegation for use path (used by imports.rs)
pub(super) fn parse_use_path(
    state: &mut ParserState,
    start_span: crate::frontend::ast::Span,
) -> Result<Expr> {
    expressions_helpers::use_statements::parse_use_path(state, start_span)
}

// Removed old parse_import_statement - now in imports.rs

// Enum definition parsing moved to expressions_helpers/enums.rs module
fn parse_enum_definition(state: &mut ParserState) -> Result<Expr> {
    expressions_helpers::enums::parse_enum_definition(state)
}

/// Parse actor name (this appears orphaned - likely needs cleanup)
// Actor definition delegates to actors module:
fn parse_actor_definition(state: &mut ParserState) -> Result<Expr> {
    super::actors::parse_actor(state)
}
/// Parse actor name
// Re-export binary operator functions from binary_operators module
// These are used by mod.rs and collections.rs
pub use expressions_helpers::binary_operators::{get_precedence, token_to_binary_op};

/// Parse export statement - delegates to utils module
fn parse_export_token(state: &mut ParserState) -> Result<Expr> {
    super::utils::parse_export(state)
}

/// Parse async function declaration, async block, or async lambda
// Async expression parsing moved to expressions_helpers/async_expressions.rs module
fn parse_async_token(state: &mut ParserState) -> Result<Expr> {
    expressions_helpers::async_expressions::parse_async_token(state)
}

// Increment and decrement operator parsing moved to expressions_helpers/increment_decrement.rs module
fn parse_increment_token(state: &mut ParserState, span: Span) -> Result<Expr> {
    expressions_helpers::increment_decrement::parse_increment_token(state, span)
}

fn parse_decrement_token(state: &mut ParserState, span: Span) -> Result<Expr> {
    expressions_helpers::increment_decrement::parse_decrement_token(state, span)
}

