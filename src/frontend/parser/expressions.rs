//! Basic expression parsing - minimal version with only used functions
use super::{
    bail, parse_expr_recursive, Expr, ExprKind, Literal, ParserState, Pattern, Result, Span, Token,
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
        // Literals (Issue #168: Added HexInteger for hex literal support)
        Token::Integer(_)
        | Token::HexInteger(_)
        | Token::Float(_)
        | Token::String(_)
        | Token::RawString(_)
        | Token::FString(_)
        | Token::Char(_)
        | Token::Byte(_)
        | Token::Bool(_)
        | Token::Atom(_)
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
        | Token::Spawn => {
            expressions_helpers::unary_operators::parse_unary_prefix(state, token, span)
        }

        // Range operators (prefix for open-start ranges like ..5)
        Token::DotDot | Token::DotDotEqual => parse_prefix_range(state, token, span),

        // Identifiers and special keywords
        Token::Identifier(_)
        | Token::Underscore
        | Token::Self_
        | Token::Super
        | Token::Default
        | Token::Result => parse_identifier_prefix(state, token, span),

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
        | Token::Lifetime(_)
        | Token::Label(_) => parse_control_prefix(state, token, span),

        // Data structures and definitions
        Token::Struct
        | Token::Class
        | Token::Trait
        | Token::Interface
        | Token::Impl
        | Token::Type
        | Token::DataFrame
        | Token::Actor
        | Token::Effect
        | Token::Handle => parse_structure_prefix(state, token, span),

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
        | Token::Lazy
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
        // Basic literals - delegated to literals module (Issue #168: Added HexInteger)
        Token::Integer(_)
        | Token::HexInteger(_)
        | Token::Float(_)
        | Token::String(_)
        | Token::RawString(_)
        | Token::FString(_)
        | Token::Char(_)
        | Token::Byte(_)
        | Token::Bool(_)
        | Token::Atom(_) => expressions_helpers::literals::parse_literal_token(state, &token, span),

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
        // PARSER-081: @label syntax for labeled loops
        // BUG-033: Distinguish between @label: (loop label) and @decorator (attribute)
        // Check if next token (after Label) is Colon to decide
        Token::Label(label_name) => {
            state.tokens.advance(); // consume the Label token
                                    // Peek at next token to determine: loop label vs decorator
            if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
                // @label: loop_keyword - this is a labeled loop
                parse_loop_label(state, label_name)
            } else {
                // @decorator or @decorator(...) - this is a decorator/attribute
                parse_label_as_decorator(state, label_name)
            }
        }
        _ => unreachable!(),
    }
}

