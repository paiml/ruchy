//! Frontend parsing and lexical analysis
//!
//! This module handles tokenization, parsing, and AST construction for the
//! Ruchy programming language, implementing a complete frontend compiler pipeline.
//!
//! # Architecture
//!
//! The frontend follows the traditional compiler design:
//!
//! ```text
//! Source Code → Lexer → Token Stream → Parser → AST
//!      ↓          ↓         ↓           ↓        ↓
//!   String    Tokens   TokenStream   Parse   Expressions
//! ```
//!
//! # Components
//!
//! ## Lexer
//! Converts source code into a stream of tokens:
//! - Keyword recognition and context-sensitive parsing
//! - String literals with escape sequences
//! - Numeric literals (integers, floats, scientific notation)
//! - Operators and punctuation
//! - Comment handling and whitespace management
//!
//! ## Parser
//! Builds an Abstract Syntax Tree (AST) from tokens:
//! - **Pratt Parser**: Operator precedence parsing
//! - **Recursive Descent**: For statements and declarations
//! - **Error Recovery**: Continue parsing after syntax errors
//! - **Look-ahead**: Minimal token buffering for efficiency
//!
//! ## AST
//! Type-safe representation of Ruchy programs:
//! - Expression nodes for all value-producing constructs
//! - Pattern matching for destructuring
//! - Type annotations and generics
//! - Actor system constructs
//! - DataFrame and data science operations
//!
//! ## Error Recovery
//! Robust error handling for interactive development:
//! - Syntax error reporting with location information
//! - Error recovery strategies to continue parsing
//! - Diagnostic suggestions for common mistakes
//!
//! ## Memory Management
//! Efficient AST construction with arena allocation:
//! - Bump allocation for AST nodes
//! - String interning for identifiers
//! - Memory reuse across parse sessions
//!
//! # Examples
//!
//! ## Basic Parsing
//! ```
//! use ruchy::frontend::{Parser, Expr, ExprKind};
//!
//! let mut parser = Parser::new("42 + 3.14");
//! let ast = parser.parse().unwrap();
//!
//! if let ExprKind::Binary { left, op, right } = &ast.kind {
//!     println!("Found binary operation: {:?}", op);
//! }
//! ```
//!
//! ## Tokenization
//! ```
//! use ruchy::frontend::{TokenStream, Token};
//!
//! let mut tokens = TokenStream::new("let x = 42");
//! while let Some((token, _span)) = tokens.next() {
//!     match token {
//!         Token::Let => println!("Let keyword"),
//!         Token::Identifier(name) => println!("Identifier: {}", name),
//!         Token::Integer(n) => println!("Number: {}", n),
//!         _ => {}
//!     }
//! }
//! ```
//!
//! ## Error Handling
//! ```
//! use ruchy::frontend::Parser;
//!
//! let mut parser = Parser::new("let x = ");  // Incomplete
//! match parser.parse() {
//!     Ok(ast) => println!("Parsed: {:?}", ast),
//!     Err(error) => println!("Parse error: {}", error),
//! }
//! ```
//!
//! # Language Features Supported
//!
//! - **Expressions**: Literals, variables, operators, function calls
//! - **Control Flow**: if/else, match, loops (for, while)
//! - **Functions**: Definitions, lambdas, generics, async/await
//! - **Data Types**: Structs, enums, traits, implementations
//! - **Pattern Matching**: Destructuring, guards, exhaustiveness
//! - **Actors**: Message passing, supervision, ask/tell operations
//! - **DataFrames**: Column operations, data manipulation
//! - **Imports**: Module system with dependency resolution
pub mod arena;
pub mod ast;
pub mod diagnostics;
pub mod error_recovery;
pub mod lexer;
pub mod parser;
pub use ast::*;
pub use error_recovery::{ParseError, ParseResult, RecoveryParser};
pub use lexer::{Token, TokenStream};
pub use parser::Parser;
