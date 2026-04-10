//! The FLUX virtual-machine interpreter (fetch-decode-execute loop).

use crate::error::VmError;
use crate::memory::{MemoryManager, Permissions, CODE_BASE};
use crate::registers::{FlagBits, RegisterFile};
use flux_bytecode::{InstrFormat, Op};
use std::fmt;

// ────────────────────────────────────────────────────────────────

/// Configuration knobs for the interpreter.
#[derive(Debug, Clone)]
pub struct VmConfig {
    pub max_cycles: u64,
    pub trace_enabled: bool,
    pub stack_size: u64,
    pub heap_size: u64,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            max_cycles: 1_000_000,
            trace_enabled: false,
            stack_size: 64 * 1024, // 64 KiB
            heap_size: 1024 * 1024, // 1 MiB
        }
    }
}

// ────────────────────────────────────────────────────────────────

/// Decoded instruction with all operand values extracted.
#[derive(Debug, Clone)]
pub struct DecodedInstr {
    pub op: Op,
    /// First register byte (often dst / data register).
    pub rd: u8,
    /// Second register byte (often src1 / base register).
    pub rs1: u8,
    /// Third register byte (often src2, used by format C).
    pub rs2: u8,
    /// Decoded immediate value (i16 for format D, i64 for format G).
    pub imm: i64,
    /// Total instruction length including the opcode byte.
    pub total_len: usize,
}

/// Result of a single `step()` call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepResult {
    Continue,
    Halted,
    Panicked(String),
    CycleLimitReached,
}

// ────────────────────────────────────────────────────────────────

/// The FLUX interpreter.
///
/// Holds the register file, memory manager, configuration, and
/// execution state.  It is `Send + Sync` (no interior mutability).
pub struct Interpreter {
    pub regs: RegisterFile,
    pub memory: MemoryManager,
    config: VmConfig,
    _bytecode_region: Option<crate::memory::RegionId>,
    halted: bool,
    panicked: bool,
    panic_message: Option<String>,
    trace_log: Vec<String>,
    /// Base address of the stack region (for overflow/underflow checks).
    stack_region_base: u64,
    stack_region_size: u64,
}

impl fmt::Debug for Interpreter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Interpreter")
            .field("halted", &self.halted)
            .field("panicked", &self.panicked)
            .field("cycles", &self.regs.cycles())
            .finish()
    }
}

// ────────────────────────────────────────────────────────────────

impl Interpreter {
    /// Create a new interpreter, load `bytecode` into memory, set up
    /// the stack, and initialise PC / SP.
    pub fn new(bytecode: &[u8], config: VmConfig) -> Result<Self, VmError> {
        let mut memory = MemoryManager::new();

        // Load bytecode into the code region.
        let bc_id = memory.load_bytecode(bytecode)?;

        // Allocate the stack region.
        let stack_id = memory.allocate(config.stack_size, Permissions::read_write(), Some("stack"))?;
        let stack_region = memory.region(stack_id).ok_or_else(|| {
            VmError::Execution("stack region disappeared after allocation".into())
        })?;
        let stack_region_base = stack_region.base;
        let stack_region_size = stack_region.size;

        // Allocate the heap region.
        let _heap_id = memory.allocate(config.heap_size, Permissions::read_write(), Some("heap"))?;

        // Initialise registers.
        let mut regs = RegisterFile::new();
        regs.set_pc(CODE_BASE);
        // SP starts at the top of the stack region (one past end).
        regs.set_sp((stack_region_base + stack_region_size) as i64);

        Ok(Self {
            regs,
            memory,
            config,
            _bytecode_region: Some(crate::memory::RegionId(bc_id.0)),
            halted: false,
            panicked: false,
            panic_message: None,
            trace_log: Vec::new(),
            stack_region_base,
            stack_region_size,
        })
    }

    // ── Public interface ────────────────────────────────────────

    /// Run the VM until it halts, panics, or hits the cycle limit.
    ///
    /// Returns the number of cycles consumed on a clean halt.
    pub fn execute(&mut self) -> Result<u64, VmError> {
        loop {
            match self.step()? {
                StepResult::Continue => {}
                StepResult::Halted => return Ok(self.regs.cycles()),
                StepResult::Panicked(msg) => return Err(VmError::Panic(msg)),
                StepResult::CycleLimitReached => {
                    return Err(VmError::CycleLimit(self.config.max_cycles));
                }
            }
        }
    }