/// BUG-033: Parse a Label token as a decorator when not followed by Colon
///
/// The lexer emits @identifier as `Token::Label("@identifier")`. When not followed
/// by a Colon (indicating a loop label), we treat it as a decorator.
fn parse_label_as_decorator(state: &mut ParserState, label_name: String) -> Result<Expr> {
    use crate::frontend::ast::{Attribute, Decorator};

    // Get span for the first decorator
    let first_span = state.tokens.peek().map_or(Span::new(0, 0), |(_, s)| *s);

    // Extract the decorator name (strip the '@' prefix)
    let name = label_name
        .strip_prefix('@')
        .unwrap_or(&label_name)
        .to_string();

    // Parse optional arguments: @decorator("arg1", "arg2")
    let args = if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        parse_decorator_args_inline(state)?
    } else {
        Vec::new()
    };

    let mut attributes = vec![Attribute {
        name,
        args,
        span: first_span,
    }];
    let mut decorators = vec![];

    // Parse any additional decorators (consecutive @decorator patterns)
    while let Some((Token::Label(next_label), next_span)) = state.tokens.peek() {
        let next_label = next_label.clone();
        let next_span = *next_span;
        state.tokens.advance();
        // Check if this label is also a decorator (not followed by Colon)
        if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
            // This is a labeled loop - put back context and bail
            bail!("Unexpected labeled loop after decorator");
        }
        let next_name = next_label
            .strip_prefix('@')
            .unwrap_or(&next_label)
            .to_string();
        let next_args = if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
            parse_decorator_args_inline(state)?
        } else {
            Vec::new()
        };
        attributes.push(Attribute {
            name: next_name.clone(),
            args: next_args.clone(),
            span: next_span,
        });
        decorators.push(Decorator {
            name: next_name,
            args: next_args,
        });
    }

    // Also handle Token::At decorators that may follow
    while let Some((Token::At, at_span)) = state.tokens.peek() {
        let at_span = *at_span;
        state.tokens.advance(); // consume @
        let dec_name = match state.tokens.peek() {
            Some((Token::Identifier(n), _)) => {
                let name = n.clone();
                state.tokens.advance();
                name
            }
            _ => bail!("Expected identifier after '@'"),
        };
        let dec_args = if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
            parse_decorator_args_inline(state)?
        } else {
            Vec::new()
        };
        attributes.push(Attribute {
            name: dec_name.clone(),
            args: dec_args.clone(),
            span: at_span,
        });
        decorators.push(Decorator {
            name: dec_name,
            args: dec_args,
        });
    }

    // Now parse the decorated item (function, class, etc.)
    let mut expr = parse_prefix(state)?;

    // Apply attributes to the expression
    expr.attributes.extend(attributes);

    // For classes, also set the decorators field
    if let ExprKind::Class {
        decorators: class_decorators,
        ..
    } = &mut expr.kind
    {
        // Convert attributes to decorators for class
        let first_dec = Decorator {
            name: label_name
                .strip_prefix('@')
                .unwrap_or(&label_name)
                .to_string(),
            args: if let Some(attr) = expr.attributes.first() {
                attr.args.clone()
            } else {
                Vec::new()
            },
        };
        let mut all_decorators = vec![first_dec];
        all_decorators.extend(decorators);
        *class_decorators = all_decorators;
    }

    Ok(expr)
}

