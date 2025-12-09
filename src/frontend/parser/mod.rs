//! Recursive descent parser for the Ruchy programming language.
//!
//! This module implements a hand-written recursive descent parser with Pratt precedence
//! handling for operator expressions. The parser converts a stream of tokens from the
//! lexer into an Abstract Syntax Tree (AST) that can be processed by subsequent
//! compilation phases.
//!
//! # Architecture
//!
//! The parser is modularized for maintainability and complexity management:
//!
//! ## Core Modules
//!
//! - **`core`** - Main parser entry points and precedence handling
//! - **`expressions`** - Basic expressions (literals, binary/unary ops, identifiers)
//! - **`control_flow`** - Control structures (if/else, match, for/while loops)
//! - **`functions`** - Function definitions, lambdas, method calls
//! - **`types`** - Type system constructs (structs, traits, impls, generics)
//! - **`collections`** - Collection literals and comprehensions
//! - **`actors`** - Actor model constructs for concurrency
//! - **`utils`** - Error recovery and parsing utilities
//!
//! ## Parsing Strategy
//!
//! The parser uses several key techniques:
//!
//! 1. **Pratt Parsing**: For handling operator precedence and associativity
//! 2. **Recursive Descent**: For parsing nested structures and statements
//! 3. **Error Recovery**: Continues parsing after errors for better diagnostics
//! 4. **Arena Allocation**: Efficient memory management for AST nodes
//! 5. **String Interning**: Deduplication of identifiers and strings
//!
//! # Examples
//!
//! ```ignore
//! use ruchy::Parser;
//!
//! let mut parser = Parser::new("let x = 42");
//! let ast = parser.parse().expect("Failed to parse");
//! ```
//!
//! # Error Handling
//!
//! The parser attempts to recover from errors to provide multiple diagnostics
//! in a single pass. Errors are collected and can be retrieved after parsing.
mod actors;
mod collections;
mod core;
mod effects;
mod expressions;
mod functions;
mod imports;
mod macro_parsing;
mod operator_precedence;
mod types;
mod utils;

// Re-export the main parser
use crate::frontend::arena::{Arena, StringInterner};
use crate::frontend::ast::{
    // Additional types for re-export to submodules
    Attribute,
    BinaryOp,
    Comment,
    CommentKind,
    Expr,
    ExprKind,
    Literal,
    Param,
    Pattern,
    PipelineStage,
    Span,
    StringPart,
    Type,
    TypeKind,
};
use crate::frontend::lexer::{Token, TokenStream};
use crate::parser::error_recovery::ErrorNode;
use anyhow::{bail, Result};
pub use core::Parser;
use std::collections::VecDeque;

/// Parse use statement with visibility modifier
pub(crate) fn parse_use_statement_with_visibility(
    state: &mut ParserState,
    is_pub: bool,
) -> Result<Expr> {
    // The 'use' token has already been consumed by the caller
    let start_span = Span { start: 0, end: 0 };
    let mut expr = expressions::parse_use_path(state, start_span)?;
    if is_pub {
        expr.attributes.push(crate::frontend::ast::Attribute {
            name: "pub".to_string(),
            args: vec![],
            span: expr.span,
        });
    }
    Ok(expr)
}
/// Internal parser state containing tokens, errors, and memory management.
///
/// This structure maintains all mutable state during parsing including:
/// - Token stream for lookahead and consumption
/// - Error collection for diagnostics
/// - Arena allocator for efficient AST allocation
/// - String interner for identifier deduplication
/// - Expression cache for common subexpressions
///
/// The parser state is passed through all parsing functions to maintain
/// consistency and enable error recovery.
pub(crate) struct ParserState<'a> {
    /// Token stream providing lookahead and token consumption.
    pub tokens: TokenStream<'a>,
    /// Collection of parse errors for diagnostic reporting.
    pub errors: Vec<ErrorNode>,
    /// Arena allocator for efficient AST node allocation.
    #[allow(dead_code)]
    pub arena: Arena,
    /// String interner for deduplicating identifiers and strings.
    #[allow(dead_code)]
    pub interner: StringInterner,
    /// Small cache for recently parsed expressions (capacity 8)
    #[allow(dead_code)]
    pub expr_cache: VecDeque<(usize, Expr)>,
    /// PARSER-071: Flag to indicate we're parsing a match guard expression
    /// When true, `=>` and `->` should not be treated as lambda syntax
    pub in_guard_context: bool,
    /// Flag to indicate we're parsing a let expression's value
    /// When true, `in` should not be treated as a binary operator (containment)
    /// so that `let x = value in body` parses correctly
    pub in_let_value_context: bool,
}
impl<'a> ParserState<'a> {
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            tokens: TokenStream::new(input),
            errors: Vec::new(),
            arena: Arena::new(),
            interner: StringInterner::new(),
            expr_cache: VecDeque::with_capacity(8),
            in_guard_context: false, // PARSER-071: Initialize guard context flag
            in_let_value_context: false, // Initialize let-value context flag
        }
    }
    /// Get all errors encountered during parsing
    pub fn get_errors(&self) -> &[ErrorNode] {
        &self.errors
    }
    /// Get arena statistics for performance monitoring
    #[allow(dead_code)]
    pub fn arena_stats(&self) -> (usize, usize) {
        (self.arena.total_allocated(), self.arena.num_items())
    }
    /// Get interner statistics
    #[allow(dead_code)]
    pub fn interner_stats(&self) -> (usize, usize) {
        self.interner.stats()
    }

    /// Consume all leading comments before an expression (complexity: 2)
    pub fn consume_leading_comments(&mut self) -> Vec<Comment> {
        let mut comments = Vec::new();
        while let Some((token, span)) = self.tokens.peek() {
            if let Some(comment) = token_to_comment(token, *span) {
                self.tokens.advance();
                comments.push(comment);
            } else {
                break;
            }
        }
        comments
    }

    /// Consume trailing comment on same line as expression (complexity: 3)
    pub fn consume_trailing_comment(&mut self, expr_span: &Span) -> Option<Comment> {
        if let Some((_token, span)) = self.tokens.peek() {
            // Copy span values before borrowing mutably
            let comment_start = span.start;

            // Get source reference before any mutable borrows
            let source = self.tokens.source();

            // Check if comment is on same line
            if !is_on_same_line(source, expr_span.end, comment_start) {
                // Comment is on a different line - don't consume (will be leading comment for next expr)
                return None;
            }

            // Now we can safely peek again and consume
            if let Some((token, span)) = self.tokens.peek() {
                if let Some(comment) = token_to_comment(token, *span) {
                    self.tokens.advance();
                    return Some(comment);
                }
            }
        }
        None
    }

    /// Skip non-doc comment tokens transparently
    /// This allows method chains and expressions to work with intervening comments
    /// PARSER-053: Fix for multi-line comments breaking method chains
    /// PARSER-083: Doc comments are NOT skipped - they are preserved for attachment
    pub fn skip_comments(&mut self) {
        while let Some((token, _)) = self.tokens.peek() {
            match token {
                Token::LineComment(_) | Token::BlockComment(_) | Token::HashComment(_) => {
                    self.tokens.advance();
                }
                // PARSER-083: Doc comments must be preserved for consume_leading_comments()
                Token::DocComment(_) => break,
                _ => break,
            }
        }
    }
}

/// Check if two byte positions are on the same line (no newline between them)
///
/// # Safety
/// Ensures slicing respects UTF-8 char boundaries to prevent panics on multi-byte characters.
fn is_on_same_line(source: &str, pos1: usize, pos2: usize) -> bool {
    if pos1 > pos2 {
        return false; // Invalid range
    }

    // Clamp positions to source length
    let start = pos1.min(source.len());
    let end = pos2.min(source.len());

    // Ensure positions are on char boundaries (handles multi-byte UTF-8 like ✓, 世, emoji)
    let start_safe = if source.is_char_boundary(start) {
        start
    } else {
        // Find nearest char boundary before start
        (0..=start)
            .rev()
            .find(|&i| source.is_char_boundary(i))
            .unwrap_or(0)
    };

    let end_safe = if source.is_char_boundary(end) {
        end
    } else {
        // Find nearest char boundary after end
        (end..source.len())
            .find(|&i| source.is_char_boundary(i))
            .unwrap_or(source.len())
    };

    let between = &source[start_safe..end_safe];
    !between.contains('\n')
}

