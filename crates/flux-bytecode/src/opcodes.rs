//! FLUX bytecode opcodes — the complete instruction set.
//!
//! This module defines all 100+ opcodes used by the FLUX virtual machine,
//! organized into categories: control flow, stack, integer arithmetic, float
//! arithmetic, conversions, memory, agent-to-agent communication, type/meta,
//! bitwise operations, and vector operations.

use crate::format::InstrFormat;

/// All opcodes in the FLUX bytecode instruction set.
///
/// Each variant is assigned a fixed byte value matching the FLUX specification.
/// Use [`Op::try_from`] to convert a raw byte into an opcode, and
/// [`Op::format`] to determine the encoding layout for a given instruction.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Op {
    // ── Control flow (0x00–0x0F) ─────────────────────────────────────────

    /// Stop execution. Format A.
    Halt = 0x00,
    /// Do nothing. Format A.
    Nop = 0x01,
    /// Return from the current function. Format A.
    Ret = 0x02,
    /// Unconditional jump to a target address. Format G (payload = 2-byte LE address).
    Jump = 0x03,
    /// Conditional jump (if condition is truthy). Format G.
    JumpIf = 0x04,
    /// Conditional jump (if condition is falsy). Format G.
    JumpIfNot = 0x05,
    /// Call a function by index. Format G (payload = 2-byte LE function index).
    Call = 0x06,
    /// Indirect function call via register. Format G.
    CallIndirect = 0x07,
    /// Yield execution to the scheduler. Format A.
    Yield = 0x08,
    /// Abort with an error message. Format A.
    Panic = 0x09,
    /// Unreachable code guard (traps if executed). Format A.
    Unreachable = 0x0A,

    // ── Stack operations (0x10–0x1F) ─────────────────────────────────────

    /// Push a value onto the stack. Format B.
    Push = 0x10,
    /// Pop the top value from the stack. Format B.
    Pop = 0x11,
    /// Duplicate the top-of-stack value. Format B.
    Dup = 0x12,
    /// Swap two stack values. Format B.
    Swap = 0x13,

    // ── Integer arithmetic (0x20–0x3F) ──────────────────────────────────

    /// Integer move (register-to-register copy). Format B.
    IMov = 0x20,
    /// Integer addition. Format C.
    IAdd = 0x21,
    /// Integer subtraction. Format C.
    ISub = 0x22,
    /// Integer multiplication. Format C.
    IMul = 0x23,
    /// Integer division (signed, traps on divide-by-zero). Format C.
    IDiv = 0x24,
    /// Integer modulo. Format C.
    IMod = 0x25,
    /// Integer negation. Format C.
    INeg = 0x26,
    /// Integer absolute value. Format C.
    IAbs = 0x27,
    /// Increment integer register by immediate. Format D.
    IInc = 0x28,
    /// Decrement integer register by immediate. Format D.
    IDec = 0x29,
    /// Integer minimum. Format C.
    IMin = 0x2A,
    /// Integer maximum. Format C.
    IMax = 0x2B,
    /// Bitwise AND (treated as integer op). Format C.
    IAnd = 0x2C,
    /// Bitwise OR (treated as integer op). Format C.
    IOr = 0x2D,
    /// Bitwise XOR (treated as integer op). Format C.
    IXor = 0x2E,
    /// Shift left. Format C.
    IShl = 0x2F,
    /// Shift right (arithmetic). Format C.
    IShr = 0x30,
    /// Bitwise NOT (treated as integer op). Format C.
    INot = 0x31,
    /// Integer equality comparison. Format C.
    ICmpEq = 0x32,
    /// Integer inequality comparison. Format C.
    ICmpNe = 0x33,
    /// Integer less-than comparison. Format C.
    ICmpLt = 0x34,
    /// Integer less-than-or-equal comparison. Format C.
    ICmpLe = 0x35,
    /// Integer greater-than comparison. Format C.
    ICmpGt = 0x36,
    /// Integer greater-than-or-equal comparison. Format C.
    ICmpGe = 0x37,

    // ── Float arithmetic (0x40–0x5F) ────────────────────────────────────

    /// Float move (register-to-register copy). Format B.
    FMov = 0x40,
    /// Float addition. Format C.
    FAdd = 0x41,
    /// Float subtraction. Format C.
    FSub = 0x42,
    /// Float multiplication. Format C.
    FMul = 0x43,
    /// Float division. Format C.
    FDiv = 0x44,
    /// Float modulo. Format C.
    FMod = 0x45,
    /// Float negation. Format C.
    FNeg = 0x46,
    /// Float absolute value. Format C.
    FAbs = 0x47,
    /// Float square root. Format C.
    FSqrt = 0x48,
    /// Float floor. Format C.
    FFloor = 0x49,
    /// Float ceiling. Format C.
    FCeil = 0x4A,
    /// Float round to nearest integer. Format C.
    FRound = 0x4B,
    /// Float minimum. Format C.
    FMin = 0x4C,
    /// Float maximum. Format C.
    FMax = 0x4D,
    /// Float sine. Format C.
    FSin = 0x4E,
    /// Float cosine. Format C.
    FCos = 0x4F,
    /// Float exponential (e^x). Format C.
    FExp = 0x50,
    /// Float natural logarithm. Format C.
    FLog = 0x51,
    /// Float clamp to range. Format C.
    FClamp = 0x52,
    /// Float linear interpolation. Format C.
    FLerp = 0x53,
    /// Float equality comparison. Format C.
    FCmpEq = 0x54,
    /// Float inequality comparison. Format C.
    FCmpNe = 0x55,
    /// Float less-than comparison. Format C.
    FCmpLt = 0x56,
    /// Float less-than-or-equal comparison. Format C.
    FCmpLe = 0x57,
    /// Float greater-than comparison. Format C.
    FCmpGt = 0x58,
    /// Float greater-than-or-equal comparison. Format C.
    FCmpGe = 0x59,

    // ── Conversions (0x60–0x6F) ─────────────────────────────────────────

    /// Integer to float conversion. Format C.
    IToF = 0x60,
    /// Float to integer conversion (truncating). Format C.
    FToI = 0x61,
    /// Boolean to integer conversion. Format C.
    BToI = 0x62,
    /// Integer to boolean conversion. Format C.
    IToB = 0x63,

    // ── Memory (0x70–0x7F) ──────────────────────────────────────────────

    /// Load an unsigned 8-bit value from memory. Format E.
    Load8 = 0x70,
    /// Load an unsigned 16-bit value from memory. Format E.
    Load16 = 0x71,
    /// Load an unsigned 32-bit value from memory. Format E.
    Load32 = 0x72,
    /// Load an unsigned 64-bit value from memory. Format E.
    Load64 = 0x73,
    /// Store an 8-bit value to memory. Format E.
    Store8 = 0x74,
    /// Store a 16-bit value to memory. Format E.
    Store16 = 0x75,
    /// Store a 32-bit value to memory. Format E.
    Store32 = 0x76,
    /// Store a 64-bit value to memory. Format E.
    Store64 = 0x77,
    /// Load the address of a memory location. Format E.
    LoadAddr = 0x78,
    /// Allocate stack space (in bytes). Format D.
    StackAlloc = 0x79,

    // ── Agent-to-Agent communication (0x80–0x8F) ───────────────────────

    /// Send a message to another agent. Format G.
    ASend = 0x80,
    /// Receive a message from another agent. Format G.
    ARecv = 0x81,
    /// Ask another agent a question. Format G.
    AAsk = 0x82,
    /// Tell another agent something. Format G.
    ATell = 0x83,
    /// Delegate a task to another agent. Format G.
    ADelegate = 0x84,
    /// Broadcast a message to all agents. Format G.
    ABroadcast = 0x85,
    /// Subscribe to a message channel. Format G.
    ASubscribe = 0x86,
    /// Wait for a condition or event. Format G.
    AWait = 0x87,
    /// Establish trust with another agent. Format G.
    ATrust = 0x88,
    /// Verify the trust level of another agent. Format G.
    AVerify = 0x89,

    // ── Type / Meta (0x90–0x9F) ────────────────────────────────────────

    /// Cast a value to a different type. Format C.
    Cast = 0x90,
    /// Compute the size of a type in bytes. Format C.
    SizeOf = 0x91,
    /// Get the runtime type tag of a value. Format C.
    TypeOf = 0x92,

    // ── Bitwise (0xA0–0xAF) ─────────────────────────────────────────────

    /// Bitwise AND. Format C.
    BAnd = 0xA0,
    /// Bitwise OR. Format C.
    BOr = 0xA1,
    /// Bitwise XOR. Format C.
    BXor = 0xA2,
    /// Bitwise shift left. Format C.
    BShl = 0xA3,
    /// Bitwise shift right. Format C.
    BShr = 0xA4,
    /// Bitwise NOT. Format C.
    BNot = 0xA5,

    // ── Vector operations (0xB0–0xBF) ───────────────────────────────────

    /// Load a vector component. Format E.
    VLoad = 0xB0,
    /// Store a vector component. Format E.
    VStore = 0xB1,
    /// Vector addition. Format C.
    VAdd = 0xB2,
    /// Vector multiplication (component-wise). Format C.
    VMul = 0xB3,
    /// Vector dot product. Format C.
    VDot = 0xB4,
}

