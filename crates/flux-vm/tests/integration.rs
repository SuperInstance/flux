//! Integration tests for the FLUX virtual machine.
//!
//! All bytecode is constructed via `flux_bytecode::BytecodeEncoder` and
//! `Instruction` so that the wire format is guaranteed to match what the
//! interpreter expects.

use flux_bytecode::{BytecodeEncoder, BytecodeHeader, Instruction, Op};
use flux_vm::{Interpreter, VmConfig, VmError};

// ── Bytecode helpers ───────────────────────────────────────────

/// Encode a slice of `Instruction` values into raw bytecode bytes
/// (no header — the VM fetches from the first byte).
fn encode(instrs: &[Instruction]) -> Vec<u8> {
    let mut enc = BytecodeEncoder::new();
    for i in instrs {
        enc.emit(i).unwrap();
    }
    enc.into_bytes()
}

/// Helper: build an i64 payload for Format-G instructions (8 bytes LE).
fn i64_payload(val: i64) -> Vec<u8> {
    val.to_le_bytes().to_vec()
}

/// Create a default VM config.
fn default_config() -> VmConfig {
    VmConfig::default()
}

/// Create a VM from raw bytecode bytes.
fn make_vm(bytecode: &[u8]) -> Interpreter {
    Interpreter::new(bytecode, default_config()).expect("failed to create VM")
}

/// Create a VM with a custom config.
fn make_vm_cfg(bytecode: &[u8], config: VmConfig) -> Interpreter {
    Interpreter::new(bytecode, config).expect("failed to create VM")
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

// ── 1. HALT immediately ────────────────────────────────────────

#[test]
fn halt_immediately() {
    let bc = encode(&[Instruction::nullary(Op::Halt)]);
    let mut vm = make_vm(&bc);
    let cycles = vm.execute().unwrap();
    assert!(vm.is_halted());
    assert!(cycles >= 1);
}

// ── 2. NOP then HALT ───────────────────────────────────────────

#[test]
fn nop_then_halt() {
    let bc = encode(&[
        Instruction::nullary(Op::Nop),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert!(vm.is_halted());
    assert_eq!(vm.regs.cycles(), 2); // 1 NOP + 1 HALT
}

// ── 3. IINC (load immediate via increment from 0) ─────────────

#[test]
fn iinc_sets_register() {
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 42),  // R1 = 0 + 42 = 42
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(1), 42);
}

// ── 4. IINC negative value ────────────────────────────────────

#[test]
fn iinc_negative_value() {
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, -10), // R1 = 0 + (-10) = -10
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(1), -10);
}

// ── 5. IMOV basic (register-to-register copy) ──────────────────

#[test]
fn imov_basic() {
    // IInc R1, 77; IMov R0, R1; HALT → R0 == 77
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 77),
        Instruction::reg(Op::IMov, 0, 1),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 77);
}

// ── 6. IADD basic ─────────────────────────────────────────────

#[test]
fn iadd_basic() {
    // IInc R1, 10; IInc R2, 20; IADD R0, R1, R2; HALT → R0 == 30
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 10),
        Instruction::imm(Op::IInc, 2, 20),
        Instruction::reg_ty(Op::IAdd, 0, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 30);
}

// ── 7. ISUB basic ─────────────────────────────────────────────

#[test]
fn isub_basic() {
    // IInc R1, 30; IInc R2, 10; ISUB R0, R1, R2; HALT → R0 == 20
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 30),
        Instruction::imm(Op::IInc, 2, 10),
        Instruction::reg_ty(Op::ISub, 0, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 20);
}

// ── 8. IMUL basic ─────────────────────────────────────────────

#[test]
fn imul_basic() {
    // IInc R1, 7; IInc R2, 3; IMUL R0, R1, R2; HALT → R0 == 21
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 7),
        Instruction::imm(Op::IInc, 2, 3),
        Instruction::reg_ty(Op::IMul, 0, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 21);
}

// ── 9. IDIV basic ─────────────────────────────────────────────

#[test]
fn idiv_basic() {
    // IInc R1, 100; IInc R2, 7; IDIV R0, R1, R2; HALT → R0 == 14
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 100),
        Instruction::imm(Op::IInc, 2, 7),
        Instruction::reg_ty(Op::IDiv, 0, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 14);
}

// ── 10. Division by zero ───────────────────────────────────────

