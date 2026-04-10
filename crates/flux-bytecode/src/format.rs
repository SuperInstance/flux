//! Instruction encoding formats for the FLUX bytecode virtual machine.
//!
//! Each opcode is associated with one of six formats that determine how many
//! operand bytes follow the opcode byte.

use std::fmt;

/// The six instruction encoding formats used by the FLUX bytecode format.
///
/// Each variant represents a different operand layout:
///
/// | Format | Name  | Operand bytes | Description                                    |
/// |--------|-------|---------------|------------------------------------------------|
/// | A      | Null  | 0             | No operands                                    |
/// | B      | Reg   | 2             | Two register IDs                               |
/// | C      | RegTy | 3             | Two register IDs + type tag byte               |
/// | D      | Imm   | 3             | Register ID + 16-bit signed immediate (LE)     |
/// | E      | Mem   | 4             | Two register IDs + 16-bit unsigned offset (LE) |
/// | G      | Var   | 1 + N         | Length byte + variable-length payload           |
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InstrFormat {
    /// No operands. Used by: `Halt`, `Nop`, `Ret`, `Panic`, `Unreachable`, `Yield`.
    ///
    /// Wire encoding: `[opcode]` — 1 byte total.
    A = 0,

    /// Two register IDs. Used by: `Push`, `Pop`, `Dup`, `Swap`, `IMov`, `FMov`, etc.
    ///
    /// Wire encoding: `[opcode, reg_dst, reg_src]` — 3 bytes total.
    B = 1,

    /// Two register IDs + type tag. Used by most arithmetic instructions.
    ///
    /// Wire encoding: `[opcode, reg_dst, reg_src, type_tag]` — 4 bytes total.
    C = 2,

    /// Register ID + 16-bit little-endian immediate. Used by: `IInc`, `IDec`, etc.
    ///
    /// Wire encoding: `[opcode, reg, imm_lo, imm_hi]` — 4 bytes total.
    D = 3,

    /// Two register IDs + 16-bit little-endian offset. Used by memory load/store.
    ///
    /// Wire encoding: `[opcode, reg_dst, reg_base, off_lo, off_hi]` — 5 bytes total.
    E = 4,

    /// Variable-length payload. Used by: `Call`, `Jump`, `JumpIf`, `JumpIfNot`,
    /// `CallIndirect`, `ASend`, `ARecv`, `AAsk`, `ATell`, `ADelegate`, `ABroadcast`,
    /// `ASubscribe`, `AWait`, `ATrust`, `AVerify`.
    ///
    /// Wire encoding: `[opcode, len, payload…]` — `2 + len` bytes total.
    G = 5,
}

impl InstrFormat {
    /// Returns the fixed operand size in bytes (excluding the opcode byte itself).
    ///
    /// Returns `None` for format `G` because its size depends on the payload length.
    #[must_use]
    pub const fn operand_size(&self) -> Option<usize> {
        match self {
            InstrFormat::A => Some(0),
            InstrFormat::B => Some(2),
            InstrFormat::C => Some(3),
            InstrFormat::D => Some(3),
            InstrFormat::E => Some(4),
            InstrFormat::G => None,
        }
    }

    /// Returns the total encoded instruction size in bytes for a given payload length.
    ///
    /// For format `G`, the caller must provide the payload length. For all other formats,
    /// the payload length is ignored.
    #[must_use]
    pub const fn total_size(&self, payload_len: usize) -> usize {
        1 + match self {
            InstrFormat::A => 0,
            InstrFormat::B => 2,
            InstrFormat::C => 3,
            InstrFormat::D => 3,
            InstrFormat::E => 4,
            InstrFormat::G => 1 + payload_len,
        }
    }

    /// Returns a human-readable name for this format.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            InstrFormat::A => "null",
            InstrFormat::B => "reg",
            InstrFormat::C => "reg_ty",
            InstrFormat::D => "imm",
            InstrFormat::E => "mem",
            InstrFormat::G => "var",
        }
    }
}

impl fmt::Display for InstrFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl TryFrom<u8> for InstrFormat {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(InstrFormat::A),
            1 => Ok(InstrFormat::B),
            2 => Ok(InstrFormat::C),
            3 => Ok(InstrFormat::D),
            4 => Ok(InstrFormat::E),
            5 => Ok(InstrFormat::G),
            _ => Err(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_operand_sizes() {
        assert_eq!(InstrFormat::A.operand_size(), Some(0));
        assert_eq!(InstrFormat::B.operand_size(), Some(2));
        assert_eq!(InstrFormat::C.operand_size(), Some(3));
        assert_eq!(InstrFormat::D.operand_size(), Some(3));
        assert_eq!(InstrFormat::E.operand_size(), Some(4));
        assert_eq!(InstrFormat::G.operand_size(), None);
    }

    #[test]
    fn format_total_sizes() {
        assert_eq!(InstrFormat::A.total_size(0), 1);
        assert_eq!(InstrFormat::B.total_size(0), 3);
        assert_eq!(InstrFormat::C.total_size(0), 4);
        assert_eq!(InstrFormat::D.total_size(0), 4);
        assert_eq!(InstrFormat::E.total_size(0), 5);
        assert_eq!(InstrFormat::G.total_size(10), 12);
    }

    #[test]
    fn format_roundtrip() {
        for val in 0u8..=5 {
            let fmt = InstrFormat::try_from(val).unwrap();
            assert_eq!(fmt as u8, val);
        }
        assert!(InstrFormat::try_from(6).is_err());
    }
}
