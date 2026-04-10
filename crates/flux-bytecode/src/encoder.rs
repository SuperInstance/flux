//! FLUX bytecode encoder — converts structured instructions into raw bytes.
//!
//! The encoder writes a sequence of [`Instruction`] values into a byte buffer,
//! then prepends the 18-byte file header when [`BytecodeEncoder::finish`] is
//! called.

use crate::error::EncodeError;
use crate::format::InstrFormat;
use crate::header::{BytecodeHeader, HEADER_SIZE};
use crate::opcodes::Op;

/// Maximum allowed payload length for format-G instructions (255 bytes).
pub const MAX_PAYLOAD_LEN: usize = 255;

/// Maximum valid register ID (inclusive).
pub const MAX_REGISTER: u8 = 63;

/// A single FLUX bytecode instruction with all possible operand fields.
///
/// Not every field is used by every instruction — which fields are relevant
/// depends on the opcode's [`InstrFormat`]:
///
/// | Format | Fields used                          |
/// |--------|--------------------------------------|
/// | A      | `op` only                            |
/// | B      | `op`, `dst`, `src`                   |
/// | C      | `op`, `dst`, `src`, `type_tag`       |
/// | D      | `op`, `dst`, `immediate`             |
/// | E      | `op`, `dst`, `src`, `offset`         |
/// | G      | `op`, `payload`                      |
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    /// The opcode.
    pub op: Op,
    /// Destination register (used by formats B, C, D, E).
    pub dst: u8,
    /// Source register (used by formats B, C, E).
    pub src: u8,
    /// Type tag byte (used by format C).
    pub type_tag: u8,
    /// 16-bit signed immediate (used by format D).
    pub immediate: i16,
    /// 16-bit unsigned offset (used by format E).
    pub offset: u16,
    /// Variable-length payload (used by format G).
    pub payload: Vec<u8>,
}

impl Instruction {
    /// Creates a format-A (nullary) instruction.
    #[must_use]
    pub const fn nullary(op: Op) -> Self {
        Self {
            op,
            dst: 0,
            src: 0,
            type_tag: 0,
            immediate: 0,
            offset: 0,
            payload: Vec::new(),
        }
    }

    /// Creates a format-B instruction (two registers).
    #[must_use]
    pub const fn reg(op: Op, dst: u8, src: u8) -> Self {
        Self {
            op,
            dst,
            src,
            type_tag: 0,
            immediate: 0,
            offset: 0,
            payload: Vec::new(),
        }
    }

    /// Creates a format-C instruction (two registers + type tag).
    #[must_use]
    pub const fn reg_ty(op: Op, dst: u8, src: u8, type_tag: u8) -> Self {
        Self {
            op,
            dst,
            src,
            type_tag,
            immediate: 0,
            offset: 0,
            payload: Vec::new(),
        }
    }

    /// Creates a format-D instruction (register + 16-bit immediate).
    #[must_use]
    pub const fn imm(op: Op, dst: u8, immediate: i16) -> Self {
        Self {
            op,
            dst,
            src: 0,
            type_tag: 0,
            immediate,
            offset: 0,
            payload: Vec::new(),
        }
    }

    /// Creates a format-E instruction (two registers + 16-bit offset).
    #[must_use]
    pub const fn mem(op: Op, dst: u8, src: u8, offset: u16) -> Self {
        Self {
            op,
            dst,
            src,
            offset,
            type_tag: 0,
            immediate: 0,
            payload: Vec::new(),
        }
    }

    /// Creates a format-G instruction (variable-length payload).
    #[must_use]
    pub fn var(op: Op, payload: Vec<u8>) -> Self {
        Self {
            op,
            dst: 0,
            src: 0,
            type_tag: 0,
            immediate: 0,
            offset: 0,
            payload,
        }
    }
}

/// Encodes FLUX bytecode instructions into a byte buffer.
#[derive(Debug, Clone, Default)]
pub struct BytecodeEncoder {
    /// The instruction byte buffer (header is prepended at finish time).
    buf: Vec<u8>,
}