#[test]
fn idiv_by_zero() {
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 100),
        Instruction::imm(Op::IInc, 2, 0),
        Instruction::reg_ty(Op::IDiv, 0, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    let result = vm.execute();
    assert!(matches!(result, Err(VmError::DivisionByZero)));
}

// ── 11. IREM (IMod) basic ──────────────────────────────────────

#[test]
fn irem_basic() {
    // IInc R1, 17; IInc R2, 5; IMod R0, R1, R2; HALT → R0 == 2
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 17),
        Instruction::imm(Op::IInc, 2, 5),
        Instruction::reg_ty(Op::IMod, 0, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 2);
}

// ── 12. INEG basic ─────────────────────────────────────────────

#[test]
fn ineg_basic() {
    // IInc R1, -10; INeg R0, R1; HALT → R0 == 10
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, -10),
        Instruction::reg_ty(Op::INeg, 0, 1, 0), // Format C: dst=0, src=1, type_tag=0(unused)
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 10);
}

// ── 13. Bitwise operations ─────────────────────────────────────

#[test]
fn bitwise_ops() {
    // IInc R1, 0xFF; IInc R2, 0x0F; IAND R3, R1, R2; IOR R4, R1, R2; IXOR R5, R1, R2; HALT
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 0xFF),
        Instruction::imm(Op::IInc, 2, 0x0F),
        Instruction::reg_ty(Op::IAnd, 3, 1, 2),
        Instruction::reg_ty(Op::IOr, 4, 1, 2),
        Instruction::reg_ty(Op::IXor, 5, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(3), 0x0F); // AND
    assert_eq!(vm.regs.read_gp(4), 0xFF); // OR
    assert_eq!(vm.regs.read_gp(5), 0xF0); // XOR
}

// ── 14. Shift operations ───────────────────────────────────────

#[test]
fn shift_ops() {
    // IInc R1, 1; IInc R2, 4; ISHL R3, R1, R2; HALT → R3 = 16
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 1),
        Instruction::imm(Op::IInc, 2, 4),
        Instruction::reg_ty(Op::IShl, 3, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(3), 16);
}

// ── 15. ICMP equal (true) ──────────────────────────────────────

#[test]
fn icmp_equal_true() {
    // IInc R1, 5; ICmpEq R0, R1, R1; HALT → R0 == 1, ZERO flag set
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 5),
        Instruction::reg_ty(Op::ICmpEq, 0, 1, 1),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 1);
    assert!(vm.regs.flags().is_zero());
}

// ── 16. ICMP equal (false) ─────────────────────────────────────

#[test]
fn icmp_equal_false() {
    // IInc R1, 5; IInc R2, 3; ICmpEq R0, R1, R2; HALT → R0 == 0, ZERO not set
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 5),
        Instruction::imm(Op::IInc, 2, 3),
        Instruction::reg_ty(Op::ICmpEq, 0, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 0);
    assert!(!vm.regs.flags().is_zero());
}

// ── 17. ICMP less-than ─────────────────────────────────────────

#[test]
fn icmp_less_than() {
    // IInc R1, 3; IInc R2, 7; ICmpLt R0, R1, R2; HALT → R0 == 1
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 3),
        Instruction::imm(Op::IInc, 2, 7),
        Instruction::reg_ty(Op::ICmpLt, 0, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 1);
    assert!(vm.regs.flags().is_zero());
}

// ── 18. ICMP greater-than ──────────────────────────────────────

#[test]
fn icmp_greater_than() {
    // IInc R1, 7; IInc R2, 3; ICmpGt R0, R1, R2; HALT → R0 == 1
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 7),
        Instruction::imm(Op::IInc, 2, 3),
        Instruction::reg_ty(Op::ICmpGt, 0, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 1);
    assert!(vm.regs.flags().is_zero());
}

// ── 19. ICMP less-than-or-equal (true) ─────────────────────────

#[test]
fn icmp_le_true() {
    // IInc R1, 5; IInc R2, 5; ICmpLe R0, R1, R2; HALT → R0 == 1
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 5),
        Instruction::imm(Op::IInc, 2, 5),
        Instruction::reg_ty(Op::ICmpLe, 0, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 1);
}

// ── 20. ICMP greater-than-or-equal (true) ──────────────────────

#[test]
fn icmp_ge_true() {
    // IInc R1, 5; IInc R2, 3; ICmpGe R0, R1, R2; HALT → R0 == 1
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 5),
        Instruction::imm(Op::IInc, 2, 3),
        Instruction::reg_ty(Op::ICmpGe, 0, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 1);
}