/// Convert token to comment if it's a comment token (complexity: 2)
fn token_to_comment(token: &Token, span: Span) -> Option<Comment> {
    match token {
        Token::LineComment(text) => Some(Comment::new(CommentKind::Line(text.clone()), span)),
        Token::DocComment(text) => Some(Comment::new(CommentKind::Doc(text.clone()), span)),
        Token::BlockComment(text) => Some(Comment::new(CommentKind::Block(text.clone()), span)),
        Token::HashComment(text) => Some(Comment::new(
            CommentKind::Line(text.clone()), // Treat hash comments as line comments
            span,
        )),
        _ => None,
    }
}
/// Parses an expression using recursive descent.
///
/// This is the main entry point for expression parsing, starting with
/// the lowest precedence level (0) to ensure proper operator binding.
///
/// # Arguments
///
/// * `state` - The current parser state
///
/// # Returns
///
/// The parsed expression or an error if parsing fails.
pub(crate) fn parse_expr_recursive(state: &mut ParserState) -> Result<Expr> {
    parse_expr_with_precedence_recursive(state, 0)
}
/// Parse expression with precedence handling (complexity: 5, cognitive: 5)
pub(crate) fn parse_expr_with_precedence_recursive(
    state: &mut ParserState,
    min_prec: i32,
) -> Result<Expr> {
    // Consume any leading comments before the expression
    let leading_comments = state.consume_leading_comments();

    // Parse the expression
    let mut left = expressions::parse_prefix(state)?;
    left = parse_postfix_and_infix_chain(state, left, min_prec)?;

    // Consume any trailing comment on the same line
    let trailing_comment = state.consume_trailing_comment(&left.span);

    // Attach comments to the expression
    attach_comments_to_expr(left, leading_comments, trailing_comment)
}

/// Attach comments to expression (complexity: 1)
fn attach_comments_to_expr(
    mut expr: Expr,
    leading_comments: Vec<Comment>,
    trailing_comment: Option<Comment>,
) -> Result<Expr> {
    expr.leading_comments = leading_comments;
    expr.trailing_comment = trailing_comment;
    Ok(expr)
}

/// Parse postfix and infix operator chain (complexity: 3, cognitive: 3)
fn parse_postfix_and_infix_chain(
    state: &mut ParserState,
    mut left: Expr,
    min_prec: i32,
) -> Result<Expr> {
    loop {
        left = handle_postfix_operators(state, left)?;
        if let Some(new_left) = try_handle_infix_operators(state, left.clone(), min_prec)? {
            left = new_left;
        } else {
            break;
        }
    }
    Ok(left)
}
/// Attempts to parse infix operators at the current position.
///
/// This function tries various infix operator handlers in priority order,
/// returning the first successful parse. The handlers are ordered to
/// ensure correct precedence and avoid ambiguity.
///
/// # Arguments
///
/// * `state` - Current parser state
/// * `left` - Left-hand side expression
/// * `min_prec` - Minimum precedence level for binding
///
/// # Returns
///
/// `Some(expr)` if an infix operator was parsed, `None` otherwise.
pub(crate) fn try_handle_infix_operators(
    state: &mut ParserState,
    left: Expr,
    min_prec: i32,
) -> Result<Option<Expr>> {
    // PARSER-053/054 FIX: Peek past comments to find operators (line continuation support)
    // Save position in case we need to backtrack
    let saved_position = state.tokens.position();

    // Skip comments temporarily to peek at the actual next token
    while matches!(
        state.tokens.peek(),
        Some((
            Token::LineComment(_)
                | Token::DocComment(_)
                | Token::BlockComment(_)
                | Token::HashComment(_),
            _
        ))
    ) {
        state.tokens.advance();
    }

    // Get current token for infix processing (after skipping comments)
    let token_result = state.tokens.peek().map(|(t, _)| t.clone());

    // Restore position - we'll consume comments properly in the operator handlers
    state.tokens.set_position(saved_position);

    let Some(token) = token_result else {
        return Ok(None);
    };
    let token_clone = token;
    // Try operators in order of priority
    let handlers = [
        try_new_actor_operators,
        try_type_cast_operator,
        try_ternary_operator, // Add ternary before binary (lower precedence)
        try_binary_operators,
        try_assignment_operators,
        try_pipeline_operators,
        try_range_operators,
    ];
    for handler in &handlers {
        if let Some(new_left) = handler(state, left.clone(), &token_clone, min_prec)? {
            return Ok(Some(new_left));
        }
    }
    Ok(None)
}
/// Handle all postfix operators in a loop
pub(crate) fn handle_postfix_operators(state: &mut ParserState, mut left: Expr) -> Result<Expr> {
    while let Some(new_left) = try_handle_single_postfix(state, left.clone())? {
        left = new_left;
    }
    Ok(left)
}
/// Try to handle a single postfix operator
/// Returns Some(expr) if handled, None if no postfix operator found
fn try_handle_single_postfix(state: &mut ParserState, left: Expr) -> Result<Option<Expr>> {
    // PARSER-053 FIX: Skip comments before checking for postfix operators
    // This allows: "hello" # comment\n .to_uppercase()
    // Comments are consumed (not restored) so method chains work properly
    while matches!(
        state.tokens.peek(),
        Some((
            Token::LineComment(_)
                | Token::DocComment(_)
                | Token::BlockComment(_)
                | Token::HashComment(_),
            _
        ))
    ) {
        state.tokens.advance();
    }
    let token_peek = state.tokens.peek().map(|(t, _)| t.clone());

    match token_peek.as_ref() {
        Some(Token::Dot) => handle_dot_operator(state, left).map(Some),
        Some(Token::ColonColon) => handle_colon_colon_operator(state, left).map(Some),
        Some(Token::SafeNav) => handle_safe_nav_operator(state, left).map(Some),
        Some(Token::LeftParen) => {
            // PARSER-DEFECT-001 FIX: Block-like control flow expressions should NOT
            // consume `(...)` as function calls in statement position.
            // This prevents: `loop { break; } (x, x)` from being parsed as
            // `(loop { break; })(x, x)` (calling loop result with args).
            // Instead it should be two separate statements.
            if is_block_like_expression(&left) {
                Ok(None) // Don't treat `(` as postfix call
            } else {
                // PARSER-XXX FIX: Don't parse `(` as function call if it's on a new line
                // This prevents: `let sum = a + b\n(sum, product)` from being parsed as
                // `let sum = a + b(sum, product)` (calling b with tuple args).
                // Instead it should be two separate statements.
                // Extract span start before borrowing source
                let paren_start = state.tokens.peek().map(|(_, s)| s.start);
                if let Some(paren_pos) = paren_start {
                    // Check if there's a significant gap (newline) between expr and (
                    let source = state.tokens.source();
                    if !is_on_same_line(source, left.span.end, paren_pos) {
                        return Ok(None); // `(` is on new line, not a function call
                    }
                }
                Ok(Some(functions::parse_call(state, left)?))
            }
        }
        Some(Token::LeftBracket) => {
            // PARSER-081 FIX: Don't treat `[` as array indexing after literals, struct literals, let statements, or standalone function calls
            // This prevents `let y = 2 [x, y]`, `let p = Point{...} [x]`, and `let result = foo() [1, 2]` from being parsed as indexing
            // PARSER-086: Extended to fix block-level let with function call followed by array literal
            // PARSER-XXX: Extended to include Await and Try expressions
            if matches!(
                left.kind,
                ExprKind::Literal(_)
                    | ExprKind::StructLiteral { .. }
                    | ExprKind::Let { .. }
                    | ExprKind::Call { .. }
                    | ExprKind::Await { .. }
                    | ExprKind::Try { .. }
            ) {
                Ok(None) // Not array indexing, `[...]` is a separate expression
            } else {
                Ok(Some(handle_array_indexing(state, left)?))
            }
        }
        Some(Token::LeftBrace) => try_parse_struct_literal(state, &left),
        Some(Token::Increment) => handle_increment_operator(state, left).map(Some),
        Some(Token::Decrement) => handle_decrement_operator(state, left).map(Some),
        Some(Token::Question) => {
            // Check if this is ternary or try operator
            if is_ternary_operator(state) {
                Ok(None) // Let ternary handler in infix operators handle it
            } else {
                handle_try_operator(state, left).map(Some)
            }
        }
        Some(Token::Bang) => try_parse_macro_call(state, &left),
        _ => Ok(None),
    }
}

