pub mod vm;
pub mod memory;

#[cfg(feature = "dataframe")]
pub mod dataframe;

#[cfg(feature = "native")]
pub mod server;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use vm::{OpCode, BytecodeModule, VirtualMachine, ExecutionResult};
pub use memory::{Arena, ArenaRef, SlabAllocator, SlabHandle};