// ── 21. ICMP not-equal ─────────────────────────────────────────

#[test]
fn icmp_ne_true() {
    // IInc R1, 5; IInc R2, 3; ICmpNe R0, R1, R2; HALT → R0 == 1
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 5),
        Instruction::imm(Op::IInc, 2, 3),
        Instruction::reg_ty(Op::ICmpNe, 0, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 1);
    assert!(vm.regs.flags().is_zero());
}

// ── 22. JUMP_IF taken ──────────────────────────────────────────

#[test]
fn jump_if_taken() {
    // offset 0:  IInc R1, 5              (4 bytes, Format D)
    // offset 4:  ICmpEq R3, R1, R1      (4 bytes, Format C) → ZERO set
    // offset 8:  JumpIf +5               (10 bytes, Format G, 8-byte payload)
    //   → PC after decode = offset 18
    //   → target = 18 + 5 = 23
    // offset 18: IInc R0, 0             (4 bytes, Format D) — fall-through (skipped)
    // offset 22: Halt                    (1 byte, Format A) — skipped
    // offset 23: IInc R0, 1             (4 bytes, Format D) — jump target
    // offset 27: Halt                    (1 byte, Format A)
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 5),         // 0
        Instruction::reg_ty(Op::ICmpEq, 3, 1, 1), // 4 — write to R3, not R0
        Instruction::var(Op::JumpIf, i64_payload(5)), // 8
        Instruction::imm(Op::IInc, 0, 0),          // 18 — skipped
        Instruction::nullary(Op::Halt),            // 22 — skipped
        Instruction::imm(Op::IInc, 0, 1),          // 23 — jump target
        Instruction::nullary(Op::Halt),            // 27
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 1);
}

// ── 23. JUMP_IF not taken ──────────────────────────────────────

#[test]
fn jump_if_falls_through() {
    // offset 0:  IInc R1, 5             (4 bytes)
    // offset 4:  IInc R2, 3             (4 bytes)
    // offset 8:  ICmpEq R0, R1, R2     (4 bytes) → ZERO NOT set
    // offset 12: JumpIf +5              (10 bytes) → should NOT jump
    // offset 22: IInc R0, 99            (4 bytes) — executed
    // offset 26: Halt                   (1 byte)
    // offset 27: IInc R0, 0             (4 bytes) — not reached
    // offset 31: Halt                   (1 byte)
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 5),
        Instruction::imm(Op::IInc, 2, 3),
        Instruction::reg_ty(Op::ICmpEq, 0, 1, 2),
        Instruction::var(Op::JumpIf, i64_payload(5)),
        Instruction::imm(Op::IInc, 0, 99),      // executed
        Instruction::nullary(Op::Halt),
        Instruction::imm(Op::IInc, 0, 0),       // not reached
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 99);
}

// ── 24. JUMP_IF_NOT taken ──────────────────────────────────────

#[test]
fn jump_if_not_taken() {
    // IInc R1, 5; IInc R2, 3; ICmpEq R0, R1, R2 → ZERO NOT set (5 != 3)
    // JumpIfNot +5 → !ZERO = true → jumps
    // offset 0:  IInc R1, 5             (4 bytes)
    // offset 4:  IInc R2, 3             (4 bytes)
    // offset 8:  ICmpEq R0, R1, R2     (4 bytes) → ZERO not set
    // offset 12: JumpIfNot +5           (10 bytes) → jumps
    // offset 22: IInc R0, 0             (4 bytes) — skipped
    // offset 26: Halt                   (1 byte) — skipped
    // offset 27: IInc R0, 1             (4 bytes) — jump target
    // offset 31: Halt                   (1 byte)
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 5),
        Instruction::imm(Op::IInc, 2, 3),
        Instruction::reg_ty(Op::ICmpEq, 0, 1, 2),
        Instruction::var(Op::JumpIfNot, i64_payload(5)),
        Instruction::imm(Op::IInc, 0, 0),       // skipped
        Instruction::nullary(Op::Halt),         // skipped
        Instruction::imm(Op::IInc, 0, 1),       // jump target
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 1);
}

// ── 25. CALL and RET ───────────────────────────────────────────