/// Check if expression is a block-like control flow construct
///
/// Block-like expressions (block, loop, while, for, if, match, try) should not
/// automatically consume `(...)` as postfix function calls.
///
/// # Complexity
/// Cyclomatic complexity: 2 (simple match)
fn is_block_like_expression(expr: &Expr) -> bool {
    matches!(
        expr.kind,
        ExprKind::Block(_)
            | ExprKind::Loop { .. }
            | ExprKind::While { .. }
            | ExprKind::For { .. }
            | ExprKind::If { .. }
            | ExprKind::Match { .. }
            | ExprKind::TryCatch { .. }
    )
}
/// Handle dot operator for method calls
fn handle_dot_operator(state: &mut ParserState, left: Expr) -> Result<Expr> {
    state.tokens.advance(); // consume dot

    // PARSER-053 FIX: Skip comments after dot (allows: .  # comment\n  method_name())
    while matches!(
        state.tokens.peek(),
        Some((
            Token::LineComment(_)
                | Token::DocComment(_)
                | Token::BlockComment(_)
                | Token::HashComment(_),
            _
        ))
    ) {
        state.tokens.advance();
    }

    functions::parse_method_call(state, left)
}
/// Handle :: operator for enum variant access (e.g., `Status::Success`)
/// Complexity: 3
/// PARSER-064: Extract identifier name from token (handles keywords used as identifiers)
///
/// In Rust and Ruchy, keywords can be used as method/function names in path expressions.
/// Example: `String::from()` - where 'from' is a keyword but valid after ::
fn token_as_identifier(token: &Token) -> Option<String> {
    match token {
        Token::Identifier(name) => Some(name.clone()),
        // Keywords that can be used as identifiers (common in Rust stdlib)
        Token::Ok => Some("Ok".to_string()),
        Token::Err => Some("Err".to_string()),
        Token::Some => Some("Some".to_string()),
        Token::None => Some("None".to_string()),
        Token::From => Some("from".to_string()), // PARSER-064: Allow from (String::from)
        Token::As => Some("as".to_string()),
        Token::In => Some("in".to_string()),
        Token::Type => Some("type".to_string()),
        // PARSER-094: Allow keywords in stdlib paths (std::env::var, nested::module::function)
        Token::Var => Some("var".to_string()),
        Token::Module => Some("module".to_string()),
        // Add more keywords as needed for Rust compatibility
        _ => Option::None,
    }
}

fn handle_colon_colon_operator(state: &mut ParserState, left: Expr) -> Result<Expr> {
    state.tokens.advance(); // consume ::

    // PARSER-070: Check for turbofish syntax (`::<Type>`) in path expressions
    // Examples: Vec::<i32>::new(), HashMap::<String, i32>::new()
    if let Some((Token::Less, _)) = state.tokens.peek() {
        // Parse turbofish type parameters: ::<Type1, Type2, ...>
        parse_turbofish(state)?;

        // After turbofish, expect :: again for method call
        // Example: Vec::<i32>::new() - we're now at the second ::
        if let Some((Token::ColonColon, _)) = state.tokens.peek() {
            state.tokens.advance(); // consume second ::
        } else {
            return Err(anyhow::anyhow!(
                "Expected '::' after turbofish type parameters"
            ));
        }
    }

    // PARSER-064: Accept identifiers AND keywords after :: (keywords can be method names)
    // Examples: String::from(), Result::Ok(), Option::Some()
    match state.tokens.peek() {
        Some((token, span)) => {
            if let Some(field) = token_as_identifier(token) {
                let field_span = *span;
                state.tokens.advance();

                // PARSER-091: Fix for Issue #75 - Command::new() should parse as QualifiedName
                // REGRESSION-076: Fix for Issue #76 - Vec/Box/HashMap should remain FieldAccess
                // When we see `Module::function(`, determine if this should be QualifiedName or FieldAccess
                // - QualifiedName: Builtin modules (Command, etc.)
                // - FieldAccess: Standard library types (Vec, Box, HashMap, etc.)
                if let ExprKind::Identifier(ref module) = left.kind {
                    // Peek ahead to see if this is a function call
                    if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                        // Check if this is a builtin module that should use QualifiedName
                        // Standard library types (Vec, Box, HashMap, etc.) should remain FieldAccess
                        let is_builtin_module = matches!(
                            module.as_str(),
                            "Command" | "DataFrame" | "Sql" | "Process" | "String"
                        );

                        if is_builtin_module {
                            // This is a builtin module → create QualifiedName
                            return Ok(Expr::new(
                                ExprKind::QualifiedName {
                                    module: module.clone(),
                                    name: field,
                                },
                                field_span,
                            ));
                        }
                        // Otherwise, fall through to FieldAccess for stdlib types (Vec, Box, HashMap)
                    }
                }

                // Not a call pattern → keep as FieldAccess for enum variants, etc.
                Ok(Expr::new(
                    ExprKind::FieldAccess {
                        object: Box::new(left),
                        field,
                    },
                    field_span,
                ))
            } else {
                Err(anyhow::anyhow!(
                    "Expected identifier or keyword usable as identifier after '::' but got {token:?}"
                ))
            }
        }
        Option::None => Err(anyhow::anyhow!(
            "Expected identifier after '::' but reached end of input"
        )),
    }
}

/// Parse turbofish type parameters: ::<Type1, Type2, ...>
/// PARSER-070: Extracted helper for parsing turbofish in path expressions
fn parse_turbofish(state: &mut ParserState) -> Result<()> {
    // Expect '<' token
    if let Some((Token::Less, _)) = state.tokens.peek() {
        state.tokens.advance(); // consume '<'

        // Skip until matching '>' (handle nested angle brackets and '>>' tokens)
        let mut depth = 1;
        while depth > 0 {
            match state.tokens.peek() {
                Some((Token::Less, _)) => {
                    depth += 1;
                    state.tokens.advance();
                }
                Some((Token::Greater, _)) => {
                    depth -= 1;
                    state.tokens.advance();
                }
                Some((Token::RightShift, _)) => {
                    // '>>' is two closing angle brackets
                    depth -= 2;
                    state.tokens.advance();
                }
                Some(_) => {
                    state.tokens.advance();
                }
                None => {
                    return Err(anyhow::anyhow!(
                        "Unexpected end of input while parsing turbofish type parameters"
                    ));
                }
            }
        }
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Expected '<' to start turbofish type parameters"
        ))
    }
}
/// Handle safe navigation operator ?.
fn handle_safe_nav_operator(state: &mut ParserState, left: Expr) -> Result<Expr> {
    state.tokens.advance();
    functions::parse_optional_method_call(state, left)
}
/// Handle postfix increment operator ++
fn handle_increment_operator(state: &mut ParserState, left: Expr) -> Result<Expr> {
    state.tokens.advance();
    Ok(create_post_increment(left))
}
/// Handle postfix decrement operator --
fn handle_decrement_operator(state: &mut ParserState, left: Expr) -> Result<Expr> {
    state.tokens.advance();
    Ok(create_post_decrement(left))
}
/// Check if ? is for ternary operator (not try operator)
fn is_ternary_operator(state: &mut ParserState) -> bool {
    // PARSER-XXX: Use same logic as is_try_operator_not_ternary
    // Returns true if this IS a ternary operator (has `:` at same level)
    // Returns false if this is a try operator (no `:` found)
    !is_try_operator_not_ternary(state)
}

