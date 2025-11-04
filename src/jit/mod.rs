//! JIT Compilation Module for Ruchy
//!
//! JIT-001: Proof of Concept - Cranelift-based Just-In-Time compilation
//!
//! # Performance Goals
//! - fibonacci(20): <0.5ms (vs 19ms AST interpreter - 38x speedup minimum)
//! - Compilation overhead: <10ms per function
//! - Expected speedup: 50-100x over AST interpreter
//!
//! # Architecture
//! ```
//! Ruchy AST → Cranelift IR → Native Machine Code → Execute
//! ```
//!
//! # Implementation Phases
//! - JIT-001 (Current): Proof of concept - arithmetic expressions
//! - JIT-002: Full language support (control flow, data structures)
//! - JIT-003: Tiered optimization with profiling
//!
//! # References
//! - GitHub Issue #135: https://github.com/paiml/ruchy/issues/135
//! - Cranelift Docs: https://cranelift.dev/
//! - SimpleJIT Example: https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/simplejit

#![cfg(feature = "jit")]

pub mod compiler;
pub mod lowering;

pub use compiler::JitCompiler;

use anyhow::Result;
use crate::frontend::ast::Expr;

/// JIT compile and execute a Ruchy expression
///
/// # Example
/// ```ignore
/// let ast = parse("2 + 2");
/// let result = jit_execute(&ast)?;
/// assert_eq!(result, 4);
/// ```
pub fn jit_execute(ast: &Expr) -> Result<i64> {
    let mut compiler = JitCompiler::new()?;
    compiler.compile_and_execute(ast)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::Parser;

    #[test]
    #[ignore] // Enable once JIT implementation is complete
    fn test_jit_simple_arithmetic() {
        let code = "2 + 2";
        let ast = Parser::new(code).parse().unwrap();
        let result = jit_execute(&ast).unwrap();
        assert_eq!(result, 4);
    }

    #[test]
    #[ignore] // Enable once JIT implementation is complete
    fn test_jit_fibonacci_10() {
        let code = r"
            fun fib(n: i32) -> i32 {
                if n <= 1 {
                    n
                } else {
                    fib(n - 1) + fib(n - 2)
                }
            }
            fib(10)
        ";
        let ast = Parser::new(code).parse().unwrap();
        let result = jit_execute(&ast).unwrap();
        assert_eq!(result, 55); // fibonacci(10) = 55
    }
}
