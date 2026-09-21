//! RHL text → intent tree (spec RHL-001 §2, row RHL-1).
//!
//! [`parse`] runs the LALRPOP parser generated from `src/rhl/grammar.lalrpop`
//! and turns its borrowed error into an owned [`ParseFailure`], so a later
//! diagnostics phase can keep failures past the lifetime of the source text.
//!
//! The `pub(super)` builders below are the grammar's action code. They live
//! here, not in the grammar file, so every action in `grammar.lalrpop` is one
//! call and the arithmetic on spans is written once, in ordinary Rust.

use super::grammar::ProgramParser;
use super::tree::{
    Cond, CondKind, Decl, Effect, EffectVerb, Int, Phrase, Program, Quantity, Span, Stmt, StmtKind,
    Text, Unit, UnitKind, UseDecl, Word,
};
use lalrpop_util::lexer::Token;
use lalrpop_util::ParseError;

/// Why a source text is not an RHL program. Owned: no borrow of the source.
///
/// Offsets and spans are byte positions into the parsed text. `expected`
/// holds LALRPOP's own terminal spellings, for example `"\"end\""` for the
/// keyword `end` and `"NEWLINE"` for a named terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseFailure {
    /// No terminal of the grammar matches the text at byte `at`.
    InvalidToken {
        /// Byte offset of the first unmatched character.
        at: usize,
    },
    /// The text ended where the grammar needed more, for example a missing `end`.
    UnrecognizedEof {
        /// Byte offset of the end of the text (as LALRPOP reports it).
        at: usize,
        /// The terminals that could have come next.
        expected: Vec<String>,
    },
    /// A token appeared where the grammar does not allow it.
    UnrecognizedToken {
        /// Where the token is.
        span: Span,
        /// The token's text.
        found: String,
        /// The terminals that could have come instead.
        expected: Vec<String>,
    },
    /// A complete program was followed by another token.
    ExtraToken {
        /// Where the token is.
        span: Span,
        /// The token's text.
        found: String,
    },
}

/// Parse one `.rhl` source text into its intent tree.
///
/// # Errors
///
/// Returns a [`ParseFailure`] when `source` is not a program of the RHL v0
/// grammar.
pub fn parse(source: &str) -> Result<Program, ParseFailure> {
    ProgramParser::new().parse(source).map_err(failure)
}

/// Own a LALRPOP error. The grammar's actions are all infallible (`=>`, never
/// `=>?`), so LALRPOP has no code path that builds `ParseError::User`; the
/// last arm documents that, and `parse_tests` checks the grammar for `=>?`.
fn failure(e: ParseError<usize, Token<'_>, &'static str>) -> ParseFailure {
    match e {
        ParseError::InvalidToken { location } => ParseFailure::InvalidToken { at: location },
        ParseError::UnrecognizedEof { location, expected } => ParseFailure::UnrecognizedEof {
            at: location,
            expected,
        },
        ParseError::UnrecognizedToken {
            token: (l, t, r),
            expected,
        } => ParseFailure::UnrecognizedToken {
            span: Span::new(l, r),
            found: t.1.to_string(),
            expected,
        },
        ParseError::ExtraToken { token: (l, t, r) } => ParseFailure::ExtraToken {
            span: Span::new(l, r),
            found: t.1.to_string(),
        },
        ParseError::User { error } => unreachable!(
            "RHL-1: grammar.lalrpop has no fallible action, yet LALRPOP reported `{error}`"
        ),
    }
}

/// Append `item` to a left-recursive list and hand the list back.
pub(super) fn push<T>(mut list: Vec<T>, item: T) -> Vec<T> {
    list.push(item);
    list
}

/// A word written at byte `start`.
pub(super) fn word(start: usize, text: &str) -> Word {
    Word {
        text: text.to_string(),
        span: Span::new(start, start + text.len()),
    }
}

/// A string literal written at byte `start`; the value drops the quotes.
pub(super) fn text(start: usize, literal: &str) -> Text {
    let inner = literal
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(literal);
    Text {
        value: inner.to_string(),
        span: Span::new(start, start + literal.len()),
    }
}

/// An integer literal written at byte `start`, leading zeros dropped.
pub(super) fn int(start: usize, digits: &str) -> Int {
    let trimmed = digits.trim_start_matches('0');
    let canonical = if trimmed.is_empty() { "0" } else { trimmed };
    Int {
        digits: canonical.to_string(),
        span: Span::new(start, start + digits.len()),
    }
}

/// `INT [unit]` from byte `start` to byte `end`; the unit is the last token.
pub(super) fn quantity(start: usize, digits: &str, unit: Option<&str>, end: usize) -> Quantity {
    Quantity {
        value: int(start, digits),
        unit: unit.map(|u| word(end - u.len(), u)),
        span: Span::new(start, end),
    }
}

/// A one-word phrase.
pub(super) fn phrase(start: usize, w: &str) -> Phrase {
    let first = word(start, w);
    Phrase {
        span: first.span,
        words: vec![first],
    }
}

/// `phrase` followed by one more word.
pub(super) fn extend(mut p: Phrase, start: usize, w: &str) -> Phrase {
    let next = word(start, w);
    p.span = p.span.to(next.span);
    p.words.push(next);
    p
}

/// `use vocabulary <phrase>` from byte `start` to byte `end`.
pub(super) fn use_decl(start: usize, vocabulary: Phrase, end: usize) -> Decl {
    Decl::Use(UseDecl {
        vocabulary,
        span: Span::new(start, end),
    })
}

/// A unit block from its kind keyword (`start`) to its `end` (`end`).
pub(super) fn unit(start: usize, kind: UnitKind, name: Text, body: Vec<Stmt>, end: usize) -> Decl {
    Decl::Unit(Unit {
        kind,
        name,
        body,
        span: Span::new(start, end),
    })
}

/// A statement from byte `start` to byte `end`.
pub(super) fn stmt(start: usize, kind: StmtKind, end: usize) -> Stmt {
    Stmt {
        kind,
        span: Span::new(start, end),
    }
}

/// `may <verb> <target>` from byte `start` to byte `end`.
pub(super) fn may(start: usize, verb: EffectVerb, target: Phrase, end: usize) -> Stmt {
    stmt(start, StmtKind::May(Effect { verb, target }), end)
}

/// A condition from byte `start` to byte `end`.
pub(super) fn cond(start: usize, kind: CondKind, end: usize) -> Cond {
    Cond {
        kind,
        span: Span::new(start, end),
    }
}

/// `left or right`, spanning both operands.
pub(super) fn or(left: Cond, right: Cond) -> Cond {
    let span = left.span.to(right.span);
    Cond {
        kind: CondKind::Or(Box::new(left), Box::new(right)),
        span,
    }
}

/// `left and right`, spanning both operands.
pub(super) fn and(left: Cond, right: Cond) -> Cond {
    let span = left.span.to(right.span);
    Cond {
        kind: CondKind::And(Box::new(left), Box::new(right)),
        span,
    }
}
