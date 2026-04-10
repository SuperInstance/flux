//! FLUX virtual-machine errors.

/// Errors produced by the memory manager.
#[derive(Debug, Clone, thiserror::Error)]
pub enum MemError {
    #[error("address 0x{addr:016X} out of bounds")]
    AddressOutOfBounds { addr: u64 },
    #[error("permission denied for address 0x{addr:016X}")]
    PermissionDenied { addr: u64 },
    #[error("read of {size} bytes at 0x{addr:016X} crosses region boundary")]
    ReadOutOfBounds { addr: u64, size: usize },
    #[error("write of {size} bytes at 0x{addr:016X} crosses region boundary")]
    WriteOutOfBounds { addr: u64, size: usize },
    #[error("region not found: {0:?}")]
    RegionNotFound(u32),
    #[error("region already freed: {0:?}")]
    RegionFreed(u32),
}

/// Errors produced by the interpreter.
#[derive(Debug, thiserror::Error)]
pub enum VmError {
    #[error("cycle limit reached: {0}")]
    CycleLimit(u64),
    #[error("invalid register: {0}")]
    InvalidRegister(u8),
    #[error("memory error: {0}")]
    Memory(#[from] MemError),
    #[error("invalid opcode: 0x{0:02x}")]
    InvalidOpcode(u8),
    #[error("division by zero")]
    DivisionByZero,
    #[error("stack overflow")]
    StackOverflow,
    #[error("stack underflow")]
    StackUnderflow,
    #[error("panic: {0}")]
    Panic(String),
    #[error("execution error: {0}")]
    Execution(String),
}
