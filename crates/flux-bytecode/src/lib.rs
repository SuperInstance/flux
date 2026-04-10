//! FLUX bytecode: opcodes, encoder, decoder, validator.
//!
//! This crate defines the FLUX bytecode format including 100+ opcodes across
//! 6 encoding formats, a binary encoder/decoder, and a bytecode validator.

mod error;
mod format;
mod header;
mod encoder;
mod decoder;
mod validator;
mod opcodes;

pub use error::*;
pub use format::*;
pub use header::*;
pub use encoder::*;
pub use decoder::*;
pub use validator::*;
pub use opcodes::*;