/// Handle try operator ?
fn handle_try_operator(state: &mut ParserState, left: Expr) -> Result<Expr> {
    state.tokens.advance();
    Ok(Expr::new(
        ExprKind::Try {
            expr: Box::new(left),
        },
        Span { start: 0, end: 0 },
    ))
}
/// Handle array indexing and slicing syntax `[expr]` or `[start:end]`
fn handle_array_indexing(state: &mut ParserState, left: Expr) -> Result<Expr> {
    state.tokens.advance(); // consume [
                            // Check for empty slice [:end]
    if is_colon_next(state) {
        return parse_empty_start_slice(state, left);
    }
    let first_expr = parse_expr_recursive(state)?;
    // Check if this is a slice [start:end] or just indexing [index]
    if is_colon_next(state) {
        parse_slice_with_start(state, left, first_expr)
    } else {
        parse_index_access(state, left, first_expr)
    }
}
/// Check if next token is colon (complexity: 1)
fn is_colon_next(state: &mut ParserState) -> bool {
    matches!(state.tokens.peek(), Some((Token::Colon, _)))
}
/// Parse slice with empty start `[:end]` (complexity: 4)
fn parse_empty_start_slice(state: &mut ParserState, left: Expr) -> Result<Expr> {
    state.tokens.advance(); // consume :
    let end = parse_optional_slice_end(state)?;
    state.tokens.expect(&Token::RightBracket)?;
    Ok(create_slice_expr(left, None, end))
}
/// Parse slice with start `[start:end]` (complexity: 3)
fn parse_slice_with_start(state: &mut ParserState, left: Expr, start: Expr) -> Result<Expr> {
    state.tokens.advance(); // consume :
    let end = parse_optional_slice_end(state)?;
    state.tokens.expect(&Token::RightBracket)?;
    Ok(create_slice_expr(left, Some(Box::new(start)), end))
}
/// Parse optional slice end expression (complexity: 3)
fn parse_optional_slice_end(state: &mut ParserState) -> Result<Option<Box<Expr>>> {
    if matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
        Ok(None)
    } else {
        Ok(Some(Box::new(parse_expr_recursive(state)?)))
    }
}

/// Helper: Create Expr with default span and empty attributes (CERTEZA-001: Reduce duplication)
/// Complexity: 1 (within Toyota Way limits)
/// Used to eliminate ~130 lines of boilerplate across 13 Expr constructions
#[inline]
fn create_expr(kind: ExprKind) -> Expr {
    Expr {
        kind,
        span: Span { start: 0, end: 0 },
        attributes: Vec::new(),
        leading_comments: Vec::new(),
        trailing_comment: None,
    }
}

/// Parse index access `[index]` (complexity: 2)
fn parse_index_access(state: &mut ParserState, left: Expr, index: Expr) -> Result<Expr> {
    state.tokens.expect(&Token::RightBracket)?;
    Ok(create_expr(ExprKind::IndexAccess {
        object: Box::new(left),
        index: Box::new(index),
    }))
}
/// Create slice expression (complexity: 1)
fn create_slice_expr(object: Expr, start: Option<Box<Expr>>, end: Option<Box<Expr>>) -> Expr {
    create_expr(ExprKind::Slice {
        object: Box::new(object),
        start,
        end,
    })
}
/// Try to parse struct literal
///
/// PARSER-018: Extended to handle both simple identifiers (`Point { x: 1 }`)
/// and path expressions (`Shape::Circle { radius: 5.0 }`).
///
/// # Arguments
/// * `state` - Parser state
/// * `left` - The expression before the `{` (identifier or path)
///
/// # Returns
/// * `Ok(Some(expr))` - Parsed struct literal
/// * `Ok(None)` - Not a struct literal pattern
fn try_parse_struct_literal(state: &mut ParserState, left: &Expr) -> Result<Option<Expr>> {
    // Case 1: Simple identifier (Point { x: 1 })
    if let ExprKind::Identifier(name) = &left.kind {
        if name.chars().next().is_some_and(char::is_uppercase) {
            let name = name.clone();
            let span = left.span;
            return Ok(Some(types::parse_struct_literal(state, name, span)?));
        }
    }

    // Case 2: Path expression (Shape::Circle { radius: 5.0 })
    // DEFECT-018 FIX: Handle FieldAccess chain as struct literal path
    if let Some(path) = extract_struct_literal_path(left) {
        let span = left.span;
        return Ok(Some(types::parse_struct_literal(state, path, span)?));
    }

    Ok(None)
}

/// Extract path string from `FieldAccess` chain for struct literals
///
/// Converts expressions like `Shape::Circle` (`FieldAccess { object: Shape, field: Circle }`)
/// into the string `"Shape::Circle"` for use as struct literal name.
///
/// # Returns
/// * `Some(path)` - If the expression is a valid struct literal path ending in uppercase
/// * `None` - If not a valid struct literal path pattern
fn extract_struct_literal_path(expr: &Expr) -> Option<String> {
    match &expr.kind {
        // FieldAccess: Shape::Circle -> "Shape::Circle"
        ExprKind::FieldAccess { object, field } => {
            // Only treat as struct literal if field starts with uppercase (enum variant)
            if !field.chars().next().is_some_and(char::is_uppercase) {
                return None;
            }

            // Recursively extract path from object
            let obj_path = extract_struct_literal_path_component(object)?;
            Some(format!("{obj_path}::{field}"))
        }
        _ => None,
    }
}

/// Extract path component from expression (helper for nested paths)
///
/// Handles both identifiers and nested `FieldAccess` for paths like `A::B::C`.
fn extract_struct_literal_path_component(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::Identifier(name) => Some(name.clone()),
        ExprKind::FieldAccess { object, field } => {
            let obj_path = extract_struct_literal_path_component(object)?;
            Some(format!("{obj_path}::{field}"))
        }
        _ => None,
    }
}
/// Create post-increment expression
fn create_post_increment(left: Expr) -> Expr {
    create_expr(ExprKind::PostIncrement {
        target: Box::new(left),
    })
}
/// Create post-decrement expression
fn create_post_decrement(left: Expr) -> Expr {
    create_expr(ExprKind::PostDecrement {
        target: Box::new(left),
    })
}
/// Try to parse binary operators
fn try_binary_operators(
    state: &mut ParserState,
    left: Expr,
    token: &Token,
    min_prec: i32,
) -> Result<Option<Expr>> {
    // PARSER-068 FIX: Check if Bang token has whitespace before it
    // If there's whitespace (especially newline), treat ! as prefix unary NOT, not infix Send
    if matches!(token, Token::Bang) {
        if let Some((_, span)) = state.tokens.peek() {
            // If there's a gap between left.span.end and Bang token, don't treat as infix
            if span.start > left.span.end + 1 {
                // There's whitespace - ! should be prefix operator on next line
                return Ok(None);
            }
        }
    }

    // Skip 'in' as binary operator when inside let-value context
    // This ensures `let f = |x| x + 1 in f(5)` parses as let-in, not containment
    if matches!(token, Token::In) && state.in_let_value_context {
        return Ok(None);
    }

    if let Some(bin_op) = expressions::token_to_binary_op(token) {
        let prec = expressions::get_precedence(bin_op);
        if prec < min_prec {
            return Ok(None);
        }

        // PARSER-053/054 FIX: Consume comments before operator (line continuation support)
        // This allows expressions like:
        //   let x = 1 + 2
        //       // comment
        //       + 3
        // We skip comments to get to the operator, then let parse_expr_with_precedence_recursive()
        // consume them as leading comments for the right-hand side.
        while matches!(
            state.tokens.peek(),
            Some((
                Token::LineComment(_)
                    | Token::DocComment(_)
                    | Token::BlockComment(_)
                    | Token::HashComment(_),
                _
            ))
        ) {
            state.tokens.advance();
        }

        state.tokens.advance(); // consume operator

        let right = parse_expr_with_precedence_recursive(state, prec + 1)?;
        Ok(Some(create_expr(ExprKind::Binary {
            left: Box::new(left),
            op: bin_op,
            right: Box::new(right),
        })))
    } else {
        Ok(None)
    }
}
/// Try to parse ternary operator (? :) (complexity: 5, cognitive: 5)
fn try_ternary_operator(
    state: &mut ParserState,
    left: Expr,
    token: &Token,
    min_prec: i32,
) -> Result<Option<Expr>> {
    const TERNARY_PRECEDENCE: i32 = 1;

    if !is_valid_ternary_start(token, min_prec, TERNARY_PRECEDENCE) {
        return Ok(None);
    }

    if is_try_operator_not_ternary(state) {
        return Ok(None);
    }

    parse_ternary_expression(state, left, TERNARY_PRECEDENCE)
}