    /// Execute a single instruction.
    pub fn step(&mut self) -> Result<StepResult, VmError> {
        if self.halted {
            return Ok(StepResult::Halted);
        }
        if self.panicked {
            return Ok(StepResult::Panicked(
                self.panic_message.clone().unwrap_or_default(),
            ));
        }

        // Check cycle limit.
        if self.regs.cycles() >= self.config.max_cycles {
            return Ok(StepResult::CycleLimitReached);
        }

        // Increment cycle counter.
        self.regs.increment_cycles();

        // Fetch + decode.
        let pc = self.regs.pc();
        let decoded = self.fetch_and_decode(pc)?;

        // Advance PC past this instruction (default fall-through).
        let next_pc = pc + decoded.total_len as u64;
        self.regs.set_pc(next_pc);

        // Trace.
        if self.config.trace_enabled {
            self.trace_log
                .push(format!("[{:04}] {:?}", self.regs.cycles(), decoded.op));
        }

        // Execute.
        let result = self.execute_instruction(&decoded)?;

        Ok(result)
    }

    #[must_use]
    pub fn is_halted(&self) -> bool {
        self.halted
    }

    #[must_use]
    pub fn is_panicked(&self) -> bool {
        self.panicked
    }

    #[must_use]
    pub fn panic_message(&self) -> Option<&str> {
        self.panic_message.as_deref()
    }

    #[must_use]
    pub fn trace_log(&self) -> &[String] {
        &self.trace_log
    }

    /// Reset the VM to its initial state (keeping the same config
    /// and bytecode).  Re-initialises registers and clears the
    /// execution state.
    pub fn reset(&mut self) {
        self.regs.reset();
        self.regs.set_pc(CODE_BASE);
        self.regs
            .set_sp((self.stack_region_base + self.stack_region_size) as i64);
        self.halted = false;
        self.panicked = false;
        self.panic_message = None;
        self.trace_log.clear();
    }

    // ── Fetch + decode ──────────────────────────────────────────

    fn fetch_and_decode(&self, pc: u64) -> Result<DecodedInstr, VmError> {
        let opcode_byte = self.memory.read_u8(pc)?;
        let op = Op::try_from(opcode_byte).map_err(|_| VmError::InvalidOpcode(opcode_byte))?;
        let format = op.format();

        match format {
            InstrFormat::A => Ok(DecodedInstr {
                op,
                rd: 0,
                rs1: 0,
                rs2: 0,
                imm: 0,
                total_len: 1,
            }),

            InstrFormat::B => {
                let rd = self.memory.read_u8(pc + 1)?;
                let rs1 = self.memory.read_u8(pc + 2)?;
                Ok(DecodedInstr {
                    op,
                    rd,
                    rs1,
                    rs2: 0,
                    imm: 0,
                    total_len: 3,
                })
            }

            InstrFormat::C => {
                let rd = self.memory.read_u8(pc + 1)?;
                let rs1 = self.memory.read_u8(pc + 2)?;
                let rs2 = self.memory.read_u8(pc + 3)?;
                Ok(DecodedInstr {
                    op,
                    rd,
                    rs1,
                    rs2,
                    imm: 0,
                    total_len: 4,
                })
            }

            InstrFormat::D => {
                let rd = self.memory.read_u8(pc + 1)?;
                let lo = self.memory.read_u8(pc + 2)?;
                let hi = self.memory.read_u8(pc + 3)?;
                let imm = i16::from_le_bytes([lo, hi]) as i64;
                Ok(DecodedInstr {
                    op,
                    rd,
                    rs1: 0,
                    rs2: 0,
                    imm,
                    total_len: 4,
                })
            }

            InstrFormat::E => {
                let rd = self.memory.read_u8(pc + 1)?;
                let rs1 = self.memory.read_u8(pc + 2)?;
                let off_lo = self.memory.read_u8(pc + 3)?;
                let off_hi = self.memory.read_u8(pc + 4)?;
                let imm = u16::from_le_bytes([off_lo, off_hi]) as i64;
                Ok(DecodedInstr {
                    op,
                    rd,
                    rs1,
                    rs2: 0,
                    imm,
                    total_len: 5,
                })
            }

            InstrFormat::G => {
                let len = self.memory.read_u8(pc + 1)? as usize;
                if len == 0 {
                    return Ok(DecodedInstr {
                        op,
                        rd: 0,
                        rs1: 0,
                        rs2: 0,
                        imm: 0,
                        total_len: 2,
                    });
                }
                // Read up to 8 bytes of payload as a signed i64 (LE).
                let imm = if len >= 8 {
                    self.memory.read_u64(pc + 2)? as i64
                } else if len >= 4 {
                    self.memory.read_u32(pc + 2)? as i64
                } else if len >= 2 {
                    self.memory.read_u16(pc + 2)? as i64
                } else {
                    self.memory.read_u8(pc + 2)? as i8 as i64
                };
                Ok(DecodedInstr {
                    op,
                    rd: 0,
                    rs1: 0,
                    rs2: 0,
                    imm,
                    total_len: 2 + len,
                })
            }
        }
    }