#[test]
fn call_and_ret() {
    // offset 0:  IInc R1, 10            (4 bytes)
    // offset 4:  Call +1                (10 bytes) → PC after = 14, target = 15
    // offset 14: Halt                   (1 byte) — return here
    // offset 15: IInc R2, 32            (4 bytes)
    // offset 19: IAdd R1, R1, R2       (4 bytes) — R1 = 10 + 32 = 42
    // offset 23: Ret                    (1 byte)
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 10),
        Instruction::var(Op::Call, i64_payload(1)),
        Instruction::nullary(Op::Halt),
        Instruction::imm(Op::IInc, 2, 32),
        Instruction::reg_ty(Op::IAdd, 1, 1, 2),
        Instruction::nullary(Op::Ret),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(1), 42);
    assert!(vm.is_halted());
}

// ── 26. PUSH and POP basic ─────────────────────────────────────

#[test]
fn push_pop_basic() {
    // IInc R1, 42; PUSH R1; POP R0; HALT → R0 == 42
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 42),
        Instruction::reg(Op::Push, 1, 0),
        Instruction::reg(Op::Pop, 0, 0),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 42);
}

// ── 27. PUSH/POP LIFO order ────────────────────────────────────

#[test]
fn push_pop_lifo() {
    // IInc R1, 1; IInc R2, 2; PUSH R1; PUSH R2; POP R0; POP R3; HALT
    // → R0 = 2, R3 = 1
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 1),
        Instruction::imm(Op::IInc, 2, 2),
        Instruction::reg(Op::Push, 1, 0),
        Instruction::reg(Op::Push, 2, 0),
        Instruction::reg(Op::Pop, 0, 0),
        Instruction::reg(Op::Pop, 3, 0),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 2); // last pushed, first popped
    assert_eq!(vm.regs.read_gp(3), 1);
}

// ── 28. SWAP registers ─────────────────────────────────────────

#[test]
fn swap_registers() {
    // IInc R1, 10; IInc R2, 20; SWAP R1, R2; HALT → R1=20, R2=10
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 10),
        Instruction::imm(Op::IInc, 2, 20),
        Instruction::reg(Op::Swap, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(1), 20);
    assert_eq!(vm.regs.read_gp(2), 10);
}

// ── 29. DUP (duplicate top of stack) ───────────────────────────

#[test]
fn dup_stack() {
    // IInc R1, 42; PUSH R1; DUP; POP R0; POP R3; HALT → R0==42, R3==42
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 42),
        Instruction::reg(Op::Push, 1, 0),
        Instruction::reg(Op::Dup, 0, 0),
        Instruction::reg(Op::Pop, 0, 0),
        Instruction::reg(Op::Pop, 3, 0),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 42);
    assert_eq!(vm.regs.read_gp(3), 42);
}

// ── 30. IToF / FToI round-trip ─────────────────────────────────

#[test]
fn itof_ftoi_roundtrip() {
    // IInc R1, 42; IToF F0, R1; FToI R0, F0; HALT → R0 == 42
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 42),
        Instruction::reg_ty(Op::IToF, 0, 1, 0), // Format C: F0 = f64(R1)
        Instruction::reg_ty(Op::FToI, 0, 0, 0), // Format C: R0 = i64(F0)
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 42);
}

// ── 31. FADD basic ─────────────────────────────────────────────

#[test]
fn fadd_basic() {
    // IInc R1, 3; IInc R2, 2; IToF F0, R1; IToF F1, R2; FADD F2, F0, F1; FToI R0, F2; HALT → R0 == 5
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 3),
        Instruction::imm(Op::IInc, 2, 2),
        Instruction::reg_ty(Op::IToF, 0, 1, 0),  // F0 = 3.0
        Instruction::reg_ty(Op::IToF, 1, 2, 0),  // F1 = 2.0
        Instruction::reg_ty(Op::FAdd, 2, 0, 1),   // F2 = 5.0
        Instruction::reg_ty(Op::FToI, 0, 2, 0),   // R0 = 5
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 5);
}

// ── 32. FSUB basic ─────────────────────────────────────────────

#[test]
fn fsub_basic() {
    // IInc R1, 10; IInc R2, 3; IToF F0, R1; IToF F1, R2; FSUB F2, F0, F1; FToI R0, F2; HALT → R0 == 7
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 10),
        Instruction::imm(Op::IInc, 2, 3),
        Instruction::reg_ty(Op::IToF, 0, 1, 0),
        Instruction::reg_ty(Op::IToF, 1, 2, 0),
        Instruction::reg_ty(Op::FSub, 2, 0, 1),
        Instruction::reg_ty(Op::FToI, 0, 2, 0),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 7);
}