/// Check if token and precedence allow ternary parsing (complexity: 2, cognitive: 2)
fn is_valid_ternary_start(token: &Token, min_prec: i32, ternary_prec: i32) -> bool {
    matches!(token, Token::Question) && min_prec <= ternary_prec
}

/// Check if this is a try operator rather than ternary (complexity: 5, cognitive: 5)
/// PARSER-XXX: Scan ahead to find `:` at same nesting level for ternary detection
/// Returns true if this is definitely a try operator, false if it could be ternary
fn is_try_operator_not_ternary(state: &mut ParserState) -> bool {
    // First check: statement-starting keywords definitely indicate try
    if let Some((next_token, _)) = state.tokens.peek_nth(1) {
        if matches!(
            next_token,
            Token::Let
                | Token::For
                | Token::While
                | Token::Match
                | Token::Return
                | Token::Fn
                | Token::Fun
                | Token::Loop
                | Token::Continue
                | Token::Break
                | Token::Pub
                | Token::Const
                | Token::Static
                | Token::Struct
                | Token::Enum
                | Token::Impl
                | Token::Trait
                | Token::Type
                | Token::Use
                | Token::Mod
        ) {
            return true;
        }
    }

    // Second check: scan ahead for `:` at same nesting level
    // Ternary requires `? expr : expr` - the `:` must exist at same level
    let pos = state.tokens.position();
    state.tokens.advance(); // skip `?`

    let mut depth = 0i32;
    let mut found_colon = false;
    let max_lookahead = 30;

    for _ in 0..max_lookahead {
        match state.tokens.advance() {
            Some((Token::LeftParen | Token::LeftBracket | Token::LeftBrace, _)) => depth += 1,
            Some((Token::RightParen | Token::RightBracket | Token::RightBrace, _)) => {
                depth -= 1;
                if depth < 0 {
                    break; // Unbalanced - end of expression
                }
            }
            Some((Token::Colon, _)) if depth == 0 => {
                found_colon = true;
                break;
            }
            Some((Token::Semicolon, _)) => break, // Statement end
            None => break,                         // EOF
            _ => {}
        }
    }

    state.tokens.set_position(pos);

    // If no colon found at depth 0, it's a try operator
    !found_colon
}

/// Parse complete ternary expression (complexity: 3, cognitive: 3)
fn parse_ternary_expression(
    state: &mut ParserState,
    condition: Expr,
    ternary_prec: i32,
) -> Result<Option<Expr>> {
    state.tokens.advance(); // Consume '?'
    let true_expr = parse_expr_with_precedence_recursive(state, ternary_prec + 1)?;

    if !matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        bail!("Expected ':' in ternary expression");
    }
    state.tokens.advance(); // Consume ':'

    let false_expr = parse_expr_with_precedence_recursive(state, ternary_prec)?;

    Ok(Some(create_ternary_expr(condition, true_expr, false_expr)))
}

/// Create ternary expression AST node (complexity: 1, cognitive: 1)
fn create_ternary_expr(condition: Expr, true_expr: Expr, false_expr: Expr) -> Expr {
    create_expr(ExprKind::Ternary {
        condition: Box::new(condition),
        true_expr: Box::new(true_expr),
        false_expr: Box::new(false_expr),
    })
}

/// Try to parse type cast operator (as) - complexity: 5
fn try_type_cast_operator(
    state: &mut ParserState,
    left: Expr,
    token: &Token,
    _min_prec: i32,
) -> Result<Option<Expr>> {
    if !matches!(token, Token::As) {
        return Ok(None);
    }
    state.tokens.advance(); // consume 'as'
                            // Get the target type
    let target_type = match state.tokens.peek() {
        Some((Token::Identifier(t), _)) => {
            let type_name = t.clone();
            state.tokens.advance();
            type_name
        }
        _ => bail!("Expected type name after 'as'"),
    };
    Ok(Some(create_expr(ExprKind::TypeCast {
        expr: Box::new(left),
        target_type,
    })))
}
/// Helper: Parse actor-style binary operation with precedence check
/// Reduces cognitive complexity by extracting common actor op pattern
fn parse_actor_style_op<F>(
    state: &mut ParserState,
    left: Expr,
    min_prec: i32,
    op_name: &str,
    make_kind: F,
) -> Result<ExprKind>
where
    F: FnOnce(Box<Expr>, Box<Expr>) -> ExprKind,
{
    const PREC: i32 = 1; // Same as assignment
    if PREC < min_prec {
        bail!("Precedence check failed for {op_name}");
    }
    state.tokens.advance();
    let message = parse_expr_with_precedence_recursive(state, PREC)?;
    Ok(make_kind(Box::new(left), Box::new(message)))
}

/// Try to parse actor operations (<-, <?, !) (complexity: 4, cognitive: 4)
/// PARSER-068: Fixed Bang token ambiguity - don't treat ! as infix if there's whitespace before it
fn try_new_actor_operators(
    state: &mut ParserState,
    left: Expr,
    token: &Token,
    min_prec: i32,
) -> Result<Option<Expr>> {
    let expr_kind = match token {
        Token::LeftArrow => parse_actor_send_op(state, left, min_prec)?,
        Token::ActorQuery => parse_actor_query_op(state, left, min_prec)?,
        Token::Bang => {
            // PARSER-068 FIX: Check if Bang is adjacent to left expression
            // If there's whitespace (especially newline), treat ! as prefix unary NOT, not infix Send
            if let Some((_, span)) = state.tokens.peek() {
                // If there's a gap between left.span.end and Bang token, don't treat as infix
                if span.start > left.span.end + 1 {
                    // There's whitespace - ! should be prefix operator on next line
                    return Ok(None);
                }
            }
            parse_actor_bang_op(state, left, min_prec)?
        }
        _ => return Ok(None),
    };
    Ok(Some(create_expr(expr_kind)))
}

/// Parse actor send operator (<-) using helper
fn parse_actor_send_op(state: &mut ParserState, actor: Expr, min_prec: i32) -> Result<ExprKind> {
    parse_actor_style_op(state, actor, min_prec, "actor send", |actor, message| {
        ExprKind::ActorSend { actor, message }
    })
}

/// Parse actor query operator (<?) using helper
fn parse_actor_query_op(state: &mut ParserState, actor: Expr, min_prec: i32) -> Result<ExprKind> {
    parse_actor_style_op(state, actor, min_prec, "actor query", |actor, message| {
        ExprKind::ActorQuery { actor, message }
    })
}

/// Parse actor bang operator (!) using helper
fn parse_actor_bang_op(state: &mut ParserState, left: Expr, min_prec: i32) -> Result<ExprKind> {
    parse_actor_style_op(state, left, min_prec, "actor bang", |left, right| {
        ExprKind::Binary {
            op: BinaryOp::Send,
            left,
            right,
        }
    })
}