// ─────────────────────────────────────────────────────────────────────────
// TryFrom<u8> — parse a raw byte into an Op
// ─────────────────────────────────────────────────────────────────────────

impl TryFrom<u8> for Op {
    type Error = u8;

    /// Attempts to convert a raw byte into an [`Op`].
    ///
    /// Returns the byte value back as an error if it does not correspond to
    /// any defined opcode.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        // We match exhaustively on every defined variant so the compiler
        // catches it when new opcodes are added.
        match value {
            0x00 => Ok(Op::Halt),
            0x01 => Ok(Op::Nop),
            0x02 => Ok(Op::Ret),
            0x03 => Ok(Op::Jump),
            0x04 => Ok(Op::JumpIf),
            0x05 => Ok(Op::JumpIfNot),
            0x06 => Ok(Op::Call),
            0x07 => Ok(Op::CallIndirect),
            0x08 => Ok(Op::Yield),
            0x09 => Ok(Op::Panic),
            0x0A => Ok(Op::Unreachable),

            0x10 => Ok(Op::Push),
            0x11 => Ok(Op::Pop),
            0x12 => Ok(Op::Dup),
            0x13 => Ok(Op::Swap),

            0x20 => Ok(Op::IMov),
            0x21 => Ok(Op::IAdd),
            0x22 => Ok(Op::ISub),
            0x23 => Ok(Op::IMul),
            0x24 => Ok(Op::IDiv),
            0x25 => Ok(Op::IMod),
            0x26 => Ok(Op::INeg),
            0x27 => Ok(Op::IAbs),
            0x28 => Ok(Op::IInc),
            0x29 => Ok(Op::IDec),
            0x2A => Ok(Op::IMin),
            0x2B => Ok(Op::IMax),
            0x2C => Ok(Op::IAnd),
            0x2D => Ok(Op::IOr),
            0x2E => Ok(Op::IXor),
            0x2F => Ok(Op::IShl),
            0x30 => Ok(Op::IShr),
            0x31 => Ok(Op::INot),
            0x32 => Ok(Op::ICmpEq),
            0x33 => Ok(Op::ICmpNe),
            0x34 => Ok(Op::ICmpLt),
            0x35 => Ok(Op::ICmpLe),
            0x36 => Ok(Op::ICmpGt),
            0x37 => Ok(Op::ICmpGe),

            0x40 => Ok(Op::FMov),
            0x41 => Ok(Op::FAdd),
            0x42 => Ok(Op::FSub),
            0x43 => Ok(Op::FMul),
            0x44 => Ok(Op::FDiv),
            0x45 => Ok(Op::FMod),
            0x46 => Ok(Op::FNeg),
            0x47 => Ok(Op::FAbs),
            0x48 => Ok(Op::FSqrt),
            0x49 => Ok(Op::FFloor),
            0x4A => Ok(Op::FCeil),
            0x4B => Ok(Op::FRound),
            0x4C => Ok(Op::FMin),
            0x4D => Ok(Op::FMax),
            0x4E => Ok(Op::FSin),
            0x4F => Ok(Op::FCos),
            0x50 => Ok(Op::FExp),
            0x51 => Ok(Op::FLog),
            0x52 => Ok(Op::FClamp),
            0x53 => Ok(Op::FLerp),
            0x54 => Ok(Op::FCmpEq),
            0x55 => Ok(Op::FCmpNe),
            0x56 => Ok(Op::FCmpLt),
            0x57 => Ok(Op::FCmpLe),
            0x58 => Ok(Op::FCmpGt),
            0x59 => Ok(Op::FCmpGe),

            0x60 => Ok(Op::IToF),
            0x61 => Ok(Op::FToI),
            0x62 => Ok(Op::BToI),
            0x63 => Ok(Op::IToB),

            0x70 => Ok(Op::Load8),
            0x71 => Ok(Op::Load16),
            0x72 => Ok(Op::Load32),
            0x73 => Ok(Op::Load64),
            0x74 => Ok(Op::Store8),
            0x75 => Ok(Op::Store16),
            0x76 => Ok(Op::Store32),
            0x77 => Ok(Op::Store64),
            0x78 => Ok(Op::LoadAddr),
            0x79 => Ok(Op::StackAlloc),

            0x80 => Ok(Op::ASend),
            0x81 => Ok(Op::ARecv),
            0x82 => Ok(Op::AAsk),
            0x83 => Ok(Op::ATell),
            0x84 => Ok(Op::ADelegate),
            0x85 => Ok(Op::ABroadcast),
            0x86 => Ok(Op::ASubscribe),
            0x87 => Ok(Op::AWait),
            0x88 => Ok(Op::ATrust),
            0x89 => Ok(Op::AVerify),

            0x90 => Ok(Op::Cast),
            0x91 => Ok(Op::SizeOf),
            0x92 => Ok(Op::TypeOf),

            0xA0 => Ok(Op::BAnd),
            0xA1 => Ok(Op::BOr),
            0xA2 => Ok(Op::BXor),
            0xA3 => Ok(Op::BShl),
            0xA4 => Ok(Op::BShr),
            0xA5 => Ok(Op::BNot),

            0xB0 => Ok(Op::VLoad),
            0xB1 => Ok(Op::VStore),
            0xB2 => Ok(Op::VAdd),
            0xB3 => Ok(Op::VMul),
            0xB4 => Ok(Op::VDot),

            _ => Err(value),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Op methods
// ─────────────────────────────────────────────────────────────────────────

impl Op {
    /// Returns the raw byte value of this opcode.
    #[must_use]
    pub const fn byte(&self) -> u8 {
        *self as u8
    }

    /// Returns `true` if this opcode is a terminator (ends basic-block control flow).
    ///
    /// Terminators: `Halt`, `Ret`, `Panic`, `Unreachable`.
    #[must_use]
    pub const fn is_terminator(&self) -> bool {
        matches!(self, Op::Halt | Op::Ret | Op::Panic | Op::Unreachable)
    }

    /// Returns `true` if this opcode is a branch (may transfer control elsewhere).
    ///
    /// Branches: `Jump`, `JumpIf`, `JumpIfNot`, `Call`, `CallIndirect`, `Yield`.
    #[must_use]
    pub const fn is_branch(&self) -> bool {
        matches!(
            self,
            Op::Jump
                | Op::JumpIf
                | Op::JumpIfNot
                | Op::Call
                | Op::CallIndirect
                | Op::Yield
        )
    }

    /// Returns `true` if this opcode is an agent-to-agent communication primitive.
    #[must_use]
    pub const fn is_a2a(&self) -> bool {
        matches!(
            self,
            Op::ASend
                | Op::ARecv
                | Op::AAsk
                | Op::ATell
                | Op::ADelegate
                | Op::ABroadcast
                | Op::ASubscribe
                | Op::AWait
                | Op::ATrust
                | Op::AVerify
        )
    }

    /// Returns the instruction encoding format for this opcode.
    ///
    /// Each opcode is statically assigned exactly one of the six
    /// [`InstrFormat`] variants.
    #[must_use]
    pub const fn format(&self) -> InstrFormat {
        match self {
            // Format A — nullary
            Op::Halt
            | Op::Nop
            | Op::Ret
            | Op::Panic
            | Op::Unreachable
            | Op::Yield => InstrFormat::A,

            // Format B — two registers
            Op::Push
            | Op::Pop
            | Op::Dup
            | Op::Swap
            | Op::IMov
            | Op::FMov => InstrFormat::B,

            // Format C — two registers + type tag
            Op::IAdd
            | Op::ISub
            | Op::IMul
            | Op::IDiv
            | Op::IMod
            | Op::INeg
            | Op::IAbs
            | Op::IMin
            | Op::IMax
            | Op::IAnd
            | Op::IOr
            | Op::IXor
            | Op::IShl
            | Op::IShr
            | Op::INot
            | Op::ICmpEq
            | Op::ICmpNe
            | Op::ICmpLt
            | Op::ICmpLe
            | Op::ICmpGt
            | Op::ICmpGe
            | Op::FAdd
            | Op::FSub
            | Op::FMul
            | Op::FDiv
            | Op::FMod
            | Op::FNeg
            | Op::FAbs
            | Op::FSqrt
            | Op::FFloor
            | Op::FCeil
            | Op::FRound
            | Op::FMin
            | Op::FMax
            | Op::FSin
            | Op::FCos
            | Op::FExp
            | Op::FLog
            | Op::FClamp
            | Op::FLerp
            | Op::FCmpEq
            | Op::FCmpNe
            | Op::FCmpLt
            | Op::FCmpLe
            | Op::FCmpGt
            | Op::FCmpGe
            | Op::IToF
            | Op::FToI
            | Op::BToI
            | Op::IToB
            | Op::Cast
            | Op::SizeOf
            | Op::TypeOf
            | Op::BAnd
            | Op::BOr
            | Op::BXor
            | Op::BShl
            | Op::BShr
            | Op::BNot
            | Op::VAdd
            | Op::VMul
            | Op::VDot => InstrFormat::C,

            // Format D — register + 16-bit immediate
            Op::IInc | Op::IDec | Op::StackAlloc => InstrFormat::D,

            // Format E — two registers + 16-bit offset
            Op::Load8
            | Op::Load16
            | Op::Load32
            | Op::Load64
            | Op::Store8
            | Op::Store16
            | Op::Store32
            | Op::Store64
            | Op::LoadAddr
            | Op::VLoad
            | Op::VStore => InstrFormat::E,

            // Format G — variable-length payload
            Op::Jump
            | Op::JumpIf
            | Op::JumpIfNot
            | Op::Call
            | Op::CallIndirect
            | Op::ASend
            | Op::ARecv
            | Op::AAsk
            | Op::ATell
            | Op::ADelegate
            | Op::ABroadcast
            | Op::ASubscribe
            | Op::AWait
            | Op::ATrust
            | Op::AVerify => InstrFormat::G,
        }
    }

    /// Returns the mnemonic (human-readable name) of this opcode as a static string slice.
    #[must_use]
    pub const fn mnemonic(&self) -> &'static str {
        match self {
            Op::Halt => "halt",
            Op::Nop => "nop",
            Op::Ret => "ret",
            Op::Jump => "jump",
            Op::JumpIf => "jump_if",
            Op::JumpIfNot => "jump_if_not",
            Op::Call => "call",
            Op::CallIndirect => "call_indirect",
            Op::Yield => "yield",
            Op::Panic => "panic",
            Op::Unreachable => "unreachable",
            Op::Push => "push",
            Op::Pop => "pop",
            Op::Dup => "dup",
            Op::Swap => "swap",
            Op::IMov => "imov",
            Op::IAdd => "iadd",
            Op::ISub => "isub",
            Op::IMul => "imul",
            Op::IDiv => "idiv",
            Op::IMod => "imod",
            Op::INeg => "ineg",
            Op::IAbs => "iabs",
            Op::IInc => "iinc",
            Op::IDec => "idec",
            Op::IMin => "imin",
            Op::IMax => "imax",
            Op::IAnd => "iand",
            Op::IOr => "ior",
            Op::IXor => "ixor",
            Op::IShl => "ishl",
            Op::IShr => "ishr",
            Op::INot => "inot",
            Op::ICmpEq => "icmp_eq",
            Op::ICmpNe => "icmp_ne",
            Op::ICmpLt => "icmp_lt",
            Op::ICmpLe => "icmp_le",
            Op::ICmpGt => "icmp_gt",
            Op::ICmpGe => "icmp_ge",
            Op::FMov => "fmov",
            Op::FAdd => "fadd",
            Op::FSub => "fsub",
            Op::FMul => "fmul",
            Op::FDiv => "fdiv",
            Op::FMod => "fmod",
            Op::FNeg => "fneg",
            Op::FAbs => "fabs",
            Op::FSqrt => "fsqrt",
            Op::FFloor => "ffloor",
            Op::FCeil => "fceil",
            Op::FRound => "fround",
            Op::FMin => "fmin",
            Op::FMax => "fmax",
            Op::FSin => "fsin",
            Op::FCos => "fcos",
            Op::FExp => "fexp",
            Op::FLog => "flog",
            Op::FClamp => "fclamp",
            Op::FLerp => "flerp",
            Op::FCmpEq => "fcmp_eq",
            Op::FCmpNe => "fcmp_ne",
            Op::FCmpLt => "fcmp_lt",
            Op::FCmpLe => "fcmp_le",
            Op::FCmpGt => "fcmp_gt",
            Op::FCmpGe => "fcmp_ge",
            Op::IToF => "i2f",
            Op::FToI => "f2i",
            Op::BToI => "b2i",
            Op::IToB => "i2b",
            Op::Load8 => "load8",
            Op::Load16 => "load16",
            Op::Load32 => "load32",
            Op::Load64 => "load64",
            Op::Store8 => "store8",
            Op::Store16 => "store16",
            Op::Store32 => "store32",
            Op::Store64 => "store64",
            Op::LoadAddr => "load_addr",
            Op::StackAlloc => "stack_alloc",
            Op::ASend => "a_send",
            Op::ARecv => "a_recv",
            Op::AAsk => "a_ask",
            Op::ATell => "a_tell",
            Op::ADelegate => "a_delegate",
            Op::ABroadcast => "a_broadcast",
            Op::ASubscribe => "a_subscribe",
            Op::AWait => "a_wait",
            Op::ATrust => "a_trust",
            Op::AVerify => "a_verify",
            Op::Cast => "cast",
            Op::SizeOf => "sizeof",
            Op::TypeOf => "typeof",
            Op::BAnd => "band",
            Op::BOr => "bor",
            Op::BXor => "bxor",
            Op::BShl => "bshl",
            Op::BShr => "bshr",
            Op::BNot => "bnot",
            Op::VLoad => "vload",
            Op::VStore => "vstore",
            Op::VAdd => "vadd",
            Op::VMul => "vmul",
            Op::VDot => "vdot",
        }
    }

    /// Returns the category name for this opcode.
    #[must_use]
    pub const fn category(&self) -> &'static str {
        match self {
            Op::Halt
            | Op::Nop
            | Op::Ret
            | Op::Jump
            | Op::JumpIf
            | Op::JumpIfNot
            | Op::Call
            | Op::CallIndirect
            | Op::Yield
            | Op::Panic
            | Op::Unreachable => "control",

            Op::Push | Op::Pop | Op::Dup | Op::Swap => "stack",

            Op::IMov
            | Op::IAdd
            | Op::ISub
            | Op::IMul
            | Op::IDiv
            | Op::IMod
            | Op::INeg
            | Op::IAbs
            | Op::IInc
            | Op::IDec
            | Op::IMin
            | Op::IMax
            | Op::IAnd
            | Op::IOr
            | Op::IXor
            | Op::IShl
            | Op::IShr
            | Op::INot
            | Op::ICmpEq
            | Op::ICmpNe
            | Op::ICmpLt
            | Op::ICmpLe
            | Op::ICmpGt
            | Op::ICmpGe => "integer",

            Op::FMov
            | Op::FAdd
            | Op::FSub
            | Op::FMul
            | Op::FDiv
            | Op::FMod
            | Op::FNeg
            | Op::FAbs
            | Op::FSqrt
            | Op::FFloor
            | Op::FCeil
            | Op::FRound
            | Op::FMin
            | Op::FMax
            | Op::FSin
            | Op::FCos
            | Op::FExp
            | Op::FLog
            | Op::FClamp
            | Op::FLerp
            | Op::FCmpEq
            | Op::FCmpNe
            | Op::FCmpLt
            | Op::FCmpLe
            | Op::FCmpGt
            | Op::FCmpGe => "float",

            Op::IToF | Op::FToI | Op::BToI | Op::IToB => "conversion",

            Op::Load8
            | Op::Load16
            | Op::Load32
            | Op::Load64
            | Op::Store8
            | Op::Store16
            | Op::Store32
            | Op::Store64
            | Op::LoadAddr
            | Op::StackAlloc => "memory",

            Op::ASend
            | Op::ARecv
            | Op::AAsk
            | Op::ATell
            | Op::ADelegate
            | Op::ABroadcast
            | Op::ASubscribe
            | Op::AWait
            | Op::ATrust
            | Op::AVerify => "a2a",

            Op::Cast | Op::SizeOf | Op::TypeOf => "meta",

            Op::BAnd | Op::BOr | Op::BXor | Op::BShl | Op::BShr | Op::BNot => "bitwise",

            Op::VLoad | Op::VStore | Op::VAdd | Op::VMul | Op::VDot => "vector",
        }
    }

