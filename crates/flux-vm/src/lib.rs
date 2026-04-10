//! FLUX virtual machine.
//!
//! Re-exports all public modules.

pub mod error;
pub mod interpreter;
pub mod memory;
pub mod registers;

pub use error::{MemError, VmError};
pub use interpreter::{DecodedInstr, Interpreter, StepResult, VmConfig};
pub use memory::{MemoryManager, MemoryRegion, Permissions, RegionId, CODE_BASE, STACK_BASE};
pub use registers::{FlagBits, RegisterFile};
