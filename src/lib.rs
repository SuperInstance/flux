//! FLUX — Fluid Language Universal eXecution
//!
//! This is the top-level crate for the FLUX runtime, re-exporting all
//! sub-crates for convenient access.
//!
//! # Crates
//!
//! - [`flux_bytecode`] — Opcodes, encoder, decoder, validator (100+ opcodes)
//! - [`flux_vm`] — 64-register virtual machine interpreter
//! - [`flux_fir`] — SSA intermediate representation builder & validator
//! - [`flux_parser`] — FLUX.MD parser and AST-to-FIR compiler

pub use flux_bytecode;
pub use flux_fir;
pub use flux_parser;
pub use flux_vm;
