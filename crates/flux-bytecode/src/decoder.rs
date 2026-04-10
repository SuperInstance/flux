//! FLUX bytecode decoder — converts raw bytes into structured instructions.
//!
//! The decoder reads a byte buffer produced by the encoder, first parsing the
//! 18-byte [`BytecodeHeader`], then decoding each instruction sequentially.

use std::io::{Cursor, Read};

use crate::error::DecodeError;
use crate::encoder::Instruction;
use crate::format::InstrFormat;
use crate::header::{BytecodeHeader, HEADER_SIZE};
use crate::opcodes::Op;

/// A fully decoded FLUX bytecode module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedModule {
    /// The file header.
    pub header: BytecodeHeader,
    /// The list of decoded instructions.
    pub instructions: Vec<Instruction>,
}

/// Decodes FLUX bytecode bytes into structured instructions.
#[derive(Debug, Clone)]
pub struct BytecodeDecoder<'a> {
    /// Read cursor over the input bytes.
    cursor: Cursor<&'a [u8]>,
    /// Byte offset of the last decoded instruction (for error reporting).
    offset: usize,
}

impl<'a> BytecodeDecoder<'a> {
    /// Creates a new decoder wrapping the given byte slice.
    #[must_use]
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(bytes),
            offset: 0,
        }
    }

    /// Returns the current read position in the byte stream.
    #[must_use]
    pub fn position(&self) -> usize {
        self.cursor.position() as usize
    }

    /// Returns `true` if there are no more bytes to read.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cursor.position() as usize >= self.cursor.get_ref().len()
    }

    /// Decodes the 18-byte file header from the beginning of the stream.
    ///
    /// The cursor is advanced past the header.
    ///
    /// # Errors
    ///
    /// Returns [`DecodeError`] if the bytes are too short, the magic is
    /// incorrect, or the version is unsupported.
    pub fn decode_header(&mut self) -> Result<BytecodeHeader, DecodeError> {
        let remaining = &self.cursor.get_ref()[self.cursor.position() as usize..];
        let header = BytecodeHeader::from_bytes(remaining)?;
        self.cursor.set_position(HEADER_SIZE as u64);
        self.offset = HEADER_SIZE;
        Ok(header)
    }

    /// Decodes a single instruction from the current cursor position.
    ///
    /// Returns `Ok(None)` when the end of the byte stream is reached.
    ///
    /// # Errors
    ///
    /// Returns [`DecodeError`] on malformed input.
    pub fn decode_instruction(&mut self) -> Result<Option<Instruction>, DecodeError> {
        // Read the opcode byte.
        let opcode_byte = match read_u8(&mut self.cursor) {
            Ok(b) => b,
            Err(_) => return Ok(None), // EOF — no more instructions.
        };

        let op = Op::try_from(opcode_byte)
            .map_err(|b| DecodeError::InvalidOpcode { byte: b })?;

        let fmt = op.format();
        let instr_offset = self.offset;

        let instr = match fmt {
            InstrFormat::A => Instruction::nullary(op),

            InstrFormat::B => {
                let dst = read_u8(&mut self.cursor).map_err(|_| DecodeError::UnexpectedEof {
                    expected: 1,
                    available: 0,
                })?;
                let src = read_u8(&mut self.cursor).map_err(|_| DecodeError::UnexpectedEof {
                    expected: 1,
                    available: 0,
                })?;
                Instruction::reg(op, dst, src)
            }

            InstrFormat::C => {
                let dst = read_u8(&mut self.cursor).map_err(|_| DecodeError::UnexpectedEof {
                    expected: 3,
                    available: 0,
                })?;
                let src = read_u8(&mut self.cursor).map_err(|_| DecodeError::UnexpectedEof {
                    expected: 2,
                    available: 0,
                })?;
                let type_tag = read_u8(&mut self.cursor).map_err(|_| DecodeError::UnexpectedEof {
                    expected: 1,
                    available: 0,
                })?;
                Instruction::reg_ty(op, dst, src, type_tag)
            }

            InstrFormat::D => {
                let dst = read_u8(&mut self.cursor).map_err(|_| DecodeError::UnexpectedEof {
                    expected: 3,
                    available: 0,
                })?;
                let imm_lo = read_u8(&mut self.cursor).map_err(|_| DecodeError::UnexpectedEof {
                    expected: 2,
                    available: 0,
                })?;
                let imm_hi = read_u8(&mut self.cursor).map_err(|_| DecodeError::UnexpectedEof {
                    expected: 1,
                    available: 0,
                })?;
                let immediate = i16::from_le_bytes([imm_lo, imm_hi]);
                Instruction::imm(op, dst, immediate)
            }

            InstrFormat::E => {
                let dst = read_u8(&mut self.cursor).map_err(|_| DecodeError::UnexpectedEof {
                    expected: 4,
                    available: 0,
                })?;
                let src = read_u8(&mut self.cursor).map_err(|_| DecodeError::UnexpectedEof {
                    expected: 3,
                    available: 0,
                })?;
                let off_lo = read_u8(&mut self.cursor).map_err(|_| DecodeError::UnexpectedEof {
                    expected: 2,
                    available: 0,
                })?;
                let off_hi = read_u8(&mut self.cursor).map_err(|_| DecodeError::UnexpectedEof {
                    expected: 1,
                    available: 0,
                })?;
                let offset = u16::from_le_bytes([off_lo, off_hi]);
                Instruction::mem(op, dst, src, offset)
            }

            InstrFormat::G => {
                let payload_len = read_u8(&mut self.cursor).map_err(|_| DecodeError::UnexpectedEof {
                    expected: 1,
                    available: 0,
                })?;
                let remaining = &self.cursor.get_ref()[self.cursor.position() as usize..];
                if (payload_len as usize) > remaining.len() {
                    return Err(DecodeError::InvalidPayloadLength { len: payload_len });
                }
                let mut payload = Vec::with_capacity(payload_len as usize);
                for _ in 0..payload_len {
                    let b = read_u8(&mut self.cursor).unwrap();
                    payload.push(b);
                }
                Instruction::var(op, payload)
            }
        };

        self.offset = instr_offset + fmt.total_size(if fmt == InstrFormat::G {
            instr.payload.len()
        } else {
            0
        });

        Ok(Some(instr))
    }

    /// Decodes the entire module: header followed by all instructions.
    ///
    /// # Errors
    ///
    /// Returns [`DecodeError`] on any malformed input.
    pub fn decode_all(&mut self) -> Result<DecodedModule, DecodeError> {
        let header = self.decode_header()?;

        let mut instructions = Vec::new();
        while let Some(instr) = self.decode_instruction()? {
            instructions.push(instr);
        }

        Ok(DecodedModule {
            header,
            instructions,
        })
    }
}