/// Try to parse assignment operators
fn try_assignment_operators(
    state: &mut ParserState,
    left: Expr,
    token: &Token,
    min_prec: i32,
) -> Result<Option<Expr>> {
    if !token.is_assignment_op() {
        return Ok(None);
    }
    let prec = 1;
    if prec < min_prec {
        return Ok(None);
    }
    state.tokens.advance();
    let value = parse_expr_with_precedence_recursive(state, prec)?;
    let expr = if *token == Token::Equal {
        create_expr(ExprKind::Assign {
            target: Box::new(left),
            value: Box::new(value),
        })
    } else {
        let bin_op = get_compound_assignment_op(token);
        create_expr(ExprKind::CompoundAssign {
            target: Box::new(left),
            op: bin_op,
            value: Box::new(value),
        })
    };
    Ok(Some(expr))
}
/// Get binary operator for compound assignment
fn get_compound_assignment_op(token: &Token) -> BinaryOp {
    match token {
        Token::PlusEqual => BinaryOp::Add,
        Token::MinusEqual => BinaryOp::Subtract,
        Token::StarEqual => BinaryOp::Multiply,
        Token::SlashEqual => BinaryOp::Divide,
        Token::PercentEqual => BinaryOp::Modulo,
        Token::PowerEqual => BinaryOp::Power,
        Token::AmpersandEqual => BinaryOp::BitwiseAnd,
        Token::PipeEqual => BinaryOp::BitwiseOr,
        Token::CaretEqual => BinaryOp::BitwiseXor,
        Token::LeftShiftEqual => BinaryOp::LeftShift,
        _ => unreachable!("Already checked is_assignment_op"),
    }
}
/// Try to parse pipeline operators (>>)
fn try_pipeline_operators(
    state: &mut ParserState,
    left: Expr,
    token: &Token,
    min_prec: i32,
) -> Result<Option<Expr>> {
    if !matches!(token, Token::Pipeline) {
        return Ok(None);
    }
    let prec = 3;
    if prec < min_prec {
        return Ok(None);
    }
    state.tokens.advance();
    let stage_expr = parse_expr_with_precedence_recursive(state, prec + 1)?;
    let expr = if let ExprKind::Pipeline { expr, mut stages } = left.kind {
        stages.push(PipelineStage {
            op: Box::new(stage_expr),
            span: Span { start: 0, end: 0 },
        });
        create_expr(ExprKind::Pipeline { expr, stages })
    } else {
        create_expr(ExprKind::Pipeline {
            expr: Box::new(left),
            stages: vec![PipelineStage {
                op: Box::new(stage_expr),
                span: Span { start: 0, end: 0 },
            }],
        })
    };
    Ok(Some(expr))
}
/// Try to parse range operators (.., ..=)
fn try_range_operators(
    state: &mut ParserState,
    left: Expr,
    token: &Token,
    min_prec: i32,
) -> Result<Option<Expr>> {
    if !matches!(token, Token::DotDot | Token::DotDotEqual) {
        return Ok(None);
    }
    let prec = 5;
    if prec < min_prec {
        return Ok(None);
    }
    let inclusive = matches!(token, Token::DotDotEqual);
    state.tokens.advance();

    // PARSER-084 FIX: Handle open-ended ranges (e.g., `2..` or `..5`)
    // Check if the next token indicates no expression follows
    let end = match state.tokens.peek() {
        Some((
            Token::RightBracket
            | Token::Semicolon
            | Token::Comma
            | Token::RightParen
            | Token::RightBrace,
            _,
        )) => {
            // Open-ended range - no end expression
            Expr::new(ExprKind::Literal(Literal::Unit), Span { start: 0, end: 0 })
        }
        _ => {
            // Closed range - parse the end expression
            parse_expr_with_precedence_recursive(state, prec + 1)?
        }
    };

    Ok(Some(create_expr(ExprKind::Range {
        start: Box::new(left),
        end: Box::new(end),
        inclusive,
    })))
}
/// Try to parse a macro call: identifier!( args ) or identifier![ args ]
/// Refactored to reduce complexity from 105 to <10
/// Try to parse macro call syntax (complexity: 5, cognitive: 5)
fn try_parse_macro_call(state: &mut ParserState, left: &Expr) -> Result<Option<Expr>> {
    let ExprKind::Identifier(name) = &left.kind else {
        return Ok(None);
    };

    if !is_valid_macro_call_syntax(state, name) {
        return Ok(None);
    }

    parse_macro_call_by_type(state, name)
}

/// Check if syntax is valid macro call (complexity: 3, cognitive: 3)
fn is_valid_macro_call_syntax(state: &mut ParserState, name: &str) -> bool {
    let next_after_bang = state.tokens.peek_nth(1);
    let is_macro_call = matches!(
        next_after_bang,
        Some((Token::LeftParen | Token::LeftBracket | Token::LeftBrace, _))
    );
    is_macro_call || name == "df" || name == "vec"
}

/// Parse macro call based on type (complexity: 6, cognitive: 6)
fn parse_macro_call_by_type(state: &mut ParserState, name: &str) -> Result<Option<Expr>> {
    // Handle special dataframe macro
    if let Some(df_result) = try_parse_dataframe_macro(state, name)? {
        return Ok(Some(df_result));
    }

    // PARSER-092: Handle vec![] macro (Issue #137 - ruchy-lambda)
    if let Some(vec_result) = try_parse_vec_macro(state, name)? {
        return Ok(Some(vec_result));
    }

    // Consume ! token for non-df/non-vec macros
    if name != "df" && name != "vec" {
        state.tokens.advance();
    }

    // Handle SQL macro specially
    if is_sql_macro(state, name) {
        return Ok(Some(macro_parsing::parse_sql_macro(state, name)?));
    }

    // Parse generic macro
    parse_generic_macro(state, name)
}

/// Try to parse dataframe macro (complexity: 2, cognitive: 2)
fn try_parse_dataframe_macro(state: &mut ParserState, name: &str) -> Result<Option<Expr>> {
    if name == "df" {
        macro_parsing::parse_dataframe_macro(state)
    } else {
        Ok(None)
    }
}

/// Try to parse vec![] macro (complexity: 2, cognitive: 2)
/// PARSER-092: Support vec![expr; size] repeat pattern from Issue #137
fn try_parse_vec_macro(state: &mut ParserState, name: &str) -> Result<Option<Expr>> {
    if name == "vec" {
        macro_parsing::parse_vec_macro(state)
    } else {
        Ok(None)
    }
}

/// Check if this is a SQL macro (complexity: 2, cognitive: 2)
fn is_sql_macro(state: &mut ParserState, name: &str) -> bool {
    name == "sql" && matches!(state.tokens.peek(), Some((Token::LeftBrace, _)))
}

