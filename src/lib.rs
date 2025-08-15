//! Ruchy - A systems-oriented scripting language that transpiles to Rust
//!
//! Ruchy provides a high-level, expressive syntax with features like pipeline operators,
//! pattern matching, and actor-based concurrency, all while transpiling to efficient Rust code.

#![cfg_attr(test, allow(clippy::unwrap_used))]
#![cfg_attr(test, allow(clippy::expect_used))]
#![cfg_attr(test, allow(clippy::panic))]
#![allow(clippy::missing_errors_doc)] // Many functions return Result, docs would be repetitive
#![allow(clippy::unnecessary_wraps)] // Some functions need Result for consistency
#![allow(clippy::match_same_arms)] // Sometimes clearer to show all arms explicitly
#![allow(clippy::needless_pass_by_value)] // Some APIs take owned values for consistency
#![allow(clippy::manual_let_else)] // Sometimes if-else is clearer than let-else
#![allow(clippy::unused_self)] // Some methods need self for consistency in API
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
//! ```no_run
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
//!     match x {
//!         1 => "one",
//!         2 => "two",
//!         _ => "other",
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

#[cfg(test)]
pub mod testing;

// Re-export commonly used types
pub use backend::transpiler::Transpiler;
pub use frontend::{
    ast::{Expr, ExprKind},
    error_recovery::RecoveryParser,
    parser::Parser,
};
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
/// assert!(is_valid_syntax("fun add(x: i32, y: i32) -> i32 { x + y }"));
/// assert!(!is_valid_syntax("let x = ;"));  // Missing value
/// ```
#[must_use] pub fn is_valid_syntax(input: &str) -> bool {
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
#[must_use] pub fn get_parse_error(input: &str) -> Option<String> {
    let mut parser = Parser::new(input);
    match parser.parse() {
        Ok(_) => None,
        Err(e) => Some(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_expressions() {
        // Test basic arithmetic
        let result = compile("1 + 2").unwrap();
        assert!(result.contains('1'));
        assert!(result.contains('2'));
        
        let result = compile("42 * 3.14").unwrap();
        assert!(result.contains("42"));
        assert!(result.contains("3.14"));
    }

    #[test]
    fn test_compile_functions() {
        let result = compile("fun add(x: i32, y: i32) -> i32 { x + y }").unwrap();
        assert!(result.contains("fn"));
        assert!(result.contains("add"));
        assert!(result.contains("i32"));
    }

    #[test]
    fn test_compile_errors() {
        assert!(compile("").is_err());
        assert!(compile("   ").is_err());
        assert!(compile("let x =").is_err());
        assert!(compile("fun ()").is_err());
        assert!(compile("if").is_err());
    }

    #[test]
    fn test_is_valid_syntax_valid_cases() {
        assert!(is_valid_syntax("42"));
        assert!(is_valid_syntax("3.14"));
        assert!(is_valid_syntax("true"));
        assert!(is_valid_syntax("false"));
        assert!(is_valid_syntax("\"hello\""));
        assert!(is_valid_syntax("x + y"));
        assert!(is_valid_syntax("[1, 2, 3]"));
        assert!(is_valid_syntax("if true { 1 } else { 2 }"));
    }

    #[test]
    fn test_is_valid_syntax_invalid_cases() {
        assert!(!is_valid_syntax(""));
        assert!(!is_valid_syntax("   "));
        assert!(!is_valid_syntax("let x ="));
        assert!(!is_valid_syntax("fun ()"));
        assert!(!is_valid_syntax("if { }"));
        assert!(!is_valid_syntax("[1, 2,"));
        assert!(!is_valid_syntax("match"));
    }

    #[test]
    fn test_get_parse_error_with_errors() {
        let error = get_parse_error("let x =");
        assert!(error.is_some());
        assert!(!error.unwrap().is_empty());

        let error = get_parse_error("(");
        assert!(error.is_some());
        
        let error = get_parse_error("}");
        assert!(error.is_some());
    }

    #[test]
    fn test_get_parse_error_without_errors() {
        assert!(get_parse_error("42").is_none());
        assert!(get_parse_error("let x = 42").is_none());
        assert!(get_parse_error("fun foo() { 42 }").is_none());
        assert!(get_parse_error("[1, 2, 3]").is_none());
    }

    #[test]
    fn test_compile_complex_expressions() {
        let result = compile("let x = 42 in x + 1");
        assert!(result.is_ok());
        
        let result = compile("if x > 0 { x } else { -x }");
        assert!(result.is_ok());
        
        // Pipeline with lambda not yet fully supported
        // let result = compile("[1, 2, 3] |> map(x => x * 2)");
        // assert!(result.is_ok());
        
        // Test simpler pipeline
        let result = compile("data |> filter |> map");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_all_literal_types() {
        assert!(compile("42").is_ok());
        assert!(compile("3.14159").is_ok());
        assert!(compile("true").is_ok());
        assert!(compile("false").is_ok());
        assert!(compile("\"hello world\"").is_ok());
        // Unit type not yet supported in parser
        // assert!(compile("()").is_ok());
    }

    #[test]
    fn test_compile_all_operators() {
        assert!(compile("1 + 2").is_ok());
        assert!(compile("3 - 1").is_ok());
        assert!(compile("2 * 3").is_ok());
        assert!(compile("6 / 2").is_ok());
        assert!(compile("5 % 2").is_ok());
        assert!(compile("2 ** 3").is_ok());
        
        assert!(compile("1 < 2").is_ok());
        assert!(compile("2 > 1").is_ok());
        assert!(compile("1 <= 2").is_ok());
        assert!(compile("2 >= 1").is_ok());
        assert!(compile("1 == 1").is_ok());
        assert!(compile("1 != 2").is_ok());
        
        assert!(compile("true && false").is_ok());
        assert!(compile("true || false").is_ok());
        assert!(compile("!true").is_ok());
    }
}
