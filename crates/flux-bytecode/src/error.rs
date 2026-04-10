//! Error types for the FLUX bytecode encoder, decoder, and validator.

use std::fmt;

/// Errors that can occur during bytecode encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EncodeError {
    /// The payload length exceeds the maximum allowed (255 bytes).
    PayloadTooLong {
        /// The actual length of the payload.
        len: usize,
        /// The maximum allowed payload length.
        max: usize,
    },
    /// An invalid register ID was provided (must be < 64).
    InvalidRegister {
        /// The invalid register ID.
        reg: u8,
    },
    /// An immediate value is out of the representable range.
    ImmediateOutOfRange {
        /// The out-of-range value.
        value: i32,
    },
    /// An offset value is out of the representable range.
    OffsetOutOfRange {
        /// The out-of-range value.
        value: u32,
    },
    /// An instruction has a mismatched set of operands for its format.
    FormatMismatch {
        /// A description of the mismatch.
        detail: String,
    },
    /// The opcode has no defined instruction format.
    UnknownFormat {
        /// The opcode byte that has no format.
        opcode: u8,
    },
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::PayloadTooLong { len, max } => {
                write!(f, "payload length {} exceeds maximum {}", len, max)
            }
            EncodeError::InvalidRegister { reg } => {
                write!(f, "invalid register ID {} (must be < 64)", reg)
            }
            EncodeError::ImmediateOutOfRange { value } => {
                write!(f, "immediate value {} out of i16 range", value)
            }
            EncodeError::OffsetOutOfRange { value } => {
                write!(f, "offset value {} out of u16 range", value)
            }
            EncodeError::FormatMismatch { detail } => {
                write!(f, "format mismatch: {}", detail)
            }
            EncodeError::UnknownFormat { opcode } => {
                write!(f, "opcode 0x{:02X} has no defined instruction format", opcode)
            }
        }
    }
}

impl std::error::Error for EncodeError {}

/// Errors that can occur during bytecode decoding.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DecodeError {
    /// The magic bytes at the start of the file are not `b"FLUX"`.
    InvalidMagic {
        /// The actual bytes found.
        found: [u8; 4],
    },
    /// An unrecognized opcode byte was encountered.
    InvalidOpcode {
        /// The invalid opcode byte.
        byte: u8,
    },
    /// The input ended unexpectedly while reading an instruction.
    UnexpectedEof {
        /// How many bytes were expected.
        expected: usize,
        /// How many bytes were actually available.
        available: usize,
    },
    /// An instruction's operands are inconsistent with its declared format.
    InvalidFormat {
        /// A description of the inconsistency.
        detail: String,
    },
    /// A register ID in the bytecode is out of the valid range.
    InvalidRegister {
        /// The invalid register ID.
        reg: u8,
    },
    /// The payload length in a format-G instruction would read past the end of input.
    InvalidPayloadLength {
        /// The declared payload length.
        len: u8,
    },
    /// An unsupported bytecode version.
    UnsupportedVersion {
        /// The version number found.
        version: u16,
    },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::InvalidMagic { found } => {
                write!(
                    f,
                    "invalid magic: expected b\"FLUX\", found {:?}",
                    found as &[u8]
                )
            }
            DecodeError::InvalidOpcode { byte } => {
                write!(f, "invalid opcode byte 0x{:02X}", byte)
            }
            DecodeError::UnexpectedEof {
                expected,
                available,
            } => {
                write!(f, "unexpected EOF: expected {} bytes, got {}", expected, available)
            }
            DecodeError::InvalidFormat { detail } => {
                write!(f, "invalid instruction format: {}", detail)
            }
            DecodeError::InvalidRegister { reg } => {
                write!(f, "invalid register ID {} (must be < 64)", reg)
            }
            DecodeError::InvalidPayloadLength { len } => {
                write!(f, "invalid payload length {}", len)
            }
            DecodeError::UnsupportedVersion { version } => {
                write!(f, "unsupported bytecode version {}", version)
            }
        }
    }
}

impl std::error::Error for DecodeError {}

/// A validation error found in a decoded bytecode module.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ValidationError {
    /// The header magic is not `b"FLUX"`.
    InvalidMagic {
        /// The actual bytes found.
        found: [u8; 4],
    },
    /// An unrecognized opcode byte was found.
    InvalidOpcode {
        /// The byte offset in the instruction stream.
        offset: usize,
        /// The invalid opcode byte.
        byte: u8,
    },
    /// A register ID exceeds the maximum (63).
    InvalidRegister {
        /// The byte offset in the instruction stream.
        offset: usize,
        /// The invalid register ID.
        reg: u8,
    },
    /// An immediate value exceeds the i16 range.
    ImmediateOutOfRange {
        /// The byte offset in the instruction stream.
        offset: usize,
        /// The out-of-range value.
        value: i32,
    },
    /// Instructions appear after a terminator (Halt/Ret/Panic/Unreachable) in a function body.
    InstructionsAfterTerminator {
        /// The byte offset of the terminator.
        terminator_offset: usize,
        /// The byte offset of the first instruction after it.
        after_offset: usize,
    },
    /// A function body has no terminator instruction.
    MissingTerminator {
        /// The zero-based function index.
        function_index: u32,
    },
    /// The payload length would overflow the remaining bytecode.
    PayloadOverflow {
        /// The byte offset in the instruction stream.
        offset: usize,
        /// The declared payload length.
        len: u8,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidMagic { found } => {
                write!(
                    f,
                    "invalid magic: expected b\"FLUX\", found {:?}",
                    found as &[u8]
                )
            }
            ValidationError::InvalidOpcode { offset, byte } => {
                write!(f, "invalid opcode 0x{:02X} at offset {}", byte, offset)
            }
            ValidationError::InvalidRegister { offset, reg } => {
                write!(f, "invalid register {} at offset {}", reg, offset)
            }
            ValidationError::ImmediateOutOfRange { offset, value } => {
                write!(f, "immediate {} out of range at offset {}", value, offset)
            }
            ValidationError::InstructionsAfterTerminator {
                terminator_offset,
                after_offset,
            } => {
                write!(
                    f,
                    "instructions after terminator at offset {} (continues at {})",
                    terminator_offset, after_offset
                )
            }
            ValidationError::MissingTerminator { function_index } => {
                write!(f, "function {} has no terminator", function_index)
            }
            ValidationError::PayloadOverflow { offset, len } => {
                write!(f, "payload of length {} overflows at offset {}", len, offset)
            }
        }
    }
}

impl std::error::Error for ValidationError {}