    // ── Execute ─────────────────────────────────────────────────

    fn execute_instruction(&mut self, d: &DecodedInstr) -> Result<StepResult, VmError> {
        match d.op {
            // ── Format A ────────────────────────────────────────
            Op::Nop => Ok(StepResult::Continue),

            Op::Halt => {
                self.halted = true;
                Ok(StepResult::Halted)
            }

            Op::Ret => {
                let ret_addr = self.pop_stack()? as u64;
                self.regs.set_pc(ret_addr);
                Ok(StepResult::Continue)
            }

            Op::Yield => Ok(StepResult::Continue),

            Op::Panic => {
                self.panicked = true;
                self.panic_message = Some("execution panicked".to_string());
                Ok(StepResult::Panicked("execution panicked".to_string()))
            }

            // ── Format B ────────────────────────────────────────
            Op::Push => {
                let val = self.regs.read_gp(d.rd);
                self.push_stack(val)?;
                Ok(StepResult::Continue)
            }

            Op::Pop => {
                let val = self.pop_stack()?;
                self.regs.write_gp(d.rd, val);
                Ok(StepResult::Continue)
            }

            Op::Dup => {
                let sp = self.regs.sp();
                let top = self.memory.read_u64(sp as u64)? as i64;
                self.push_stack(top)?;
                Ok(StepResult::Continue)
            }

            Op::Swap => {
                let a = self.regs.read_gp(d.rd);
                let b = self.regs.read_gp(d.rs1);
                self.regs.write_gp(d.rd, b);
                self.regs.write_gp(d.rs1, a);
                Ok(StepResult::Continue)
            }

            Op::IMov => {
                let val = self.regs.read_gp(d.rs1);
                self.regs.write_gp(d.rd, val);
                Ok(StepResult::Continue)
            }

            Op::FMov => {
                let val = self.regs.read_fp(d.rs1);
                self.regs.write_fp(d.rd, val);
                Ok(StepResult::Continue)
            }

            Op::INeg => {
                let val = self.regs.read_gp(d.rs1);
                let result = val.wrapping_neg();
                self.regs.write_gp(d.rd, result);
                self.regs.update_flags_i64(result);
                Ok(StepResult::Continue)
            }

            Op::FNeg => {
                let val = self.regs.read_fp(d.rs1);
                let result = -val;
                self.regs.write_fp(d.rd, result);
                self.regs.update_flags_f64(result);
                Ok(StepResult::Continue)
            }

            Op::IToF => {
                let val = self.regs.read_gp(d.rs1);
                self.regs.write_fp(d.rd, val as f64);
                Ok(StepResult::Continue)
            }

            Op::FToI => {
                let val = self.regs.read_fp(d.rs1);
                self.regs.write_gp(d.rd, val as i64);
                Ok(StepResult::Continue)
            }

            // ── Format C — integer arithmetic ───────────────────
            Op::IAdd => {
                let a = self.regs.read_gp(d.rs1);
                let b = self.regs.read_gp(d.rs2);
                let result = a.wrapping_add(b);
                self.regs.write_gp(d.rd, result);
                self.regs.update_flags_i64(result);
                Ok(StepResult::Continue)
            }

            Op::ISub => {
                let a = self.regs.read_gp(d.rs1);
                let b = self.regs.read_gp(d.rs2);
                let result = a.wrapping_sub(b);
                self.regs.write_gp(d.rd, result);
                self.regs.update_flags_i64(result);
                Ok(StepResult::Continue)
            }

            Op::IMul => {
                let a = self.regs.read_gp(d.rs1);
                let b = self.regs.read_gp(d.rs2);
                let result = a.wrapping_mul(b);
                self.regs.write_gp(d.rd, result);
                self.regs.update_flags_i64(result);
                Ok(StepResult::Continue)
            }

            Op::IDiv => {
                let a = self.regs.read_gp(d.rs1);
                let b = self.regs.read_gp(d.rs2);
                if b == 0 {
                    return Err(VmError::DivisionByZero);
                }
                let result = a.wrapping_div(b);
                self.regs.write_gp(d.rd, result);
                self.regs.update_flags_i64(result);
                Ok(StepResult::Continue)
            }

            Op::IMod => {
                let a = self.regs.read_gp(d.rs1);
                let b = self.regs.read_gp(d.rs2);
                if b == 0 {
                    return Err(VmError::DivisionByZero);
                }
                let result = a.wrapping_rem(b);
                self.regs.write_gp(d.rd, result);
                self.regs.update_flags_i64(result);
                Ok(StepResult::Continue)
            }

            Op::IAnd => {
                let a = self.regs.read_gp(d.rs1);
                let b = self.regs.read_gp(d.rs2);
                let result = a & b;
                self.regs.write_gp(d.rd, result);
                self.regs.update_flags_i64(result);
                Ok(StepResult::Continue)
            }

            Op::IOr => {
                let a = self.regs.read_gp(d.rs1);
                let b = self.regs.read_gp(d.rs2);
                let result = a | b;
                self.regs.write_gp(d.rd, result);
                self.regs.update_flags_i64(result);
                Ok(StepResult::Continue)
            }

            Op::IXor => {
                let a = self.regs.read_gp(d.rs1);
                let b = self.regs.read_gp(d.rs2);
                let result = a ^ b;
                self.regs.write_gp(d.rd, result);
                self.regs.update_flags_i64(result);
                Ok(StepResult::Continue)
            }

            Op::IShl => {
                let a = self.regs.read_gp(d.rs1);
                let b = self.regs.read_gp(d.rs2);
                let shift = (b & 63) as u32;
                let result = a.wrapping_shl(shift);
                self.regs.write_gp(d.rd, result);
                self.regs.update_flags_i64(result);
                Ok(StepResult::Continue)
            }

            Op::IShr => {
                let a = self.regs.read_gp(d.rs1);
                let b = self.regs.read_gp(d.rs2);
                let shift = (b & 63) as u32;
                let result = a.wrapping_shr(shift);
                self.regs.write_gp(d.rd, result);
                self.regs.update_flags_i64(result);
                Ok(StepResult::Continue)
            }

            // ── Format C — float arithmetic ─────────────────────
            Op::FAdd => {
                let a = self.regs.read_fp(d.rs1);
                let b = self.regs.read_fp(d.rs2);
                let result = a + b;
                self.regs.write_fp(d.rd, result);
                self.regs.update_flags_f64(result);
                Ok(StepResult::Continue)
            }

            Op::FSub => {
                let a = self.regs.read_fp(d.rs1);
                let b = self.regs.read_fp(d.rs2);
                let result = a - b;
                self.regs.write_fp(d.rd, result);
                self.regs.update_flags_f64(result);
                Ok(StepResult::Continue)
            }

            Op::FMul => {
                let a = self.regs.read_fp(d.rs1);
                let b = self.regs.read_fp(d.rs2);
                let result = a * b;
                self.regs.write_fp(d.rd, result);
                self.regs.update_flags_f64(result);
                Ok(StepResult::Continue)
            }

            Op::FDiv => {
                let a = self.regs.read_fp(d.rs1);
                let b = self.regs.read_fp(d.rs2);
                if b == 0.0 {
                    // Allow IEEE 754 inf/nan for float division by zero.
                    let result = a / b;
                    self.regs.write_fp(d.rd, result);
                    self.regs.update_flags_f64(result);
                } else {
                    let result = a / b;
                    self.regs.write_fp(d.rd, result);
                    self.regs.update_flags_f64(result);
                }
                Ok(StepResult::Continue)
            }

            // ── Format C — comparisons ──────────────────────────
            Op::ICmpEq => {
                let a = self.regs.read_gp(d.rs1);
                let b = self.regs.read_gp(d.rs2);
                let result = if a == b { 1 } else { 0 };
                self.regs.write_gp(d.rd, result);
                let mut flags = self.regs.flags();
                flags.set_to(FlagBits::ZERO, a == b);
                self.regs.set_flags(flags);
                Ok(StepResult::Continue)
            }

            Op::ICmpNe => {
                let a = self.regs.read_gp(d.rs1);
                let b = self.regs.read_gp(d.rs2);
                let result = if a != b { 1 } else { 0 };
                self.regs.write_gp(d.rd, result);
                let mut flags = self.regs.flags();
                flags.set_to(FlagBits::ZERO, a != b);
                self.regs.set_flags(flags);
                Ok(StepResult::Continue)
            }

            Op::ICmpLt => {
                let a = self.regs.read_gp(d.rs1);
                let b = self.regs.read_gp(d.rs2);
                let result = if a < b { 1 } else { 0 };
                self.regs.write_gp(d.rd, result);
                let mut flags = self.regs.flags();
                flags.set_to(FlagBits::ZERO, a < b);
                self.regs.set_flags(flags);
                Ok(StepResult::Continue)
            }

            Op::ICmpGt => {
                let a = self.regs.read_gp(d.rs1);
                let b = self.regs.read_gp(d.rs2);
                let result = if a > b { 1 } else { 0 };
                self.regs.write_gp(d.rd, result);
                let mut flags = self.regs.flags();
                flags.set_to(FlagBits::ZERO, a > b);
                self.regs.set_flags(flags);
                Ok(StepResult::Continue)
            }

            Op::ICmpLe => {
                let a = self.regs.read_gp(d.rs1);
                let b = self.regs.read_gp(d.rs2);
                let result = if a <= b { 1 } else { 0 };
                self.regs.write_gp(d.rd, result);
                let mut flags = self.regs.flags();
                flags.set_to(FlagBits::ZERO, a <= b);
                self.regs.set_flags(flags);
                Ok(StepResult::Continue)
            }

            Op::ICmpGe => {
                let a = self.regs.read_gp(d.rs1);
                let b = self.regs.read_gp(d.rs2);
                let result = if a >= b { 1 } else { 0 };
                self.regs.write_gp(d.rd, result);
                let mut flags = self.regs.flags();
                flags.set_to(FlagBits::ZERO, a >= b);
                self.regs.set_flags(flags);
                Ok(StepResult::Continue)
            }

            // ── Format D ────────────────────────────────────────
            Op::IInc => {
                let val = self.regs.read_gp(d.rd);
                let result = val.wrapping_add(d.imm);
                self.regs.write_gp(d.rd, result);
                self.regs.update_flags_i64(result);
                Ok(StepResult::Continue)
            }

            Op::IDec => {
                let val = self.regs.read_gp(d.rd);
                let result = val.wrapping_sub(d.imm);
                self.regs.write_gp(d.rd, result);
                self.regs.update_flags_i64(result);
                Ok(StepResult::Continue)
            }

            Op::StackAlloc => {
                let size = d.imm as u64;
                let new_sp = self.regs.sp().wrapping_add(size as i64);
                // Check overflow
                let abs_sp = new_sp as u64;
                if abs_sp > self.stack_region_base + self.stack_region_size {
                    return Err(VmError::StackOverflow);
                }
                self.regs.set_sp(new_sp);
                Ok(StepResult::Continue)
            }

            // ── Format E — memory loads ─────────────────────────
            Op::Load8 => {
                let base = self.regs.read_gp(d.rs1);
                let addr = (base as u64).wrapping_add(d.imm as u64);
                let val = self.memory.read_u8(addr)? as i64;
                self.regs.write_gp(d.rd, val);
                Ok(StepResult::Continue)
            }

            Op::Load16 => {
                let base = self.regs.read_gp(d.rs1);
                let addr = (base as u64).wrapping_add(d.imm as u64);
                let val = self.memory.read_u16(addr)? as i64;
                self.regs.write_gp(d.rd, val);
                Ok(StepResult::Continue)
            }

            Op::Load32 => {
                let base = self.regs.read_gp(d.rs1);
                let addr = (base as u64).wrapping_add(d.imm as u64);
                let val = self.memory.read_u32(addr)? as i64;
                self.regs.write_gp(d.rd, val);
                Ok(StepResult::Continue)
            }

            Op::Load64 => {
                let base = self.regs.read_gp(d.rs1);
                let addr = (base as u64).wrapping_add(d.imm as u64);
                let val = self.memory.read_u64(addr)? as i64;
                self.regs.write_gp(d.rd, val);
                Ok(StepResult::Continue)
            }

            // ── Format E — memory stores ────────────────────────
            Op::Store8 => {
                let base = self.regs.read_gp(d.rs1);
                let addr = (base as u64).wrapping_add(d.imm as u64);
                let val = self.regs.read_gp(d.rd) as u8;
                self.memory.write_u8(addr, val)?;
                Ok(StepResult::Continue)
            }

            Op::Store16 => {
                let base = self.regs.read_gp(d.rs1);
                let addr = (base as u64).wrapping_add(d.imm as u64);
                let val = self.regs.read_gp(d.rd) as u16;
                self.memory.write_u16(addr, val)?;
                Ok(StepResult::Continue)
            }

            Op::Store32 => {
                let base = self.regs.read_gp(d.rs1);
                let addr = (base as u64).wrapping_add(d.imm as u64);
                let val = self.regs.read_gp(d.rd) as u32;
                self.memory.write_u32(addr, val)?;
                Ok(StepResult::Continue)
            }

            Op::Store64 => {
                let base = self.regs.read_gp(d.rs1);
                let addr = (base as u64).wrapping_add(d.imm as u64);
                let val = self.regs.read_gp(d.rd) as u64;
                self.memory.write_u64(addr, val)?;
                Ok(StepResult::Continue)
            }

            // ── Format G — control flow ─────────────────────────
            Op::Jump => {
                let target = self.regs.pc().wrapping_add(d.imm as u64);
                self.regs.set_pc(target);
                Ok(StepResult::Continue)
            }

            Op::JumpIf => {
                if self.regs.flags().is_zero() {
                    let target = self.regs.pc().wrapping_add(d.imm as u64);
                    self.regs.set_pc(target);
                }
                Ok(StepResult::Continue)
            }

            Op::JumpIfNot => {
                if !self.regs.flags().is_zero() {
                    let target = self.regs.pc().wrapping_add(d.imm as u64);
                    self.regs.set_pc(target);
                }
                Ok(StepResult::Continue)
            }

            Op::Call => {
                let return_addr = self.regs.pc(); // next instruction (already advanced)
                self.push_stack(return_addr as i64)?;
                let target = self.regs.pc().wrapping_add(d.imm as u64);
                self.regs.set_pc(target);
                Ok(StepResult::Continue)
            }

            // ── Agent stubs ─────────────────────────────────────
            Op::ASend => {
                // Stub: push placeholder handle (1).
                self.push_stack(1)?;
                Ok(StepResult::Continue)
            }

            Op::ARecv => {
                // Stub: push success indicator (0).
                self.push_stack(0)?;
                Ok(StepResult::Continue)
            }

            Op::AAsk => {
                // Stub: push received value (0).
                self.push_stack(0)?;
                Ok(StepResult::Continue)
            }

            // Catch-all for non-exhaustive Op — treat as extended NOP
            _ => {
                if self.config.trace_enabled {
                    self.trace_log.push(format!(
                        "[PC={:04x}] {:?} — unimplemented, skipping",
                        self.regs.pc(),
                        d.op
                    ));
                }
                Ok(StepResult::Continue)
            }
        }
    }

    // ── Stack helpers ───────────────────────────────────────────

    fn push_stack(&mut self, val: i64) -> Result<(), VmError> {
        let new_sp = self.regs.sp() - 8;
        if (new_sp as u64) < self.stack_region_base {
            return Err(VmError::StackOverflow);
        }
        self.regs.set_sp(new_sp);
        self.memory.write_u64(new_sp as u64, val as u64)?;
        Ok(())
    }

    fn pop_stack(&mut self) -> Result<i64, VmError> {
        let sp = self.regs.sp();
        if (sp as u64) >= self.stack_region_base + self.stack_region_size {
            return Err(VmError::StackUnderflow);
        }
        let val = self.memory.read_u64(sp as u64)? as i64;
        self.regs.set_sp(sp + 8);
        Ok(val)
    }
}
