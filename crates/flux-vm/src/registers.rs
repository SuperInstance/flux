//! 64-register file with four banks and aliasing for SP/FP/LR.

use std::fmt;

/// Bit flags stored in system register S4.
///
/// The raw value is a `u8`.  Use the constant methods or
/// `FlagBits::from_bits_truncate` to manipulate individual flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlagBits(pub u8);

impl FlagBits {
    pub const ZERO: u8 = 0x01;
    pub const NEGATIVE: u8 = 0x02;
    pub const CARRY: u8 = 0x04;
    pub const OVERFLOW: u8 = 0x08;

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn all() -> Self {
        Self(Self::ZERO | Self::NEGATIVE | Self::CARRY | Self::OVERFLOW)
    }

    #[must_use]
    pub const fn contains(&self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.contains(Self::ZERO)
    }

    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.contains(Self::NEGATIVE)
    }

    #[must_use]
    pub const fn is_carry(&self) -> bool {
        self.contains(Self::CARRY)
    }

    #[must_use]
    pub const fn is_overflow(&self) -> bool {
        self.contains(Self::OVERFLOW)
    }

    pub fn set(&mut self, flag: u8) {
        self.0 |= flag;
    }

    pub fn clear(&mut self, flag: u8) {
        self.0 &= !flag;
    }

    pub fn set_to(&mut self, flag: u8, value: bool) {
        if value {
            self.set(flag);
        } else {
            self.clear(flag);
        }
    }

    pub fn from_bits_truncate(bits: u8) -> Self {
        Self(bits & Self::all().0)
    }
}

impl fmt::Display for FlagBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:02X}", self.0)
    }
}

// ────────────────────────────────────────────────────────────────

/// System register indices.
const SYS_PC: usize = 0;
const SYS_SP: usize = 1;
const SYS_FP: usize = 2;
const SYS_LR: usize = 3;
const SYS_FLAGS: usize = 4;
const SYS_CYCLES: usize = 6;

/// Register aliases: R11 → S1 (SP), R12 → S2 (FP), R13 → S3 (LR).
const GP_SP_ALIAS: u8 = 11;
const GP_FP_ALIAS: u8 = 12;
const GP_LR_ALIAS: u8 = 13;

/// The FLUX register file: 64 registers across four banks.
///
/// | Bank | Registers | Type   |
/// |------|-----------|--------|
/// | GP   | R0 – R15  | `i64`  |
/// | FP   | F0 – F15  | `f64`  |
/// | VEC  | V0 – V15  | `[u8;16]` |
/// | SYS  | S0 – S15  | `u64`  |
///
/// Aliases: `R11` ↔ `S1` (SP), `R12` ↔ `S2` (FP), `R13` ↔ `S3` (LR).
#[derive(Debug, Clone)]
pub struct RegisterFile {
    gp: [i64; 16],
    fp: [f64; 16],
    vec: [[u8; 16]; 16],
    sys: [u64; 16],
}

impl Default for RegisterFile {
    fn default() -> Self {
        Self::new()
    }
}

impl RegisterFile {
    /// Create a new register file with all registers zeroed.
    #[must_use]
    pub fn new() -> Self {
        Self {
            gp: [0i64; 16],
            fp: [0.0f64; 16],
            vec: [[0u8; 16]; 16],
            sys: [0u64; 16],
        }
    }

    // ── General-purpose ─────────────────────────────────────────

    /// Read a general-purpose register (0–15).  Indices 11/12/13
    /// are aliased to SP/FP/LR respectively.
    pub fn read_gp(&self, idx: u8) -> i64 {
        match idx {
            GP_SP_ALIAS => self.sys[SYS_SP] as i64,
            GP_FP_ALIAS => self.sys[SYS_FP] as i64,
            GP_LR_ALIAS => self.sys[SYS_LR] as i64,
            i if (i as usize) < 16 => self.gp[i as usize],
            _ => 0,
        }
    }

    /// Write a general-purpose register (0–15).
    pub fn write_gp(&mut self, idx: u8, val: i64) {
        match idx {
            GP_SP_ALIAS => self.sys[SYS_SP] = val as u64,
            GP_FP_ALIAS => self.sys[SYS_FP] = val as u64,
            GP_LR_ALIAS => self.sys[SYS_LR] = val as u64,
            i if (i as usize) < 16 => self.gp[i as usize] = val,
            _ => {}
        }
    }

    // ── Floating-point ──────────────────────────────────────────

    pub fn read_fp(&self, idx: u8) -> f64 {
        if (idx as usize) < 16 {
            self.fp[idx as usize]
        } else {
            0.0
        }
    }

    pub fn write_fp(&mut self, idx: u8, val: f64) {
        if (idx as usize) < 16 {
            self.fp[idx as usize] = val;
        }
    }

    // ── Vector ──────────────────────────────────────────────────

