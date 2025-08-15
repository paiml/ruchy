//! Ruchy - A systems-oriented scripting language that transpiles to Rust
//!
//! Ruchy provides a high-level, expressive syntax with features like pipeline operators,
//! pattern matching, and actor-based concurrency, all while transpiling to efficient Rust code.
//!
//! # Quick Start
//!
//! ```
//! use ruchy::frontend::parser::Parser;
//! use ruchy::backend::transpiler::Transpiler;
//!
//! // Parse Ruchy code
//! let mut parser = Parser::new("1 + 2 * 3");
//! let expr = parser.parse().expect("Failed to parse");
//!
//! // Transpile to Rust
//! let transpiler = Transpiler::new();
//! let rust_code = transpiler.transpile_expr(&expr).expect("Failed to transpile");
//! ```
//!
//! # Features
//!
//! - **Pipeline Operators**: Chain operations with `|>` for readable data transformations
//! - **Pattern Matching**: Powerful pattern matching with exhaustiveness checking
//! - **Actor System**: Built-in actor model for concurrent programming
//! - **Type Inference**: Hindley-Milner type inference with extensions
//! - **Zero-Cost Abstractions**: All features compile to efficient Rust code
//!
//! # Examples
//!
//! ## Pipeline Operations
//!
//! ```
//! use ruchy::frontend::parser::Parser;
//!
//! let code = r#"
//!     [1, 2, 3, 4, 5]
//!     |> filter(x => x % 2 == 0)
//!     |> map(x => x * 2)
//!     |> sum()
//! "#;
//!
//! let mut parser = Parser::new(code);
//! let expr = parser.parse().expect("Failed to parse pipeline");
//! ```
//!
//! ## Pattern Matching
//!
//! ```
//! use ruchy::frontend::parser::Parser;
//!
//! let code = r#"
//!     match result {
//!         Ok(value) => process(value),
//!         Err(e) => handle_error(e),
//!     }
//! "#;
//!
//! let mut parser = Parser::new(code);
//! let expr = parser.parse().expect("Failed to parse match expression");
//! ```
//!
//! ## Actor Definition
//!
//! ```
//! use ruchy::frontend::parser::Parser;
//!
//! let code = r#"
//!     actor Counter {
//!         mut count: i32 = 0;
//!         
//!         pub fn increment() {
//!             self.count += 1;
//!         }
//!         
//!         pub fn get() -> i32 {
//!             self.count
//!         }
//!     }
//! "#;
//!
//! let mut parser = Parser::new(code);
//! let expr = parser.parse().expect("Failed to parse actor");
//! ```

pub mod backend;
pub mod frontend;
pub mod runtime;

// Re-export commonly used types
pub use backend::transpiler::Transpiler;
pub use frontend::{ast::{Expr, ExprKind}, parser::Parser};
pub use runtime::repl::Repl;

/// Parse and transpile Ruchy code to Rust in one step.
///
/// # Examples
///
/// ```
/// use ruchy::compile;
///
/// let rust_code = compile("42 + 1").expect("Failed to compile");
/// assert!(rust_code.contains("42"));
/// assert!(rust_code.contains("+"));
/// assert!(rust_code.contains("1"));
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The input contains syntax errors
/// - The code cannot be transpiled to valid Rust
pub fn compile(input: &str) -> anyhow::Result<String> {
    let mut parser = Parser::new(input);
    let expr = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_expr(&expr)?;
    Ok(rust_code.to_string())
}

/// Check if a string contains valid Ruchy syntax.
///
/// # Examples
///
/// ```
/// use ruchy::is_valid_syntax;
///
/// assert!(is_valid_syntax("1 + 2"));
/// assert!(is_valid_syntax("fun add(x, y) { x + y }"));
/// assert!(!is_valid_syntax("let x = ;"));  // Missing value
/// ```
pub fn is_valid_syntax(input: &str) -> bool {
    let mut parser = Parser::new(input);
    parser.parse().is_ok()
}

/// Get a formatted error message for invalid Ruchy code.
///
/// # Examples
///
/// ```
/// use ruchy::get_parse_error;
///
/// match get_parse_error("let x = ;") {
///     Some(error) => println!("Error: {}", error),
///     None => println!("No error"),
/// }
/// ```
pub fn get_parse_error(input: &str) -> Option<String> {
    let mut parser = Parser::new(input);
    match parser.parse() {
        Ok(_) => None,
        Err(e) => Some(e.to_string()),
    }
}