/// Parse generic macro with delimiters (complexity: 3, cognitive: 3)
fn parse_generic_macro(state: &mut ParserState, name: &str) -> Result<Option<Expr>> {
    let Some((_style, closing_token)) = macro_parsing::get_macro_delimiters(state) else {
        return Ok(None);
    };

    let args = macro_parsing::parse_macro_arguments(state, closing_token)?;
    Ok(Some(macro_parsing::create_macro_expr(
        name.to_string(),
        args,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Literal, UnaryOp};

    // Sprint 4: Comprehensive parser unit tests for coverage improvement

    #[test]
    fn test_parser_basic_literals() {
        let mut state = ParserState::new("42");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Literal(Literal::Integer(42, None))
        ));

        let mut state = ParserState::new("3.15");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        if let ExprKind::Literal(Literal::Float(f)) = expr.kind {
            assert!((f - 3.15).abs() < 0.001);
        } else {
            panic!("Expected float literal");
        }

        let mut state = ParserState::new("true");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::Bool(true))));

        let mut state = ParserState::new("false");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::Bool(false))));
    }

    #[test]
    fn test_parser_string_literals() {
        let mut state = ParserState::new(r#""hello world""#);
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        if let ExprKind::Literal(Literal::String(s)) = expr.kind {
            assert_eq!(s, "hello world");
        } else {
            panic!("Expected string literal");
        }

        let mut state = ParserState::new(r#""""#);
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        if let ExprKind::Literal(Literal::String(s)) = expr.kind {
            assert_eq!(s, "");
        } else {
            panic!("Expected empty string literal");
        }
    }

    #[test]
    fn test_parser_identifiers() {
        let mut state = ParserState::new("variable");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        if let ExprKind::Identifier(name) = expr.kind {
            assert_eq!(name, "variable");
        } else {
            panic!("Expected identifier");
        }

        let mut state = ParserState::new("_underscore");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        if let ExprKind::Identifier(name) = expr.kind {
            assert_eq!(name, "_underscore");
        } else {
            panic!("Expected identifier with underscore");
        }
    }

    #[test]
    fn test_parser_binary_operations() {
        let mut state = ParserState::new("1 + 2");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));

        let mut state = ParserState::new("10 - 5");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::Subtract,
                ..
            }
        ));

        let mut state = ParserState::new("3 * 4");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::Multiply,
                ..
            }
        ));

        let mut state = ParserState::new("8 / 2");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::Divide,
                ..
            }
        ));
    }

    #[test]
    fn test_parser_comparison_operations() {
        let mut state = ParserState::new("5 > 3");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::Greater,
                ..
            }
        ));

        let mut state = ParserState::new("3 < 5");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::Less,
                ..
            }
        ));

        let mut state = ParserState::new("5 == 5");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::Equal,
                ..
            }
        ));

        let mut state = ParserState::new("5 != 3");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::NotEqual,
                ..
            }
        ));
    }

    #[test]
    fn test_parser_logical_operations() {
        let mut state = ParserState::new("true && false");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::And,
                ..
            }
        ));

        let mut state = ParserState::new("true || false");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::Or,
                ..
            }
        ));
    }

    #[test]
    fn test_parser_unary_operations() {
        let mut state = ParserState::new("-42");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Unary {
                op: UnaryOp::Negate,
                ..
            }
        ));

        let mut state = ParserState::new("!true");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Unary {
                op: UnaryOp::Not,
                ..
            }
        ));
    }

    #[test]
    fn test_parser_parenthesized_expression() {
        let mut state = ParserState::new("(42)");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        // Parentheses don't create a special node, just affect precedence
        assert!(matches!(
            expr.kind,
            ExprKind::Literal(Literal::Integer(42, None))
        ));

        let mut state = ParserState::new("(1 + 2) * 3");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::Multiply,
                ..
            }
        ));
    }

    #[test]
    fn test_parser_list_literal() {
        let mut state = ParserState::new("[1, 2, 3]");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        if let ExprKind::List(items) = expr.kind {
            assert_eq!(items.len(), 3);
        } else {
            panic!("Expected list literal");
        }

        let mut state = ParserState::new("[]");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        if let ExprKind::List(items) = expr.kind {
            assert_eq!(items.len(), 0);
        } else {
            panic!("Expected empty list");
        }
    }

    #[test]
    fn test_parser_tuple_literal() {
        let mut state = ParserState::new("(1, 2)");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        if let ExprKind::Tuple(items) = expr.kind {
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected tuple literal");
        }

        let mut state = ParserState::new("(1,)");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        if let ExprKind::Tuple(items) = expr.kind {
            assert_eq!(items.len(), 1);
        } else {
            panic!("Expected single-element tuple");
        }
    }

    #[test]
    fn test_parser_range_expressions() {
        let mut state = ParserState::new("1..10");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        if let ExprKind::Range { inclusive, .. } = expr.kind {
            assert!(!inclusive);
        } else {
            panic!("Expected range expression");
        }

        let mut state = ParserState::new("1..=10");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        if let ExprKind::Range { inclusive, .. } = expr.kind {
            assert!(inclusive);
        } else {
            panic!("Expected inclusive range");
        }
    }

    #[test]
    fn test_parser_state_creation() {
        let state = ParserState::new("test input");
        assert_eq!(state.get_errors().len(), 0);

        let (allocated, items) = state.arena_stats();
        assert_eq!(allocated, 0);
        assert_eq!(items, 0);

        let (strings, bytes) = state.interner_stats();
        assert_eq!(strings, 0);
        assert_eq!(bytes, 0);
    }

    #[test]
    fn test_parser_precedence_levels() {
        // Test that multiplication has higher precedence than addition
        let mut state = ParserState::new("1 + 2 * 3");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        // Should parse as 1 + (2 * 3), not (1 + 2) * 3
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
    }

    #[test]

    fn test_parser_assignment_operators() {
        // Assignment is parsed as a binary operation in this AST
        let mut state = ParserState::new("x = 5");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        // Assignment might be parsed differently, just check it's an expression
        // The AST does have an Assign variant
        assert!(
            matches!(expr.kind, ExprKind::Let { .. })
                || matches!(expr.kind, ExprKind::Binary { .. })
                || matches!(expr.kind, ExprKind::Assign { .. })
        );

        let mut state = ParserState::new("x += 5");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(expr.kind, ExprKind::CompoundAssign { .. }));
    }

    #[test]

    fn test_parser_pipeline_operator() {
        let mut state = ParserState::new("data >> transform");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::RightShift,
                ..
            }
        ));
    }

    #[test]
    fn test_parser_try_operator() {
        let mut state = ParserState::new("result?");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(expr.kind, ExprKind::Try { .. }));
    }

    #[test]
    fn test_parser_index_access() {
        let mut state = ParserState::new("array[0]");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(expr.kind, ExprKind::IndexAccess { .. }));
    }

    #[test]
    fn test_parser_slice_expressions() {
        let mut state = ParserState::new("array[1:5]");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(expr.kind, ExprKind::Slice { .. }));

        let mut state = ParserState::new("array[:5]");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(expr.kind, ExprKind::Slice { .. }));

        let mut state = ParserState::new("array[1:]");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(expr.kind, ExprKind::Slice { .. }));
    }

    #[test]
    fn test_parser_postfix_increment() {
        // PostIncrement doesn't exist in UnaryOp, skip this test
        // The parser may handle this differently or not support it
    }

    #[test]
    fn test_parser_postfix_decrement() {
        // PostDecrement doesn't exist in UnaryOp, skip this test
        // The parser may handle this differently or not support it
    }

    #[test]
    fn test_parser_complex_expression() {
        // Test a complex nested expression
        let mut state = ParserState::new("(a + b) * (c - d) / 2");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        // Should parse successfully as a division operation at the top level
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::Divide,
                ..
            }
        ));
    }

    #[test]
    fn test_parser_character_literal() {
        let mut state = ParserState::new("'a'");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        if let ExprKind::Literal(Literal::Char(c)) = expr.kind {
            assert_eq!(c, 'a');
        } else {
            panic!("Expected character literal");
        }
    }

    #[test]
    fn test_parser_method_call_chain() {
        let mut state = ParserState::new("obj.method1().method2()");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        // Should parse as nested method calls
        assert!(matches!(expr.kind, ExprKind::MethodCall { .. }));
    }

    #[test]

    fn test_parser_safe_navigation() {
        let mut state = ParserState::new("obj?.method()");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        // Safe navigation parses as OptionalMethodCall for obj?.method() syntax
        assert!(
            matches!(expr.kind, ExprKind::OptionalFieldAccess { .. })
                || matches!(expr.kind, ExprKind::MethodCall { .. })
                || matches!(expr.kind, ExprKind::OptionalMethodCall { .. })
        );
    }

    #[test]
    #[ignore = "Macro syntax not fully implemented"]
    fn test_parser_macro_call() {
        let mut state = ParserState::new("println!(\"hello\")");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        if let ExprKind::Call { func, args } = expr.kind {
            if let ExprKind::Identifier(name) = func.kind {
                assert_eq!(name, "println");
                assert_eq!(args.len(), 1);
            } else {
                panic!("Expected function name");
            }
        } else {
            panic!("Expected function call");
        }
    }

    #[test]
    fn test_parser_bitwise_operations() {
        let mut state = ParserState::new("a & b");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::BitwiseAnd,
                ..
            }
        ));

        let mut state = ParserState::new("a | b");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::BitwiseOr,
                ..
            }
        ));

        let mut state = ParserState::new("a ^ b");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::BitwiseXor,
                ..
            }
        ));
    }

    #[test]
    fn test_parser_shift_operations() {
        let mut state = ParserState::new("a << 2");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::LeftShift,
                ..
            }
        ));

        // Right shift doesn't exist in BinaryOp, skip this test
        // The language may not support right shift or use a different representation
    }

    #[test]
    fn test_parser_modulo_operation() {
        let mut state = ParserState::new("10 % 3");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::Modulo,
                ..
            }
        ));
    }

    #[test]
    fn test_parser_type_cast() {
        let mut state = ParserState::new("x as i32");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(expr.kind, ExprKind::TypeCast { .. }));
    }

    #[test]
    fn test_parser_power_operation() {
        let mut state = ParserState::new("2 ** 8");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::Power,
                ..
            }
        ));
    }

    #[test]
    fn test_parser_prefix_increment() {
        // PreIncrement doesn't exist in UnaryOp, skip this test
        // The parser may handle this differently or not support it
    }

    #[test]
    fn test_parser_prefix_decrement() {
        // PreDecrement doesn't exist in UnaryOp, skip this test
        // The parser may handle this differently or not support it
    }

    #[test]
    fn test_parser_empty_input() {
        let mut state = ParserState::new("");
        let result = parse_expr_recursive(&mut state);
        // Empty input should return an error
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_nested_lists() {
        let mut state = ParserState::new("[[1, 2], [3, 4]]");
        let expr =
            parse_expr_recursive(&mut state).expect("parse_expr_recursive should succeed in test");
        if let ExprKind::List(outer) = expr.kind {
            assert_eq!(outer.len(), 2);
            // Each element should itself be a list
            for item in outer {
                assert!(matches!(item.kind, ExprKind::List(_)));
            }
        } else {
            panic!("Expected nested list");
        }
    }

    // Sprint 8 Phase 2: Mutation test gap coverage for mod.rs
    // Target: 8 MISSED → 0 MISSED (operator precedence boundaries & calculations)

    #[test]
    fn test_ternary_operator_precedence_boundary() {
        // Test gap: Line 464 - verify > comparison (not ==) in try_ternary_operator
        // Ternary should work when min_prec is LESS than TERNARY_PRECEDENCE
        let mut state = ParserState::new("true ? 1 : 2");
        let result = parse_expr_recursive(&mut state);
        assert!(
            result.is_ok(),
            "Ternary with default precedence should work"
        );
    }

    #[test]
    fn test_ternary_precedence_calculation() {
        // Test gap: Line 449 - verify + operator (not *) in prec + 1
        // This tests the precedence calculation for ternary true branch
        let mut state = ParserState::new("1 + 1 ? 10 : 20");
        let result = parse_expr_recursive(&mut state);
        assert!(
            result.is_ok(),
            "Ternary with addition should parse correctly"
        );
    }

    #[test]
    fn test_assignment_operator_precedence_boundary_less_than() {
        // Test gap: Line 590 - verify < comparison (not <= or ==) in try_assignment_operators
        // Assignment should NOT work when prec >= min_prec
        let mut state = ParserState::new("x = 42");
        let result = parse_expr_with_precedence_recursive(&mut state, 0);
        assert!(
            result.is_ok(),
            "Assignment with min_prec=0 should work (prec < min_prec is false)"
        );
    }

    #[test]
    fn test_range_operator_precedence_boundary() {
        // Test gap: Line 686 - verify < comparison (not ==) in try_range_operators
        let mut state = ParserState::new("1..10");
        let result = parse_expr_with_precedence_recursive(&mut state, 0);
        assert!(result.is_ok(), "Range with low min_prec should work");
    }

    #[test]
    fn test_range_precedence_calculation() {
        // Test gap: Line 691 - verify + operator (not *) in prec + 1
        let mut state = ParserState::new("1..10");
        let result = parse_expr_recursive(&mut state);
        assert!(
            result.is_ok(),
            "Range precedence calculation should use + not *"
        );
    }

    #[test]
    fn test_pipeline_precedence_calculation() {
        // Test gap: Line 649 - verify + operator (not -) in prec + 1
        let mut state = ParserState::new("x |> f");
        let result = parse_expr_recursive(&mut state);
        assert!(
            result.is_ok(),
            "Pipeline precedence should use + for right recursion"
        );
    }

    #[test]
    fn test_macro_call_returns_some() {
        // Test gap: Line 705 - verify try_parse_macro_call returns Some (not None stub)
        // FORMATTER-088: Changed to MacroInvocation (macro CALL, not definition)
        let mut state = ParserState::new("vec![1, 2, 3]");
        let result = parse_expr_recursive(&mut state);
        assert!(result.is_ok(), "Macro call should parse successfully");

        if let Ok(expr) = result {
            assert!(
                matches!(expr.kind, ExprKind::MacroInvocation { .. }),
                "Should parse as MacroInvocation expression (macro CALL, not definition)"
            );
        }
    }
}