// ── 33. FMUL basic ─────────────────────────────────────────────

#[test]
fn fmul_basic() {
    // IInc R1, 7; IInc R2, 3; IToF F0, R1; IToF F1, R2; FMUL F2, F0, F1; FToI R0, F2; HALT → R0 == 21
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 7),
        Instruction::imm(Op::IInc, 2, 3),
        Instruction::reg_ty(Op::IToF, 0, 1, 0),
        Instruction::reg_ty(Op::IToF, 1, 2, 0),
        Instruction::reg_ty(Op::FMul, 2, 0, 1),
        Instruction::reg_ty(Op::FToI, 0, 2, 0),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 21);
}

// ── 34. FDIV basic ─────────────────────────────────────────────

#[test]
fn fdiv_basic() {
    // IInc R1, 21; IInc R2, 3; IToF F0, R1; IToF F1, R2; FDIV F2, F0, F1; FToI R0, F2; HALT → R0 == 7
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 21),
        Instruction::imm(Op::IInc, 2, 3),
        Instruction::reg_ty(Op::IToF, 0, 1, 0),
        Instruction::reg_ty(Op::IToF, 1, 2, 0),
        Instruction::reg_ty(Op::FDiv, 2, 0, 1),
        Instruction::reg_ty(Op::FToI, 0, 2, 0),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 7);
}

// ── 35. FMOV basic ─────────────────────────────────────────────

#[test]
fn fmov_basic() {
    // IInc R1, 99; IToF F0, R1; FMov F1, F0; FToI R0, F1; HALT → R0 == 99
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 99),
        Instruction::reg_ty(Op::IToF, 0, 1, 0), // F0 = 99.0
        Instruction::reg(Op::FMov, 1, 0),        // F1 = F0 (Format B)
        Instruction::reg_ty(Op::FToI, 0, 0, 0), // R0 = 99
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 99);
}

// ── 36. FNEG basic ─────────────────────────────────────────────

#[test]
fn fneg_basic() {
    // IInc R1, 10; IToF F0, R1; FNeg F1, F0; FToI R0, F1; HALT → R0 == -10
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 10),
        Instruction::reg_ty(Op::IToF, 0, 1, 0), // F0 = 10.0
        Instruction::reg_ty(Op::FNeg, 1, 0, 0), // F1 = -10.0
        Instruction::reg_ty(Op::FToI, 0, 1, 0), // R0 = -10
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), -10);
}

// ── 37. Memory store/load via stack (Load64 / Store64) ─────────

#[test]
fn memory_store_load() {
    // IInc R1, 42; PUSH R1; LOAD64 R0, R11, 0; HALT → R0 == 42
    // R11 is aliased to SP. After PUSH, SP points to the pushed value.
    // Load64 uses rd for destination, rs1 for base address, offset for displacement.
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 42),
        Instruction::reg(Op::Push, 1, 0),
        Instruction::mem(Op::Load64, 0, 11, 0), // R0 = mem[R11+0] = mem[SP+0]
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 42);
}

// ── 38. Store64 then Load64 ────────────────────────────────────

#[test]
fn store64_load64() {
    // IInc R1, 42; PUSH R1; IInc R2, 99; STORE64 R2, R11, 0; LOAD64 R0, R11, 0; HALT → R0 == 99
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 42),
        Instruction::reg(Op::Push, 1, 0),              // push 42 to stack
        Instruction::imm(Op::IInc, 2, 99),
        Instruction::mem(Op::Store64, 2, 11, 0),        // mem[SP+0] = R2 = 99
        Instruction::mem(Op::Load64, 0, 11, 0),         // R0 = mem[SP+0] = 99
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 99);
}

// ── 39. Store32 then Load32 ────────────────────────────────────

#[test]
fn store32_load32() {
    // IInc R1, 42; PUSH R1; IInc R2, 99; STORE32 R2, R11, 0; LOAD32 R0, R11, 0; HALT → R0 == 99
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 42),
        Instruction::reg(Op::Push, 1, 0),
        Instruction::imm(Op::IInc, 2, 99),
        Instruction::mem(Op::Store32, 2, 11, 0),
        Instruction::mem(Op::Load32, 0, 11, 0),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 99);
}

// ── 40. Cycle limit enforcement ────────────────────────────────

