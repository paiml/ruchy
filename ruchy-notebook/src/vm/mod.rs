pub mod bytecode;
pub mod interpreter;
pub mod compiler;

pub use bytecode::{OpCode, BytecodeModule, Instruction};
pub use interpreter::{VirtualMachine, ExecutionResult};
pub use compiler::Compiler;