/// Reads a single byte from the cursor, returning an error on EOF.
fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, ()> {
    let mut buf = [0u8; 1];
    match cursor.read_exact(&mut buf) {
        Ok(()) => Ok(buf[0]),
        Err(_) => Err(()),
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_halt() {
        let bytes: &[u8] = &[0x00];
        let mut dec = BytecodeDecoder::new(bytes);
        let instr = dec.decode_instruction().unwrap().unwrap();
        assert_eq!(instr, Instruction::nullary(Op::Halt));
        assert!(dec.decode_instruction().unwrap().is_none());
    }

    #[test]
    fn decode_push() {
        let bytes: &[u8] = &[0x10, 0x01, 0x02];
        let mut dec = BytecodeDecoder::new(bytes);
        let instr = dec.decode_instruction().unwrap().unwrap();
        assert_eq!(instr, Instruction::reg(Op::Push, 1, 2));
    }

    #[test]
    fn decode_iadd() {
        let bytes: &[u8] = &[0x21, 0x00, 0x01, 0x03];
        let mut dec = BytecodeDecoder::new(bytes);
        let instr = dec.decode_instruction().unwrap().unwrap();
        assert_eq!(instr, Instruction::reg_ty(Op::IAdd, 0, 1, 3));
    }

    #[test]
    fn decode_iinc() {
        let bytes: &[u8] = &[0x28, 0x05, 0xFF, 0xFF];
        let mut dec = BytecodeDecoder::new(bytes);
        let instr = dec.decode_instruction().unwrap().unwrap();
        assert_eq!(instr, Instruction::imm(Op::IInc, 5, -1));
    }

    #[test]
    fn decode_call() {
        let bytes: &[u8] = &[0x06, 0x02, 0x05, 0x00];
        let mut dec = BytecodeDecoder::new(bytes);
        let instr = dec.decode_instruction().unwrap().unwrap();
        assert_eq!(instr, Instruction::var(Op::Call, vec![0x05, 0x00]));
    }

    #[test]
    fn decode_empty() {
        let bytes: &[u8] = &[];
        let mut dec = BytecodeDecoder::new(bytes);
        assert!(dec.decode_instruction().unwrap().is_none());
    }

    #[test]
    fn decode_invalid_opcode() {
        let bytes: &[u8] = &[0x0B]; // undefined in 0x00-0x0A range
        let mut dec = BytecodeDecoder::new(bytes);
        let result = dec.decode_instruction();
        assert!(matches!(result, Err(DecodeError::InvalidOpcode { byte: 0x0B })));
    }
}