#[test]
fn cycle_limit_enforced() {
    let config = VmConfig {
        max_cycles: 3,
        ..default_config()
    };
    // NOP; NOP; NOP; NOP; HALT — should stop after 3 cycles
    let bc = encode(&[
        Instruction::nullary(Op::Nop),
        Instruction::nullary(Op::Nop),
        Instruction::nullary(Op::Nop),
        Instruction::nullary(Op::Nop),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm_cfg(&bc, config);
    let result = vm.execute();
    assert!(matches!(result, Err(VmError::CycleLimit(3))));
}

// ── 41. Panic handling ─────────────────────────────────────────

#[test]
fn panic_instruction() {
    let bc = encode(&[
        Instruction::nullary(Op::Panic),
        Instruction::nullary(Op::Halt), // should not reach
    ]);
    let mut vm = make_vm(&bc);
    let result = vm.execute();
    assert!(matches!(result, Err(VmError::Panic(_))));
    assert!(vm.is_panicked());
    assert!(vm.panic_message().is_some());
}

// ── 42. Stack overflow ─────────────────────────────────────────

#[test]
fn stack_overflow() {
    let config = VmConfig {
        stack_size: 64, // 8 slots (8 bytes each)
        ..default_config()
    };
    // Push 9 times — 9th should overflow
    let mut instrs = Vec::new();
    for _ in 0..9 {
        instrs.push(Instruction::reg(Op::Push, 1, 0));
    }
    instrs.push(Instruction::nullary(Op::Halt));
    let bc = encode(&instrs);
    let mut vm = make_vm_cfg(&bc, config);
    let result = vm.execute();
    assert!(matches!(result, Err(VmError::StackOverflow)));
}

// ── 43. Stack underflow ────────────────────────────────────────

#[test]
fn stack_underflow() {
    let bc = encode(&[
        Instruction::reg(Op::Pop, 0, 0),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    let result = vm.execute();
    assert!(matches!(result, Err(VmError::StackUnderflow)));
}

// ── 44. Trace log verification ─────────────────────────────────

#[test]
fn trace_log_captured() {
    let config = VmConfig {
        trace_enabled: true,
        ..default_config()
    };
    let bc = encode(&[
        Instruction::nullary(Op::Nop),
        Instruction::imm(Op::IInc, 1, 42),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm_cfg(&bc, config);
    vm.execute().unwrap();
    let log = vm.trace_log();
    assert_eq!(log.len(), 3);
    assert!(log[0].contains("Nop"));
    assert!(log[1].contains("IInc"));
    assert!(log[2].contains("Halt"));
}

// ── 45. Reset VM ───────────────────────────────────────────────

#[test]
fn reset_vm() {
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 42),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(1), 42);
    assert!(vm.is_halted());

    vm.reset();
    assert_eq!(vm.regs.read_gp(1), 0); // cleared
    assert!(!vm.is_halted());
    assert_eq!(vm.regs.cycles(), 0);

    // Can execute again.
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(1), 42);
}

// ── 46. Unconditional JUMP ─────────────────────────────────────

#[test]
fn unconditional_jump() {
    // JUMP +5 (skip over IInc R0, 0 and HALT)
    // offset 0:  Jump +5              (10 bytes) → target = 10 + 5 = 15
    // offset 10: IInc R0, 0           (4 bytes) — skipped
    // offset 14: Halt                  (1 byte) — skipped
    // offset 15: IInc R0, 1           (4 bytes) — jump target
    // offset 19: Halt                  (1 byte)
    let bc = encode(&[
        Instruction::var(Op::Jump, i64_payload(5)),
        Instruction::imm(Op::IInc, 0, 0),       // skipped
        Instruction::nullary(Op::Halt),         // skipped
        Instruction::imm(Op::IInc, 0, 1),       // jump target
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 1);
}

// ── 47. Send + Sync check (compile-time) ───────────────────────

#[test]
fn vm_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Interpreter>();
    assert_send_sync::<flux_vm::RegisterFile>();
    assert_send_sync::<flux_vm::MemoryManager>();
}

// ── 48. FLAGS update after arithmetic ──────────────────────────

#[test]
fn flags_after_iadd() {
    // IInc R1, -5; IInc R2, 5; IADD R0, R1, R2; HALT
    // Result = 0 → ZERO flag set
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, -5),
        Instruction::imm(Op::IInc, 2, 5),
        Instruction::reg_ty(Op::IAdd, 0, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 0);
    assert!(vm.regs.flags().is_zero());
}

#[test]
fn flags_negative_after_isub() {
    // IInc R1, 3; IInc R2, 10; ISUB R0, R1, R2; HALT → R0 = -7
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 3),
        Instruction::imm(Op::IInc, 2, 10),
        Instruction::reg_ty(Op::ISub, 0, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), -7);
    assert!(vm.regs.flags().is_negative());
    assert!(!vm.regs.flags().is_zero());
}

