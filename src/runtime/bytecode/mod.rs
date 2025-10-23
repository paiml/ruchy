//! Bytecode Virtual Machine
//!
//! OPT-001: Bytecode VM Foundation
//!
//! This module implements a register-based bytecode VM for Ruchy.
//! Expected performance improvements: 40-60% faster than AST walking,
//! with 30-40% memory reduction.
//!
//! # Architecture
//!
//! - **Register-based**: 32 general-purpose registers per frame
//! - **32-bit fixed-width instructions**: Efficient fetch and decode
//! - **6-bit opcodes**: Support for up to 64 operations
//! - **4 instruction formats**: ABC, ABx, AsBx, Ax for different operand types
//!
//! # Performance Targets (from ruchyruchy research)
//!
//! - 40-60% execution time reduction vs AST walking
//! - 25-30% fewer instructions vs stack-based VM
//! - 30-40% memory usage reduction
//! - 50-60% reduction in cache misses
//!
//! # References
//!
//! - ../ruchyruchy/OPTIMIZATION_REPORT_FOR_RUCHY.md
//! - Würthinger et al. (2017) - One VM to Rule Them All
//! - Brunthaler (2010) - Inline Caching Meets Quickening
//! - Gal et al. (2009) - Trace-based Just-in-Time Type Specialization

pub mod compiler;
pub mod instruction;
pub mod opcode;
pub mod vm;

pub use compiler::{BytecodeChunk, Compiler};
pub use instruction::{Instruction, InstructionFormat};
pub use opcode::OpCode;
pub use vm::VM;
