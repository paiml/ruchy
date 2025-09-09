pub mod vm;

#[cfg(feature = "native")]
pub mod server;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use vm::{OpCode, BytecodeModule, VirtualMachine, ExecutionResult};