// ── 49. INEG with zero ─────────────────────────────────────────

#[test]
fn ineg_zero() {
    // IInc R1, 0; INeg R0, R1; HALT → R0 = 0, ZERO flag set
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 0),
        Instruction::reg_ty(Op::INeg, 0, 1, 0),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 0);
    assert!(vm.regs.flags().is_zero());
}

// ── 50. IINC and IDEC ──────────────────────────────────────────

#[test]
fn iinc_idec_basic() {
    // IInc R1, 10; IDec R1, 3; HALT → R1 = 7
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 10),
        Instruction::imm(Op::IDec, 1, 3),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(1), 7);
}

#[test]
fn idec_sets_negative() {
    // IInc R1, 5; IDec R1, 10; HALT → R1 = -5
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 5),
        Instruction::imm(Op::IDec, 1, 10),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(1), -5);
    assert!(vm.regs.flags().is_negative());
}

// ── 51. Multiple calls (nested) ────────────────────────────────

#[test]
fn nested_calls() {
    // Call a function twice to verify return address handling.
    // offset 0:  IInc R1, 1            (4 bytes)
    // offset 4:  Call func             (10 bytes) → PC after = 14, target = 25
    // offset 14: Call func             (10 bytes) → PC after = 24, target = 25
    // offset 24: Halt                  (1 byte)
    // func (offset 25):
    // offset 25: IAdd R1, R1, R1      (4 bytes) — R1 *= 2
    // offset 29: Ret                   (1 byte)
    //
    // R1 starts at 1, after first call R1 = 2, after second call R1 = 4
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 1),            // 0: R1 = 1
        Instruction::var(Op::Call, i64_payload(11)),  // 4: call func (target = 14+11 = 25)
        Instruction::var(Op::Call, i64_payload(1)),   // 14: call func (target = 24+1 = 25)
        Instruction::nullary(Op::Halt),               // 24
        Instruction::reg_ty(Op::IAdd, 1, 1, 1),       // 25: R1 = R1 + R1
        Instruction::nullary(Op::Ret),                // 29
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(1), 4);
}

// ── 52. Encoder produces correct header when finished ─────────

#[test]
fn encoder_finish_includes_header() {
    let mut enc = BytecodeEncoder::new();
    enc.emit(&Instruction::nullary(Op::Halt)).unwrap();
    let bytes = enc.finish(BytecodeHeader::default()).unwrap();
    assert!(bytes.len() >= 18); // header + at least 1 byte of code
    // The instruction bytes start after the 18-byte header.
    assert_eq!(bytes[18], 0x00); // Halt opcode
}

// ── 53. IMul overflow (wrapping semantics) ─────────────────────

#[test]
fn imul_wrapping() {
    // Values that fit in i16 and produce a meaningful result.
    // IInc R1, 1000; IInc R2, 2000; IMul R0, R1, R2 → R0 = 2_000_000
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, 1000),
        Instruction::imm(Op::IInc, 2, 2000),
        Instruction::reg_ty(Op::IMul, 0, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), 2_000_000);
}

// ── 54. ISHR (arithmetic shift right) ───────────────────────────

#[test]
fn ishr_basic() {
    // IInc R1, -16; IInc R2, 2; IShr R0, R1, R2; HALT → R0 = -4
    // Arithmetic right shift preserves sign.
    let bc = encode(&[
        Instruction::imm(Op::IInc, 1, -16),
        Instruction::imm(Op::IInc, 2, 2),
        Instruction::reg_ty(Op::IShr, 0, 1, 2),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert_eq!(vm.regs.read_gp(0), -4);
}

// ── 55. Yield is a no-op ───────────────────────────────────────

#[test]
fn yield_is_noop() {
    let bc = encode(&[
        Instruction::nullary(Op::Yield),
        Instruction::nullary(Op::Halt),
    ]);
    let mut vm = make_vm(&bc);
    vm.execute().unwrap();
    assert!(vm.is_halted());
    assert_eq!(vm.regs.cycles(), 2);
}