    pub fn read_vec(&self, idx: u8) -> [u8; 16] {
        if (idx as usize) < 16 {
            self.vec[idx as usize]
        } else {
            [0u8; 16]
        }
    }

    pub fn write_vec(&mut self, idx: u8, val: [u8; 16]) {
        if (idx as usize) < 16 {
            self.vec[idx as usize] = val;
        }
    }

    // ── System registers ────────────────────────────────────────

    pub fn read_sys(&self, idx: u8) -> u64 {
        if (idx as usize) < 16 {
            self.sys[idx as usize]
        } else {
            0
        }
    }

    pub fn write_sys(&mut self, idx: u8, val: u64) {
        if (idx as usize) < 16 {
            self.sys[idx as usize] = val;
        }
    }

    // ── Convenience accessors ───────────────────────────────────

    #[must_use]
    pub fn pc(&self) -> u64 {
        self.sys[SYS_PC]
    }

    pub fn set_pc(&mut self, val: u64) {
        self.sys[SYS_PC] = val;
    }

    #[must_use]
    pub fn sp(&self) -> i64 {
        self.sys[SYS_SP] as i64
    }

    pub fn set_sp(&mut self, val: i64) {
        self.sys[SYS_SP] = val as u64;
    }

    #[must_use]
    pub fn fp(&self) -> i64 {
        self.sys[SYS_FP] as i64
    }

    pub fn set_fp(&mut self, val: i64) {
        self.sys[SYS_FP] = val as u64;
    }

    #[must_use]
    pub fn lr(&self) -> u64 {
        self.sys[SYS_LR]
    }

    pub fn set_lr(&mut self, val: u64) {
        self.sys[SYS_LR] = val;
    }

    #[must_use]
    pub fn flags(&self) -> FlagBits {
        FlagBits(self.sys[SYS_FLAGS] as u8)
    }

    pub fn set_flags(&mut self, flags: FlagBits) {
        self.sys[SYS_FLAGS] = flags.0 as u64;
    }

    /// Update the ZERO and NEGATIVE flags based on an `i64` result.
    pub fn update_flags_i64(&mut self, result: i64) {
        let mut flags = self.flags();
        flags.set_to(FlagBits::ZERO, result == 0);
        flags.set_to(FlagBits::NEGATIVE, result < 0);
        self.set_flags(flags);
    }

    /// Update the ZERO flag based on an `f64` result.
    pub fn update_flags_f64(&mut self, result: f64) {
        let mut flags = self.flags();
        flags.set_to(FlagBits::ZERO, result == 0.0);
        self.set_flags(flags);
    }

    #[must_use]
    pub fn cycles(&self) -> u64 {
        self.sys[SYS_CYCLES]
    }

    pub fn increment_cycles(&mut self) {
        self.sys[SYS_CYCLES] += 1;
    }

    /// Reset all registers to zero.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sp_alias() {
        let mut rf = RegisterFile::new();
        rf.set_sp(0xDEAD_BEEF);
        assert_eq!(rf.read_gp(GP_SP_ALIAS), 0xDEAD_BEEF_i64);
        rf.write_gp(GP_SP_ALIAS, 42);
        assert_eq!(rf.sp(), 42);
    }

    #[test]
    fn fp_alias() {
        let mut rf = RegisterFile::new();
        rf.set_fp(0xCAFE);
        assert_eq!(rf.read_gp(GP_FP_ALIAS), 0xCAFE_i64);
        rf.write_gp(GP_FP_ALIAS, 99);
        assert_eq!(rf.fp(), 99);
    }

    #[test]
    fn lr_alias() {
        let mut rf = RegisterFile::new();
        rf.set_lr(0x1234_5678);
        assert_eq!(rf.read_gp(GP_LR_ALIAS) as u64, 0x1234_5678);
        rf.write_gp(GP_LR_ALIAS, 0xAAAA_i64);
        assert_eq!(rf.lr(), 0xAAAA);
    }

    #[test]
    fn flags() {
        let mut rf = RegisterFile::new();
        rf.update_flags_i64(0);
        assert!(rf.flags().is_zero());
        rf.update_flags_i64(-5);
        assert!(!rf.flags().is_zero());
        assert!(rf.flags().is_negative());
    }

    #[test]
    fn reset() {
        let mut rf = RegisterFile::new();
        rf.set_pc(100);
        rf.set_sp(200);
        rf.write_gp(1, 42);
        rf.reset();
        assert_eq!(rf.pc(), 0);
        assert_eq!(rf.sp(), 0);
        assert_eq!(rf.read_gp(1), 0);
    }

    #[test]
    fn cycles() {
        let mut rf = RegisterFile::new();
        assert_eq!(rf.cycles(), 0);
        rf.increment_cycles();
        assert_eq!(rf.cycles(), 1);
        rf.increment_cycles();
        assert_eq!(rf.cycles(), 2);
    }
}