/// Parse decorator arguments inline: ("arg1", "arg2", ...)
fn parse_decorator_args_inline(state: &mut ParserState) -> Result<Vec<String>> {
    state.tokens.advance(); // consume (
    let mut args = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        match state.tokens.peek() {
            Some((Token::String(s), _)) => {
                args.push(s.clone());
                state.tokens.advance();
            }
            Some((Token::Identifier(id), _)) => {
                args.push(id.clone());
                state.tokens.advance();
            }
            _ => bail!("Expected string or identifier in decorator arguments"),
        }
        // Handle comma separator
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }

    state.tokens.expect(&Token::RightParen)?;
    Ok(args)
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
        Token::DataFrame | Token::Actor | Token::Effect | Token::Handle => {
            parse_special_definition_token(state, token, span)
        }
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
fn parse_special_definition_token(
    state: &mut ParserState,
    token: Token,
    span: Span,
) -> Result<Expr> {
    match token {
        // DataFrame literal (df![...]) or identifier (df) - delegated to dataframes module
        Token::DataFrame => expressions_helpers::dataframes::parse_dataframe_token(state, span),
        Token::Actor => parse_actor_definition(state),
        Token::Effect => parse_effect_definition(state),
        Token::Handle => parse_handler_expression(state),
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
        Token::Abstract => {
            expressions_helpers::visibility_modifiers::parse_abstract_token(state, span)
        }
        Token::Unsafe => expressions_helpers::visibility_modifiers::parse_unsafe_token(state, span),
        Token::Break => expressions_helpers::control_flow::parse_break_token(state, span),
        Token::Continue => expressions_helpers::control_flow::parse_continue_token(state, span),
        Token::Return => expressions_helpers::control_flow::parse_return_token(state, span),
        Token::Throw => expressions_helpers::control_flow::parse_throw_token(state, span),
        Token::Export => parse_export_token(state),
        Token::Async => parse_async_token(state),
        Token::Lazy => parse_lazy_token(state),
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

/// SPEC-001-I: Effect definition delegates to effects module
fn parse_effect_definition(state: &mut ParserState) -> Result<Expr> {
    super::effects::parse_effect(state)
}

/// SPEC-001-J: Effect handler expression delegates to effects module
fn parse_handler_expression(state: &mut ParserState) -> Result<Expr> {
    super::effects::parse_handler(state)
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

/// Parse lazy token - defers computation until value is accessed
///
/// Syntax: `lazy expr` where expr is evaluated only when accessed
/// Example: `let x = lazy expensive_computation()`
fn parse_lazy_token(state: &mut ParserState) -> Result<Expr> {
    state.tokens.advance(); // consume 'lazy'

    // Parse the expression to be lazily evaluated
    let expr = parse_expr_recursive(state)?;
    let start_span = expr.span;

    Ok(Expr::new(
        ExprKind::Lazy {
            expr: Box::new(expr),
        },
        start_span,
    ))
}

// Increment and decrement operator parsing moved to expressions_helpers/increment_decrement.rs module
fn parse_increment_token(state: &mut ParserState, span: Span) -> Result<Expr> {
    expressions_helpers::increment_decrement::parse_increment_token(state, span)
}

fn parse_decrement_token(state: &mut ParserState, span: Span) -> Result<Expr> {
    expressions_helpers::increment_decrement::parse_decrement_token(state, span)
}

/// Parse prefix range operators (..5, ..=5) - PARSER-084
/// Handles open-start ranges where there's no left-hand side expression
fn parse_prefix_range(state: &mut ParserState, token: Token, _span: Span) -> Result<Expr> {
    let inclusive = matches!(token, Token::DotDotEqual);
    state.tokens.advance(); // consume .. or ..=

    // PARSER-084: Check if this is a full open range (..) with no end
    let end = match state.tokens.peek() {
        Some((
            Token::RightBracket
            | Token::Semicolon
            | Token::Comma
            | Token::RightParen
            | Token::RightBrace,
            _,
        )) => {
            // Full open range: .. with no start or end
            Expr::new(ExprKind::Literal(Literal::Unit), Span { start: 0, end: 0 })
        }
        _ => {
            // Open-start range: ..5 - parse the end expression
            super::parse_expr_with_precedence_recursive(state, 6)? // precedence 6 (higher than range infix)
        }
    };

    Ok(Expr {
        kind: ExprKind::Range {
            start: Box::new(Expr::new(
                ExprKind::Literal(Literal::Unit),
                Span { start: 0, end: 0 },
            )),
            end: Box::new(end),
            inclusive,
        },
        span: Span { start: 0, end: 0 },
        attributes: Vec::new(),
        leading_comments: Vec::new(),
        trailing_comment: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    // Helper to parse expressions
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

    // ===== parse_prefix tests =====

    #[test]
    fn test_parse_prefix_integer_literal() {
        let expr = parse("42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::Integer(42, _))
            ));
        }
    }

    #[test]
    fn test_parse_prefix_float_literal() {
        let expr = parse("3.14").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Literal(Literal::Float(f)) = &exprs[0].kind {
                assert!((f - 3.14).abs() < 0.001);
            }
        }
    }

    #[test]
    fn test_parse_prefix_string_literal() {
        let expr = parse("\"hello\"").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::String(s)) if s == "hello"
            ));
        }
    }

    #[test]
    fn test_parse_prefix_bool_true() {
        let expr = parse("true").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::Bool(true))
            ));
        }
    }

    #[test]
    fn test_parse_prefix_bool_false() {
        let expr = parse("false").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::Bool(false))
            ));
        }
    }

    #[test]
    fn test_parse_prefix_null() {
        let expr = parse("null").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Literal(Literal::Unit)));
        }
    }

    #[test]
    fn test_parse_prefix_identifier() {
        let expr = parse("foo").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Identifier(name) if name == "foo"
            ));
        }
    }

    #[test]
    fn test_parse_prefix_underscore() {
        let expr = parse("_").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Identifier(name) if name == "_"
            ));
        }
    }

    #[test]
    fn test_parse_prefix_self() {
        let expr = parse("self").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Identifier(name) if name == "self"
            ));
        }
    }

    // ===== Unary operator tests =====

    #[test]
    fn test_parse_unary_minus() {
        let expr = parse("-42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Unary { .. }));
        }
    }

    #[test]
    fn test_parse_unary_bang() {
        let expr = parse("!true").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Unary { .. }));
        }
    }

    #[test]
    fn test_parse_unary_star_deref() {
        let expr = parse("*ptr").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Unary { .. }));
        }
    }

    #[test]
    fn test_parse_unary_ampersand_ref() {
        let expr = parse("&x").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Unary { .. }));
        }
    }

    // ===== Control flow tests =====

    #[test]
    fn test_parse_if_expression() {
        let expr = parse("if true { 1 } else { 2 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::If { .. }));
        }
    }

    #[test]
    fn test_parse_if_without_else() {
        let expr = parse("if x { 1 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::If { .. }));
        }
    }

    #[test]
    fn test_parse_while_loop() {
        let expr = parse("while true { x }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::While { .. }));
        }
    }

    #[test]
    fn test_parse_for_loop() {
        let expr = parse("for i in 0..10 { i }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::For { .. }));
        }
    }

    #[test]
    fn test_parse_loop() {
        let expr = parse("loop { break }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Loop { .. }));
        }
    }

    #[test]
    fn test_parse_match_expression() {
        let expr = parse("match x { 1 => a, _ => b }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Match { .. }));
        }
    }

    // ===== Variable declaration tests =====

    #[test]
    fn test_parse_let_statement() {
        let expr = parse("let x = 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Let { .. }));
        }
    }

    #[test]
    fn test_parse_let_mut_statement() {
        let expr = parse("let mut x = 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Let { is_mutable, .. } = &exprs[0].kind {
                assert!(is_mutable);
            } else {
                panic!("Expected Let expression");
            }
        }
    }

    #[test]
    fn test_parse_let_with_type() {
        let expr = parse("let x: i32 = 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Let { type_annotation, .. } = &exprs[0].kind {
                assert!(type_annotation.is_some());
            } else {
                panic!("Expected Let expression");
            }
        }
    }

    #[test]
    fn test_parse_var_statement() {
        let expr = parse("var x = 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // var is syntactic sugar for let mut
            assert!(matches!(&exprs[0].kind, ExprKind::Let { .. }));
        }
    }

    // ===== Function tests =====

    #[test]
    fn test_parse_function_definition() {
        let expr = parse("fun foo() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Function { .. }));
        }
    }

    #[test]
    fn test_parse_function_with_params() {
        let expr = parse("fun add(a, b) { a + b }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Function { params, .. } = &exprs[0].kind {
                assert_eq!(params.len(), 2);
            } else {
                panic!("Expected Function expression");
            }
        }
    }

    #[test]
    fn test_parse_function_with_return_type() {
        let expr = parse("fun foo() -> i32 { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Function { return_type, .. } = &exprs[0].kind {
                assert!(return_type.is_some());
            } else {
                panic!("Expected Function expression");
            }
        }
    }

    // ===== Lambda tests =====

    #[test]
    fn test_parse_lambda_pipe() {
        let expr = parse("|x| x + 1").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Lambda { .. }));
        }
    }

    #[test]
    fn test_parse_lambda_no_params() {
        let expr = parse("|| 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Lambda { .. }));
        }
    }

    #[test]
    fn test_parse_lambda_backslash() {
        let expr = parse("\\x -> x + 1").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Lambda { .. }));
        }
    }

    // ===== Collection tests =====

    #[test]
    fn test_parse_array_literal() {
        let expr = parse("[1, 2, 3]").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::List(_)));
        }
    }

    #[test]
    fn test_parse_empty_array() {
        let expr = parse("[]").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::List(elements) = &exprs[0].kind {
                assert!(elements.is_empty());
            } else {
                panic!("Expected List expression");
            }
        }
    }

    #[test]
    fn test_parse_tuple() {
        let expr = parse("(1, 2, 3)").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Tuple(_)));
        }
    }

    #[test]
    fn test_parse_unit() {
        let expr = parse("()").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::Unit)
            ));
        }
    }

    #[test]
    fn test_parse_grouped_expression() {
        let expr = parse("(42)").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // Grouped expression should unwrap to the inner expression
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::Integer(42, _))
            ));
        }
    }

    // ===== Struct tests =====

    #[test]
    fn test_parse_struct_definition() {
        let expr = parse("struct Point { x: i32, y: i32 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Struct { .. }));
        }
    }

    #[test]
    fn test_parse_tuple_struct() {
        let expr = parse("struct Pair(i32, i32)").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Struct { .. }));
        }
    }

    #[test]
    fn test_parse_unit_struct() {
        let expr = parse("struct Empty").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Struct { .. }));
        }
    }

    // ===== Enum tests =====

    #[test]
    fn test_parse_enum_definition() {
        let expr = parse("enum Color { Red, Green, Blue }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Enum { .. }));
        }
    }

    // ===== Trait and impl tests =====

    #[test]
    fn test_parse_trait_definition() {
        let expr = parse("trait Foo { fun bar() }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Trait { .. }));
        }
    }

    #[test]
    fn test_parse_impl_block() {
        let expr = parse("impl Foo { fun bar() { 42 } }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Impl { .. }));
        }
    }

    // ===== Import tests =====

    #[test]
    fn test_parse_use_statement() {
        let expr = parse("use std::io").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // 'use' statements are parsed as Import
            assert!(matches!(&exprs[0].kind, ExprKind::Import { .. }));
        }
    }

    #[test]
    fn test_parse_import_statement() {
        let expr = parse("import foo").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Import { .. }));
        }
    }

    // ===== Range tests =====

    #[test]
    fn test_parse_prefix_range_exclusive() {
        let expr = parse("..5").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Range { inclusive, .. } = &exprs[0].kind {
                assert!(!inclusive);
            } else {
                panic!("Expected Range expression");
            }
        }
    }

    #[test]
    fn test_parse_prefix_range_inclusive() {
        let expr = parse("..=5").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Range { inclusive, .. } = &exprs[0].kind {
                assert!(inclusive);
            } else {
                panic!("Expected Range expression");
            }
        }
    }

    // ===== Control statement tests =====

    #[test]
    fn test_parse_break() {
        let expr = parse("break").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Break { .. }));
        }
    }

    #[test]
    fn test_parse_continue() {
        let expr = parse("continue").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Continue { .. }));
        }
    }

    #[test]
    fn test_parse_return() {
        let expr = parse("return 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Return { .. }));
        }
    }

    #[test]
    fn test_parse_return_empty() {
        let expr = parse("return").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Return { .. }));
        }
    }

    // ===== Async tests =====

    #[test]
    fn test_parse_async_function() {
        let expr = parse("async fun foo() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Function { is_async, .. } = &exprs[0].kind {
                assert!(is_async);
            } else {
                panic!("Expected async Function expression");
            }
        }
    }

    #[test]
    fn test_parse_async_block() {
        let expr = parse("async { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::AsyncBlock { .. }));
        }
    }

    // ===== Lazy tests =====

    #[test]
    fn test_parse_lazy_expression() {
        let expr = parse("lazy 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Lazy { .. }));
        }
    }

    // ===== Pub and visibility tests =====

    #[test]
    fn test_parse_pub_function() {
        let expr = parse("pub fun foo() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Function { is_pub, .. } = &exprs[0].kind {
                assert!(is_pub);
            } else {
                panic!("Expected pub Function expression");
            }
        }
    }

    #[test]
    fn test_parse_pub_struct() {
        let expr = parse("pub struct Foo { x: i32 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Struct { is_pub, .. } = &exprs[0].kind {
                assert!(is_pub);
            } else {
                panic!("Expected pub Struct expression");
            }
        }
    }

    // ===== Type alias tests =====

    #[test]
    fn test_parse_type_alias() {
        let expr = parse("type Num = i32").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::TypeAlias { .. }));
        }
    }

    // ===== Block tests =====

    #[test]
    fn test_parse_block_expression() {
        let expr = parse("{ 1; 2; 3 }").unwrap();
        // The entire program is wrapped in a block, so the outer is Block
        // The inner { 1; 2; 3 } is also a Block
        assert!(matches!(&expr.kind, ExprKind::Block(_)));
    }

    // ===== Try-catch tests =====

    #[test]
    fn test_parse_try_catch() {
        let expr = parse("try { x } catch e { e }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::TryCatch { .. }));
        }
    }

    // ===== Module tests =====

    #[test]
    fn test_parse_module_declaration() {
        let expr = parse("mod foo { }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Module { .. }));
        }
    }

    // ===== Character and byte literal tests =====

    #[test]
    fn test_parse_char_literal() {
        let expr = parse("'a'").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::Char('a'))
            ));
        }
    }

    #[test]
    fn test_parse_byte_literal() {
        let expr = parse("b'x'").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::Byte(_))
            ));
        }
    }

    // ===== Hex literal tests (Issue #168) =====

    #[test]
    fn test_parse_hex_integer() {
        let expr = parse("0xFF").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::Integer(255, _))
            ));
        }
    }

    // ===== Atom literal tests =====

    #[test]
    fn test_parse_atom_literal() {
        let expr = parse(":ok").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::Atom(name)) if name == "ok"
            ));
        }
    }

    // ===== Decorator tests (BUG-033) =====

    #[test]
    fn test_parse_decorator_on_function() {
        let expr = parse("@inline fun foo() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Function { .. }));
            assert!(!exprs[0].attributes.is_empty());
        }
    }

    #[test]
    fn test_parse_decorator_with_args() {
        let expr = parse("@test(\"example\") fun foo() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Function { .. }));
        }
    }

    // ===== parse_let_mutability tests =====

    #[test]
    fn test_let_mutability_mut() {
        let expr = parse("let mut x = 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Let { is_mutable, .. } = &exprs[0].kind {
                assert!(is_mutable);
            }
        }
    }

    #[test]
    fn test_let_mutability_immut() {
        let expr = parse("let x = 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Let { is_mutable, .. } = &exprs[0].kind {
                assert!(!is_mutable);
            }
        }
    }

    // ===== Default and Result identifier tests =====

    #[test]
    fn test_parse_default_identifier() {
        let expr = parse("default").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Identifier(name) if name == "default"
            ));
        }
    }

    #[test]
    fn test_parse_result_identifier() {
        let expr = parse("Result").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Identifier(name) if name == "Result"
            ));
        }
    }

    // ===== Ok/Err constructor tests =====

    #[test]
    fn test_parse_ok_constructor() {
        let expr = parse("Ok(42)").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Call { .. }
            ));
        }
    }

    #[test]
    fn test_parse_err_constructor() {
        let expr = parse("Err(\"error\")").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Call { .. }
            ));
        }
    }

    // ===== Some/None tests =====

    #[test]
    fn test_parse_some_constructor() {
        let expr = parse("Some(42)").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Call { .. }
            ));
        }
    }

    #[test]
    fn test_parse_none_literal() {
        let expr = parse("None").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Identifier(name) if name == "None"
            ));
        }
    }

    // ===== Super identifier test =====

    #[test]
    fn test_parse_super_identifier() {
        let expr = parse("super").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Identifier(name) if name == "super"
            ));
        }
    }

    // ===== Const declaration tests =====

    #[test]
    fn test_parse_const_declaration() {
        let expr = parse("const X: i32 = 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // const X is parsed as immutable Let with "const" attribute
            if let ExprKind::Let { is_mutable, .. } = &exprs[0].kind {
                assert!(!is_mutable);
            } else {
                panic!("Expected Let expression for const");
            }
        }
    }

    // ===== Throw tests =====

    #[test]
    fn test_parse_throw() {
        let expr = parse("throw err").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Throw { .. }));
        }
    }

    // ===== Interface as trait tests =====

    #[test]
    fn test_parse_interface_as_trait() {
        let expr = parse("interface Foo { fun bar() }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // Interface is parsed as Trait
            assert!(matches!(&exprs[0].kind, ExprKind::Trait { .. }));
        }
    }

    // ===== Class tests =====

    #[test]
    fn test_parse_class_definition() {
        let expr = parse("class Point { x: i32, y: i32 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Class { .. }));
        }
    }

    // ===== Raw string tests =====

    #[test]
    fn test_parse_raw_string() {
        let expr = parse("r\"raw\\nstring\"").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // Raw strings are parsed as regular strings
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::String(_))
            ));
        }
    }

    // ===== F-string tests =====

    #[test]
    fn test_parse_fstring() {
        let expr = parse("f\"value: {x}\"").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // F-strings are parsed as StringInterpolation
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::StringInterpolation { .. }
            ));
        }
    }

    // ===== dispatch_prefix_token comprehensive tests =====

    #[test]
    fn test_dispatch_unexpected_token() {
        // Test that unexpected tokens result in error
        let result = parse("@@@");
        assert!(result.is_err());
    }

    // ===== parse_decorator_args_inline tests =====

    #[test]
    fn test_decorator_multiple_args() {
        let expr = parse("@test(\"a\", \"b\") fun foo() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let Some(attr) = exprs[0].attributes.first() {
                assert_eq!(attr.args.len(), 2);
            }
        }
    }

    #[test]
    fn test_decorator_identifier_args() {
        let expr = parse("@test(arg1, arg2) fun foo() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Function { .. }));
        }
    }

    // ===== More thorough edge case tests =====

    #[test]
    fn test_parse_nested_if() {
        let expr = parse("if a { if b { 1 } else { 2 } } else { 3 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::If { .. }));
        }
    }

    #[test]
    fn test_parse_chained_comparison() {
        // This tests binary operators as well
        let expr = parse("a < b && b < c").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Binary { .. }));
        }
    }

    #[test]
    fn test_parse_multiline_lambda() {
        let expr = parse("|x| { let y = x + 1; y * 2 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Lambda { .. }));
        }
    }

    #[test]
    fn test_parse_generic_function() {
        let expr = parse("fun identity<T>(x: T) -> T { x }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Function { type_params, .. } = &exprs[0].kind {
                assert!(!type_params.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_generic_struct() {
        let expr = parse("struct Box<T> { value: T }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Struct { type_params, .. } = &exprs[0].kind {
                assert!(!type_params.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_impl_trait_for_type() {
        let expr = parse("impl Foo for Bar { fun baz() { 42 } }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Impl { .. }));
        }
    }

    #[test]
    fn test_parse_for_with_pattern() {
        let expr = parse("for (a, b) in pairs { a + b }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::For { .. }));
        }
    }

    #[test]
    fn test_parse_break_with_value() {
        let expr = parse("break 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Break { value, .. } = &exprs[0].kind {
                assert!(value.is_some());
            }
        }
    }

    #[test]
    fn test_parse_continue_with_label() {
        let expr = parse("continue 'outer").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Continue { label } = &exprs[0].kind {
                assert!(label.is_some());
            }
        }
    }

    // ===== Pattern matching tests =====

    #[test]
    fn test_parse_tuple_pattern_in_let() {
        let expr = parse("let (a, b) = pair").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // Tuple patterns use LetPattern variant
            assert!(matches!(&exprs[0].kind, ExprKind::LetPattern { .. }));
        }
    }

    #[test]
    fn test_parse_struct_pattern_in_let() {
        let expr = parse("let Point { x, y } = point").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // Struct patterns use LetPattern variant
            assert!(matches!(&exprs[0].kind, ExprKind::LetPattern { .. }));
        }
    }

    // ===== Unsafe function tests =====

    #[test]
    fn test_parse_unsafe_function() {
        let expr = parse("unsafe fun deref_raw(ptr) { ptr }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Function { .. } = &exprs[0].kind {
                // unsafe functions have "unsafe" attribute
                assert!(exprs[0].attributes.iter().any(|a| a.name == "unsafe"));
            } else {
                panic!("Expected Function expression");
            }
        }
    }
}