impl BytecodeEncoder {
    /// Creates a new empty encoder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(1024),
        }
    }

    /// Creates a new encoder with a pre-allocated capacity hint.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
        }
    }

    /// Returns the current number of instruction bytes (excluding the header).
    #[must_use]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Returns `true` if no instructions have been emitted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Emits a single instruction into the buffer.
    ///
    /// # Errors
    ///
    /// Returns an [`EncodeError`] if:
    /// - A register ID is ≥ 64
    /// - A payload exceeds 255 bytes
    /// - The opcode has no defined format
    pub fn emit(&mut self, instr: &Instruction) -> Result<(), EncodeError> {
        let fmt = instr.op.format();

        // ── Validate register IDs ────────────────────────────────────
        match fmt {
            InstrFormat::B | InstrFormat::C | InstrFormat::E => {
                check_reg(instr.dst)?;
                check_reg(instr.src)?;
            }
            InstrFormat::D => {
                check_reg(instr.dst)?;
            }
            InstrFormat::A | InstrFormat::G => {}
        }

        // ── Write opcode byte ───────────────────────────────────────
        self.buf.push(instr.op.byte());

        // ── Write operand bytes per format ───────────────────────────
        match fmt {
            InstrFormat::A => {
                // No operands.
            }
            InstrFormat::B => {
                self.buf.push(instr.dst);
                self.buf.push(instr.src);
            }
            InstrFormat::C => {
                self.buf.push(instr.dst);
                self.buf.push(instr.src);
                self.buf.push(instr.type_tag);
            }
            InstrFormat::D => {
                self.buf.push(instr.dst);
                let imm_bytes = instr.immediate.to_le_bytes();
                self.buf.extend_from_slice(&imm_bytes);
            }
            InstrFormat::E => {
                self.buf.push(instr.dst);
                self.buf.push(instr.src);
                let off_bytes = instr.offset.to_le_bytes();
                self.buf.extend_from_slice(&off_bytes);
            }
            InstrFormat::G => {
                let len = instr.payload.len();
                if len > MAX_PAYLOAD_LEN {
                    return Err(EncodeError::PayloadTooLong {
                        len,
                        max: MAX_PAYLOAD_LEN,
                    });
                }
                self.buf.push(len as u8);
                self.buf.extend_from_slice(&instr.payload);
            }
        }

        Ok(())
    }

    /// Finalizes the bytecode by prepending the header and returning the
    /// complete byte vector.
    ///
    /// # Errors
    ///
    /// Returns [`EncodeError`] if the header cannot be serialized.
    pub fn finish(self, header: BytecodeHeader) -> Result<Vec<u8>, EncodeError> {
        let header_bytes = header.to_bytes()?;
        let total = HEADER_SIZE + self.buf.len();
        let mut out = Vec::with_capacity(total);
        out.extend_from_slice(&header_bytes);
        out.extend_from_slice(&self.buf);
        Ok(out)
    }

    /// Consumes the encoder and returns the raw instruction bytes
    /// (without any header) for testing or partial use.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }
}

/// Checks that a register ID is within the valid range (< 64).
fn check_reg(reg: u8) -> Result<(), EncodeError> {
    if reg >= MAX_REGISTER {
        Err(EncodeError::InvalidRegister { reg })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit_halt() {
        let mut enc = BytecodeEncoder::new();
        enc.emit(&Instruction::nullary(Op::Halt)).unwrap();
        let bytes = enc.into_bytes();
        assert_eq!(bytes, vec![0x00]);
    }

    #[test]
    fn emit_push() {
        let mut enc = BytecodeEncoder::new();
        enc.emit(&Instruction::reg(Op::Push, 1, 2)).unwrap();
        let bytes = enc.into_bytes();
        assert_eq!(bytes, vec![0x10, 0x01, 0x02]);
    }

    #[test]
    fn emit_iadd() {
        let mut enc = BytecodeEncoder::new();
        enc.emit(&Instruction::reg_ty(Op::IAdd, 0, 1, 0x02)).unwrap();
        let bytes = enc.into_bytes();
        assert_eq!(bytes, vec![0x21, 0x00, 0x01, 0x02]);
    }

    #[test]
    fn emit_iinc() {
        let mut enc = BytecodeEncoder::new();
        enc.emit(&Instruction::imm(Op::IInc, 5, -1)).unwrap();
        let bytes = enc.into_bytes();
        assert_eq!(bytes, vec![0x28, 0x05, 0xFF, 0xFF]); // -1 in LE i16
    }

    #[test]
    fn emit_load32() {
        let mut enc = BytecodeEncoder::new();
        enc.emit(&Instruction::mem(Op::Load32, 3, 7, 256)).unwrap();
        let bytes = enc.into_bytes();
        assert_eq!(bytes, vec![0x72, 0x03, 0x07, 0x00, 0x01]); // 256 = 0x0100 LE
    }

    #[test]
    fn emit_call() {
        let mut enc = BytecodeEncoder::new();
        enc.emit(&Instruction::var(Op::Call, vec![0x05, 0x00])).unwrap();
        let bytes = enc.into_bytes();
        assert_eq!(bytes, vec![0x06, 0x02, 0x05, 0x00]);
    }

    #[test]
    fn invalid_register() {
        let mut enc = BytecodeEncoder::new();
        let result = enc.emit(&Instruction::reg(Op::Push, 64, 0));
        assert!(matches!(result, Err(EncodeError::InvalidRegister { reg: 64 })));
    }

    #[test]
    fn payload_too_long() {
        let mut enc = BytecodeEncoder::new();
        let result = enc.emit(&Instruction::var(Op::Call, vec![0u8; 256]));
        assert!(matches!(result, Err(EncodeError::PayloadTooLong { .. })));
    }
}
