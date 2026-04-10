//! FLUX bytecode file header — 18-byte fixed preamble.
//!
//! Every FLUX bytecode module begins with an 18-byte header that identifies
//! the file format, declares the version and flags, and provides offsets to
//! the function and string tables.

use crate::error::{DecodeError, EncodeError};

/// The magic bytes that identify a FLUX bytecode file: `b"FLUX"`.
pub const MAGIC: &[u8; 4] = b"FLUX";

/// The currently supported bytecode version.
pub const VERSION: u16 = 1;

/// The fixed size of the header in bytes.
pub const HEADER_SIZE: usize = 18;

/// The 18-byte bytecode file header.
///
/// Layout (all multi-byte fields are little-endian):
///
/// | Offset | Size | Field          |
/// |--------|------|----------------|
/// | 0      | 4    | magic (`b"FLUX"`) |
/// | 4      | 2    | version        |
/// | 6      | 2    | flags          |
/// | 8      | 4    | num_functions  |
/// | 12     | 4    | num_strings    |
/// | 16     | 2    | entry_point    |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct BytecodeHeader {
    /// Magic bytes — must be `b"FLUX"`.
    pub magic: [u8; 4],
    /// Bytecode format version (currently `1`).
    pub version: u16,
    /// Feature / compilation flags.
    pub flags: u16,
    /// Number of function bodies in the module.
    pub num_functions: u32,
    /// Number of entries in the string table.
    pub num_strings: u32,
    /// Index of the entry-point function.
    pub entry_point: u16,
}

impl Default for BytecodeHeader {
    fn default() -> Self {
        Self {
            magic: *MAGIC,
            version: VERSION,
            flags: 0,
            num_functions: 0,
            num_strings: 0,
            entry_point: 0,
        }
    }
}

impl BytecodeHeader {
    /// Creates a new header with the given function and string counts.
    #[must_use]
    pub fn new(num_functions: u32, num_strings: u32, entry_point: u16) -> Self {
        Self {
            magic: *MAGIC,
            version: VERSION,
            flags: 0,
            num_functions,
            num_strings,
            entry_point,
        }
    }

    /// Creates a new header with custom flags.
    #[must_use]
    pub fn with_flags(num_functions: u32, num_strings: u32, entry_point: u16, flags: u16) -> Self {
        Self {
            magic: *MAGIC,
            version: VERSION,
            flags,
            num_functions,
            num_strings,
            entry_point,
        }
    }

    /// Serializes this header into an 18-byte little-endian byte vector.
    ///
    /// # Errors
    ///
    /// Returns [`EncodeError`] if the magic bytes are invalid.
    pub fn to_bytes(&self) -> Result<[u8; HEADER_SIZE], EncodeError> {
        if &self.magic != MAGIC {
            return Err(EncodeError::FormatMismatch {
                detail: format!("invalid magic: expected {:?}, got {:?}", MAGIC, &self.magic as &[u8]),
            });
        }

        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4..6].copy_from_slice(&self.version.to_le_bytes());
        buf[6..8].copy_from_slice(&self.flags.to_le_bytes());
        buf[8..12].copy_from_slice(&self.num_functions.to_le_bytes());
        buf[12..16].copy_from_slice(&self.num_strings.to_le_bytes());
        buf[16..18].copy_from_slice(&self.entry_point.to_le_bytes());
        Ok(buf)
    }

    /// Deserializes a header from a byte slice.
    ///
    /// The slice must be at least 18 bytes long.
    ///
    /// # Errors
    ///
    /// Returns [`DecodeError::InvalidMagic`] if the first 4 bytes are not `b"FLUX"`.
    /// Returns [`DecodeError::UnexpectedEof`] if the slice is shorter than 18 bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < HEADER_SIZE {
            return Err(DecodeError::UnexpectedEof {
                expected: HEADER_SIZE,
                available: bytes.len(),
            });
        }

        let mut magic = [0u8; 4];
        magic.copy_from_slice(&bytes[0..4]);

        if &magic != MAGIC {
            return Err(DecodeError::InvalidMagic { found: magic });
        }

        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if version != VERSION {
            return Err(DecodeError::UnsupportedVersion { version });
        }

        let flags = u16::from_le_bytes([bytes[6], bytes[7]]);
        let num_functions = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let num_strings = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let entry_point = u16::from_le_bytes([bytes[16], bytes[17]]);

        Ok(Self {
            magic,
            version,
            flags,
            num_functions,
            num_strings,
            entry_point,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_header() {
        let h = BytecodeHeader::default();
        assert_eq!(h.magic, *MAGIC);
        assert_eq!(h.version, VERSION);
    }

    #[test]
    fn roundtrip() {
        let original = BytecodeHeader::with_flags(3, 10, 0, 0x0100);
        let bytes = original.to_bytes().unwrap();
        assert_eq!(bytes.len(), HEADER_SIZE);
        let decoded = BytecodeHeader::from_bytes(&bytes).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn invalid_magic_encode() {
        let h = BytecodeHeader {
            magic: [0x00, 0x00, 0x00, 0x00],
            ..Default::default()
        };
        assert!(h.to_bytes().is_err());
    }

    #[test]
    fn invalid_magic_decode() {
        let bad = [0x00u8; HEADER_SIZE];
        assert!(matches!(
            BytecodeHeader::from_bytes(&bad),
            Err(DecodeError::InvalidMagic { .. })
        ));
    }

    #[test]
    fn too_short() {
        let short = [0u8; 10];
        assert!(matches!(
            BytecodeHeader::from_bytes(&short),
            Err(DecodeError::UnexpectedEof { .. })
        ));
    }
}
