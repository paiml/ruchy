//! Frontend parsing and lexical analysis
//!
//! This module handles tokenization, parsing, and AST construction.

pub mod ast;
pub mod error_recovery;
pub mod lexer;
pub mod parser;

pub use ast::*;
pub use error_recovery::{ParseError, ParseResult, RecoveryParser};
pub use lexer::{Token, TokenStream};
pub use parser::Parser;