    /// Returns the total number of defined opcodes.
    #[must_use]
    pub const fn count() -> usize {
        100
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mnemonic())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_roundtrip() {
        let all: &[Op] = &[
            Op::Halt, Op::Nop, Op::Ret, Op::Jump, Op::Call,
            Op::Push, Op::Pop, Op::IMov, Op::IAdd, Op::FMov,
            Op::FAdd, Op::IToF, Op::FToI, Op::Load32, Op::Store64,
            Op::ASend, Op::ARecv, Op::Cast, Op::SizeOf, Op::BAnd,
            Op::VLoad, Op::VDot,
        ];
        for op in all {
            let byte = op.byte();
            assert_eq!(Op::try_from(byte).unwrap(), *op, "roundtrip failed for {:?}", op);
        }
    }

    #[test]
    fn invalid_byte() {
        assert!(Op::try_from(0x0B).is_err());
        assert!(Op::try_from(0xFF).is_err());
    }

    #[test]
    fn all_opcodes_have_format() {
        // Every opcode value that is valid must have a format.
        for byte in 0u8..=255 {
            if let Ok(op) = Op::try_from(byte) {
                let _ = op.format();
            }
        }
    }

    #[test]
    fn terminators() {
        assert!(Op::Halt.is_terminator());
        assert!(Op::Ret.is_terminator());
        assert!(Op::Panic.is_terminator());
        assert!(Op::Unreachable.is_terminator());
        assert!(!Op::Jump.is_terminator());
        assert!(!Op::Nop.is_terminator());
    }

    #[test]
    fn branches() {
        assert!(Op::Jump.is_branch());
        assert!(Op::Call.is_branch());
        assert!(Op::Yield.is_branch());
        assert!(!Op::Halt.is_branch());
    }

    #[test]
    fn a2a() {
        assert!(Op::ASend.is_a2a());
        assert!(Op::AWait.is_a2a());
        assert!(!Op::Push.is_a2a());
    }

    #[test]
    fn categories() {
        assert_eq!(Op::Halt.category(), "control");
        assert_eq!(Op::Push.category(), "stack");
        assert_eq!(Op::IAdd.category(), "integer");
        assert_eq!(Op::FAdd.category(), "float");
        assert_eq!(Op::IToF.category(), "conversion");
        assert_eq!(Op::Load32.category(), "memory");
        assert_eq!(Op::ASend.category(), "a2a");
        assert_eq!(Op::Cast.category(), "meta");
        assert_eq!(Op::BAnd.category(), "bitwise");
        assert_eq!(Op::VAdd.category(), "vector");
    }
}