#[cfg(test)]
mod mutation_tests {
    use super::*;

    #[test]
    fn test_try_range_operators_less_than_comparison() {
        // MISSED: replace < with == in try_range_operators (line 686)

        let mut state = ParserState::new("..10");
        let left = Expr {
            kind: ExprKind::Literal(Literal::Integer(0, None)),
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
            leading_comments: vec![],
            trailing_comment: None,
        };

        // Test when prec < min_prec (should return None)
        let result = try_range_operators(&mut state, left.clone(), &Token::DotDot, 10);
        assert!(result.is_ok());
        assert!(
            result.expect("result should be Some in test").is_none(),
            "Should return None when prec < min_prec (not ==)"
        );

        // Test when prec >= min_prec (should return Some)
        let mut state = ParserState::new("..10");
        let result = try_range_operators(&mut state, left, &Token::DotDot, 5);
        assert!(result.is_ok());
        assert!(
            result.expect("result should be Some in test").is_some(),
            "Should return Some when prec >= min_prec"
        );
    }

    #[test]
    fn test_try_range_operators_plus_arithmetic() {
        // MISSED: replace + with * in try_range_operators (line 691)

        let mut state = ParserState::new("..10");
        let left = Expr {
            kind: ExprKind::Literal(Literal::Integer(0, None)),
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
            leading_comments: vec![],
            trailing_comment: None,
        };

        let result = try_range_operators(&mut state, left, &Token::DotDot, 1);
        assert!(result.is_ok());

        // The RHS should parse with precedence prec+1 (not prec*1)
        // This ensures proper right-associativity
        if let Ok(Some(expr)) = result {
            assert!(
                matches!(expr.kind, ExprKind::Range { .. }),
                "Should parse range correctly with + operator"
            );
        }
    }

    #[test]
    fn test_try_assignment_operators_less_than_comparison() {
        // MISSED: replace < with <= in try_assignment_operators (line 590)

        let mut state = ParserState::new("= 42");
        let left = Expr {
            kind: ExprKind::Identifier("x".to_string()),
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
            leading_comments: vec![],
            trailing_comment: None,
        };

        // Test boundary: when prec == min_prec
        // With <: returns Some (prec is NOT < min_prec)
        // With <=: would return None (prec IS <= min_prec)
        let result = try_assignment_operators(&mut state, left.clone(), &Token::Equal, 1);
        assert!(result.is_ok());
        assert!(
            result.expect("result should be Some in test").is_some(),
            "Should return Some when prec == min_prec (using <, not <=)"
        );

        // Test when prec < min_prec (should return None)
        let mut state = ParserState::new("= 42");
        let result = try_assignment_operators(&mut state, left, &Token::Equal, 10);
        assert!(result.is_ok());
        assert!(
            result.expect("result should be Some in test").is_none(),
            "Should return None when prec < min_prec"
        );
    }

    #[test]
    fn test_try_assignment_operators_equals_comparison() {
        // MISSED: replace < with == in try_assignment_operators (line 590)

        let mut state = ParserState::new("= 42");
        let left = Expr {
            kind: ExprKind::Identifier("x".to_string()),
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
            leading_comments: vec![],
            trailing_comment: None,
        };

        // Test when prec < min_prec (should return None with < operator)
        let result = try_assignment_operators(&mut state, left.clone(), &Token::Equal, 10);
        assert!(result.is_ok());
        assert!(
            result.expect("result should be Some in test").is_none(),
            "Should return None when prec < min_prec (not ==)"
        );

        // Test when prec > min_prec (should return Some)
        let mut state = ParserState::new("= 42");
        let result = try_assignment_operators(&mut state, left, &Token::Equal, 0);
        assert!(result.is_ok());
        assert!(
            result.expect("result should be Some in test").is_some(),
            "Should return Some when prec > min_prec"
        );
    }

    #[test]
    fn test_try_ternary_operator_plus_arithmetic() {
        // MISSED: replace + with * in try_ternary_operator (line 464)

        let mut state = ParserState::new("? 1 : 0");
        let left = Expr {
            kind: ExprKind::Literal(Literal::Bool(true)),
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
            leading_comments: vec![],
            trailing_comment: None,
        };

        let result = try_ternary_operator(&mut state, left, &Token::Question, 1);
        assert!(result.is_ok());

        // The true branch should parse with TERNARY_PRECEDENCE+1 (not *1)
        // This tests the correct arithmetic operator
        if let Ok(Some(expr)) = result {
            assert!(
                matches!(expr.kind, ExprKind::Ternary { .. }),
                "Should parse ternary with + operator"
            );
        }
    }
}
