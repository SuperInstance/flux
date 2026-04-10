//! FLUX FIR — SSA intermediate representation for the FLUX language.
//!
//! This crate provides the core data structures and builders for constructing
//! and validating FIR (FLUX Intermediate Representation) programs in SSA form.

pub mod blocks;
pub mod builder;
pub mod instructions;
pub mod types;
pub mod values;
pub mod validator;

// Re-export key types at the crate root
pub use blocks::{BasicBlock, BlockId, FirFunction, FirModule, Terminator};
pub use builder::FirBuilder;
pub use instructions::{CmpOp, Instruction};
pub use types::{FirType, TypeContext};
pub use values::{Constant, Value};
pub use validator::{FirValidator, ValidationError};
