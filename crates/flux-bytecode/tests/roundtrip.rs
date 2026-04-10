//! Integration tests for the FLUX bytecode encoder/decoder/validator roundtrip.

use flux_bytecode::*;

// ── Helper ─────────────────────────────────────────────────────────────────

/// Encodes a list of instructions into a full bytecode module and decodes it back.
fn roundtrip(instructions: &[Instruction]) -> DecodedModule {
    let header = BytecodeHeader::new(1, 0, 0);
    let bytes = encode_with_header(&header, instructions);
    let mut dec = BytecodeDecoder::new(&bytes);
    dec.decode_all().unwrap()
}

/// Encodes instructions with a given header, returning the full bytecode bytes.
fn encode_with_header(header: &BytecodeHeader, instructions: &[Instruction]) -> Vec<u8> {
    let mut enc = BytecodeEncoder::new();
    for instr in instructions {
        enc.emit(instr).unwrap();
    }
    enc.finish(*header).unwrap()
}

// ══════════════════════════════════════════════════════════════════════════
// Header roundtrip tests
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn test_header_roundtrip_default() {
    let h = BytecodeHeader::default();
    let bytes = h.to_bytes().unwrap();
    let decoded = BytecodeHeader::from_bytes(&bytes).unwrap();
    assert_eq!(h, decoded);
}

#[test]
fn test_header_roundtrip_with_values() {
    let h = BytecodeHeader::with_flags(42, 100, 7, 0xABCD);
    let bytes = h.to_bytes().unwrap();
    let decoded = BytecodeHeader::from_bytes(&bytes).unwrap();
    assert_eq!(h, decoded);
}

#[test]
fn test_header_magic_bytes() {
    let h = BytecodeHeader::default();
    let bytes = h.to_bytes().unwrap();
    assert_eq!(&bytes[0..4], b"FLUX");
}

#[test]
fn test_header_version_le() {
    let h = BytecodeHeader::default();
    let bytes = h.to_bytes().unwrap();
    // version = 1 → bytes [4] = 0x01, bytes [5] = 0x00
    assert_eq!(bytes[4], 1);
    assert_eq!(bytes[5], 0);
}

#[test]
fn test_header_size() {
    let h = BytecodeHeader::default();
    let bytes = h.to_bytes().unwrap();
    assert_eq!(bytes.len(), HEADER_SIZE);
}

// ══════════════════════════════════════════════════════════════════════════
// Format A — nullary instructions
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn test_format_a_halt() {
    let instr = Instruction::nullary(Op::Halt);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::nullary(Op::Halt));
}

#[test]
fn test_format_a_nop() {
    let instr = Instruction::nullary(Op::Nop);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::nullary(Op::Nop));
}

#[test]
fn test_format_a_ret() {
    let instr = Instruction::nullary(Op::Ret);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::nullary(Op::Ret));
}

#[test]
fn test_format_a_panic() {
    let instr = Instruction::nullary(Op::Panic);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::nullary(Op::Panic));
}

#[test]
fn test_format_a_unreachable() {
    let instr = Instruction::nullary(Op::Unreachable);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::nullary(Op::Unreachable));
}

#[test]
fn test_format_a_yield() {
    let instr = Instruction::nullary(Op::Yield);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::nullary(Op::Yield));
}

// ══════════════════════════════════════════════════════════════════════════
// Format B — two-register instructions
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn test_format_b_push() {
    let instr = Instruction::reg(Op::Push, 3, 7);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::reg(Op::Push, 3, 7));
}

#[test]
fn test_format_b_pop() {
    let instr = Instruction::reg(Op::Pop, 1, 0);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::reg(Op::Pop, 1, 0));
}

#[test]
fn test_format_b_dup() {
    let instr = Instruction::reg(Op::Dup, 5, 5);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::reg(Op::Dup, 5, 5));
}

#[test]
fn test_format_b_swap() {
    let instr = Instruction::reg(Op::Swap, 0, 1);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::reg(Op::Swap, 0, 1));
}

#[test]
fn test_format_b_imov() {
    let instr = Instruction::reg(Op::IMov, 10, 20);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::reg(Op::IMov, 10, 20));
}

#[test]
fn test_format_b_fmov() {
    let instr = Instruction::reg(Op::FMov, 15, 31);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::reg(Op::FMov, 15, 31));
}

// ══════════════════════════════════════════════════════════════════════════
// Format C — two registers + type tag
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn test_format_c_iadd() {
    let instr = Instruction::reg_ty(Op::IAdd, 0, 1, 0x02);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::reg_ty(Op::IAdd, 0, 1, 0x02));
}

#[test]
fn test_format_c_fadd() {
    let instr = Instruction::reg_ty(Op::FAdd, 2, 3, 0x01);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::reg_ty(Op::FAdd, 2, 3, 0x01));
}

#[test]
fn test_format_c_isub() {
    let instr = Instruction::reg_ty(Op::ISub, 10, 11, 0x05);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::reg_ty(Op::ISub, 10, 11, 0x05));
}

#[test]
fn test_format_cast() {
    let instr = Instruction::reg_ty(Op::Cast, 4, 5, 0xFF);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::reg_ty(Op::Cast, 4, 5, 0xFF));
}

#[test]
fn test_format_itof() {
    let instr = Instruction::reg_ty(Op::IToF, 0, 1, 0x00);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::reg_ty(Op::IToF, 0, 1, 0x00));
}

#[test]
fn test_format_bnot() {
    let instr = Instruction::reg_ty(Op::BNot, 7, 7, 0x03);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::reg_ty(Op::BNot, 7, 7, 0x03));
}

// ══════════════════════════════════════════════════════════════════════════
// Format D — register + 16-bit immediate
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn test_format_d_iinc_positive() {
    let instr = Instruction::imm(Op::IInc, 5, 42);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::imm(Op::IInc, 5, 42));
}

#[test]
fn test_format_d_iinc_negative() {
    let instr = Instruction::imm(Op::IInc, 3, -1);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::imm(Op::IInc, 3, -1));
}

#[test]
fn test_format_d_idec() {
    let instr = Instruction::imm(Op::IDec, 1, -10);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::imm(Op::IDec, 1, -10));
}

#[test]
fn test_format_d_iinc_max() {
    let instr = Instruction::imm(Op::IInc, 0, i16::MAX);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::imm(Op::IInc, 0, i16::MAX));
}

#[test]
fn test_format_d_iinc_min() {
    let instr = Instruction::imm(Op::IInc, 0, i16::MIN);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::imm(Op::IInc, 0, i16::MIN));
}

#[test]
fn test_format_d_stack_alloc() {
    let instr = Instruction::imm(Op::StackAlloc, 0, 1024);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::imm(Op::StackAlloc, 0, 1024));
}

// ══════════════════════════════════════════════════════════════════════════
// Format E — two registers + 16-bit offset
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn test_format_e_load32() {
    let instr = Instruction::mem(Op::Load32, 0, 1, 256);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::mem(Op::Load32, 0, 1, 256));
}

#[test]
fn test_format_e_store8() {
    let instr = Instruction::mem(Op::Store8, 5, 6, 0);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::mem(Op::Store8, 5, 6, 0));
}

#[test]
fn test_format_e_load64() {
    let instr = Instruction::mem(Op::Load64, 10, 20, u16::MAX);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::mem(Op::Load64, 10, 20, u16::MAX));
}

#[test]
fn test_format_e_store64() {
    let instr = Instruction::mem(Op::Store64, 3, 7, 4096);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::mem(Op::Store64, 3, 7, 4096));
}

#[test]
fn test_format_e_vload() {
    let instr = Instruction::mem(Op::VLoad, 0, 1, 8);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::mem(Op::VLoad, 0, 1, 8));
}

#[test]
fn test_format_e_load_addr() {
    let instr = Instruction::mem(Op::LoadAddr, 2, 3, 0);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::mem(Op::LoadAddr, 2, 3, 0));
}

// ══════════════════════════════════════════════════════════════════════════
// Format G — variable-length payload
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn test_format_g_call() {
    let instr = Instruction::var(Op::Call, vec![0x05, 0x00]);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::var(Op::Call, vec![0x05, 0x00]));
}

#[test]
fn test_format_g_jump() {
    let instr = Instruction::var(Op::Jump, vec![0x10, 0x00]);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::var(Op::Jump, vec![0x10, 0x00]));
}

#[test]
fn test_format_g_jump_if() {
    let instr = Instruction::var(Op::JumpIf, vec![0x0A, 0x00]);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::var(Op::JumpIf, vec![0x0A, 0x00]));
}

#[test]
fn test_format_g_asend() {
    let instr = Instruction::var(Op::ASend, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    let module = roundtrip(&[instr]);
    assert_eq!(
        module.instructions[0],
        Instruction::var(Op::ASend, vec![0xDE, 0xAD, 0xBE, 0xEF])
    );
}

#[test]
fn test_format_g_empty_payload() {
    let instr = Instruction::var(Op::Jump, vec![]);
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::var(Op::Jump, vec![]));
}

#[test]
fn test_format_g_max_payload() {
    let payload = (0u8..=254).collect::<Vec<_>>();
    let instr = Instruction::var(Op::ABroadcast, payload.clone());
    let module = roundtrip(&[instr]);
    assert_eq!(module.instructions[0], Instruction::var(Op::ABroadcast, payload));
}

// ══════════════════════════════════════════════════════════════════════════
// Full program roundtrip
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn test_full_program_roundtrip() {
    let instructions = vec![
        Instruction::reg(Op::Push, 1, 0),
        Instruction::reg(Op::Push, 2, 0),
        Instruction::reg_ty(Op::IAdd, 3, 1, 0x02),
        Instruction::imm(Op::IInc, 3, 10),
        Instruction::mem(Op::Store32, 3, 0, 100),
        Instruction::nullary(Op::Halt),
    ];

    let header = BytecodeHeader::new(1, 0, 0);
    let bytes = encode_with_header(&header, &instructions);
    let mut dec = BytecodeDecoder::new(&bytes);
    let module = dec.decode_all().unwrap();

    assert_eq!(module.header, header);
    assert_eq!(module.instructions.len(), instructions.len());
    for (a, b) in instructions.iter().zip(module.instructions.iter()) {
        assert_eq!(a, b);
    }
}

#[test]
fn test_empty_program() {
    let header = BytecodeHeader::new(0, 0, 0);
    let bytes = encode_with_header(&header, &[]);
    let mut dec = BytecodeDecoder::new(&bytes);
    let module = dec.decode_all().unwrap();
    assert_eq!(module.instructions.len(), 0);
}

#[test]
fn test_multi_function_program() {
    let instructions = vec![
        // "Function 0" — just halt
        Instruction::nullary(Op::Halt),
    ];

    let header = BytecodeHeader::new(3, 2, 0);
    let bytes = encode_with_header(&header, &instructions);
    let mut dec = BytecodeDecoder::new(&bytes);
    let module = dec.decode_all().unwrap();

    assert_eq!(module.header.num_functions, 3);
    assert_eq!(module.header.num_strings, 2);
    assert_eq!(module.instructions.len(), 1);
}

// ══════════════════════════════════════════════════════════════════════════
// Byte-identical output tests
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn test_byte_identical_halt() {
    let mut enc = BytecodeEncoder::new();
    enc.emit(&Instruction::nullary(Op::Halt)).unwrap();
    let bytes = enc.into_bytes();
    assert_eq!(bytes, vec![0x00]);
}

#[test]
fn test_byte_identical_iadd() {
    let mut enc = BytecodeEncoder::new();
    enc.emit(&Instruction::reg_ty(Op::IAdd, 0, 1, 2)).unwrap();
    let bytes = enc.into_bytes();
    assert_eq!(bytes, vec![0x21, 0x00, 0x01, 0x02]);
}

#[test]
fn test_byte_identical_iinc() {
    let mut enc = BytecodeEncoder::new();
    enc.emit(&Instruction::imm(Op::IInc, 5, -1)).unwrap();
    let bytes = enc.into_bytes();
    assert_eq!(bytes, vec![0x28, 0x05, 0xFF, 0xFF]);
}

#[test]
fn test_byte_identical_load32() {
    let mut enc = BytecodeEncoder::new();
    enc.emit(&Instruction::mem(Op::Load32, 3, 7, 256)).unwrap();
    let bytes = enc.into_bytes();
    assert_eq!(bytes, vec![0x72, 0x03, 0x07, 0x00, 0x01]);
}

#[test]
fn test_byte_identical_call() {
    let mut enc = BytecodeEncoder::new();
    enc.emit(&Instruction::var(Op::Call, vec![5, 0])).unwrap();
    let bytes = enc.into_bytes();
    assert_eq!(bytes, vec![0x06, 0x02, 0x05, 0x00]);
}

// ══════════════════════════════════════════════════════════════════════════
// Validator tests
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn test_validator_passes_valid_module() {
    let instructions = vec![
        Instruction::reg(Op::Push, 1, 0),
        Instruction::nullary(Op::Halt),
    ];
    let module = roundtrip(&instructions);
    let errors = validate(&module);
    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}

#[test]
fn test_validator_catches_missing_terminator() {
    let instructions = vec![
        Instruction::reg(Op::Push, 1, 0),
        Instruction::reg(Op::Pop, 2, 0),
    ];
    let module = roundtrip(&instructions);
    let errors = validate(&module);
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::MissingTerminator { .. }
    )));
}

#[test]
fn test_validator_catches_code_after_halt() {
    let instructions = vec![
        Instruction::nullary(Op::Halt),
        Instruction::reg(Op::Push, 1, 0),
    ];
    let module = roundtrip(&instructions);
    let errors = validate(&module);
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::InstructionsAfterTerminator { .. }
    )));
}

#[test]
fn test_validator_empty_module_ok() {
    let header = BytecodeHeader::new(0, 0, 0);
    let bytes = encode_with_header(&header, &[]);
    let mut dec = BytecodeDecoder::new(&bytes);
    let module = dec.decode_all().unwrap();
    let errors = validate(&module);
    assert!(errors.is_empty());
}

#[test]
fn test_validator_catches_invalid_magic() {
    let mut module = DecodedModule {
        header: BytecodeHeader::default(),
        instructions: vec![Instruction::nullary(Op::Halt)],
    };
    module.header.magic = [0x00, 0x00, 0x00, 0x00];
    let errors = validate(&module);
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::InvalidMagic { .. }
    )));
}

// ══════════════════════════════════════════════════════════════════════════
// Opcode coverage — every opcode has a valid format
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn test_all_opcodes_have_valid_format() {
    // Iterate through every possible byte and check that all valid opcodes
    // have a defined format.
    let mut count = 0usize;
    for byte in 0u8..=255 {
        if let Ok(op) = Op::try_from(byte) {
            let _fmt = op.format();
            count += 1;
        }
    }
    // We should have at least 100 opcodes.
    assert!(
        count >= 100,
        "expected at least 100 opcodes, got {}",
        count
    );
}

#[test]
fn test_all_opcodes_roundtrip() {
    // For each valid opcode, construct a minimal instruction and roundtrip it.
    for byte in 0u8..=255 {
        if let Ok(op) = Op::try_from(byte) {
            let instr = match op.format() {
                InstrFormat::A => Instruction::nullary(op),
                InstrFormat::B => Instruction::reg(op, 0, 1),
                InstrFormat::C => Instruction::reg_ty(op, 0, 1, 0),
                InstrFormat::D => Instruction::imm(op, 0, 0),
                InstrFormat::E => Instruction::mem(op, 0, 1, 0),
                InstrFormat::G => Instruction::var(op, vec![]),
            };

            let mut enc = BytecodeEncoder::new();
            enc.emit(&instr).unwrap();
            let bytes = enc.into_bytes();

            let mut dec = BytecodeDecoder::new(&bytes);
            let decoded = dec.decode_instruction().unwrap().unwrap();
            assert_eq!(decoded, instr, "roundtrip failed for opcode {:?} (0x{:02X})", op, byte);
        }
    }
}

#[test]
fn test_opcode_count() {
    assert_eq!(Op::count(), 100);
}

#[test]
fn test_all_opcodes_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for byte in 0u8..=255 {
        if let Ok(op) = Op::try_from(byte) {
            assert!(
                seen.insert(byte),
                "duplicate opcode byte 0x{:02X} for {:?}",
                byte,
                op
            );
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════
// Opcode property tests
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn test_terminators_set() {
    let terminators = [Op::Halt, Op::Ret, Op::Panic, Op::Unreachable];
    for op in &terminators {
        assert!(op.is_terminator());
    }
    // A few non-terminators for contrast
    assert!(!Op::Nop.is_terminator());
    assert!(!Op::Jump.is_terminator());
    assert!(!Op::Call.is_terminator());
    assert!(!Op::Yield.is_terminator());
}

#[test]
fn test_branches_set() {
    let branches = [Op::Jump, Op::JumpIf, Op::JumpIfNot, Op::Call, Op::CallIndirect, Op::Yield];
    for op in &branches {
        assert!(op.is_branch());
    }
    assert!(!Op::Halt.is_branch());
}

#[test]
fn test_a2a_set() {
    let a2a = [
        Op::ASend, Op::ARecv, Op::AAsk, Op::ATell, Op::ADelegate,
        Op::ABroadcast, Op::ASubscribe, Op::AWait, Op::ATrust, Op::AVerify,
    ];
    for op in &a2a {
        assert!(op.is_a2a());
    }
    assert!(!Op::Halt.is_a2a());
    assert!(!Op::IAdd.is_a2a());
}

#[test]
fn test_mnemonic_and_category_not_empty() {
    for byte in 0u8..=255 {
        if let Ok(op) = Op::try_from(byte) {
            assert!(!op.mnemonic().is_empty());
            assert!(!op.category().is_empty());
        }
    }
}

#[test]
fn test_display_impl() {
    assert_eq!(format!("{}", Op::Halt), "halt");
    assert_eq!(format!("{}", Op::IAdd), "iadd");
    assert_eq!(format!("{}", Op::FAdd), "fadd");
    assert_eq!(format!("{}", Op::ASend), "a_send");
}

// ══════════════════════════════════════════════════════════════════════════
// Encoder error tests
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn test_encoder_rejects_bad_register() {
    let mut enc = BytecodeEncoder::new();
    assert!(enc.emit(&Instruction::reg(Op::Push, 64, 0)).is_err());
    assert!(enc.emit(&Instruction::reg(Op::Push, 0, 100)).is_err());
}

#[test]
fn test_encoder_rejects_oversize_payload() {
    let mut enc = BytecodeEncoder::new();
    assert!(enc.emit(&Instruction::var(Op::Call, vec![0u8; 256])).is_err());
}

#[test]
fn test_encoder_rejects_bad_register_format_c() {
    let mut enc = BytecodeEncoder::new();
    assert!(enc.emit(&Instruction::reg_ty(Op::IAdd, 64, 0, 0)).is_err());
}

#[test]
fn test_encoder_rejects_bad_register_format_e() {
    let mut enc = BytecodeEncoder::new();
    assert!(enc.emit(&Instruction::mem(Op::Load32, 64, 0, 0)).is_err());
}

// ══════════════════════════════════════════════════════════════════════════
// Decoder error tests
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn test_decoder_invalid_magic() {
    let bad = vec![0u8; 18];
    let result = BytecodeHeader::from_bytes(&bad);
    assert!(matches!(result, Err(DecodeError::InvalidMagic { .. })));
}

#[test]
fn test_decoder_too_short() {
    let short = vec![0u8; 5];
    let result = BytecodeHeader::from_bytes(&short);
    assert!(matches!(result, Err(DecodeError::UnexpectedEof { .. })));
}

#[test]
fn test_decoder_truncated_instruction() {
    // Format B (Push) needs 3 bytes total; we provide only 1 (the opcode).
    let bytes: &[u8] = &[0x10]; // PUSH opcode only
    let mut dec = BytecodeDecoder::new(bytes);
    let result = dec.decode_instruction();
    assert!(matches!(result, Err(DecodeError::UnexpectedEof { .. })));
}

#[test]
fn test_decoder_truncated_format_g_payload() {
    // Format G says payload length = 5, but only 2 bytes follow.
    let bytes: &[u8] = &[0x06, 0x05, 0x01, 0x02]; // CALL with len=5 but only 2 payload bytes
    let mut dec = BytecodeDecoder::new(bytes);
    let result = dec.decode_instruction();
    assert!(matches!(result, Err(DecodeError::InvalidPayloadLength { .. })));
}

#[test]
fn test_decoder_invalid_opcode_byte() {
    let bytes: &[u8] = &[0xFF];
    let mut dec = BytecodeDecoder::new(bytes);
    let result = dec.decode_instruction();
    assert!(matches!(result, Err(DecodeError::InvalidOpcode { .. })));
}

// ══════════════════════════════════════════════════════════════════════════
// Format unit tests
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn test_format_repr_u8() {
    assert_eq!(InstrFormat::A as u8, 0);
    assert_eq!(InstrFormat::B as u8, 1);
    assert_eq!(InstrFormat::C as u8, 2);
    assert_eq!(InstrFormat::D as u8, 3);
    assert_eq!(InstrFormat::E as u8, 4);
    assert_eq!(InstrFormat::G as u8, 5);
}

#[test]
fn test_format_try_from_u8() {
    assert_eq!(InstrFormat::try_from(0).unwrap(), InstrFormat::A);
    assert_eq!(InstrFormat::try_from(5).unwrap(), InstrFormat::G);
    assert!(InstrFormat::try_from(6).is_err());
    assert!(InstrFormat::try_from(255).is_err());
}

#[test]
fn test_instruction_constructors() {
    let a = Instruction::nullary(Op::Halt);
    assert_eq!(a.op, Op::Halt);
    assert_eq!(a.dst, 0);

    let b = Instruction::reg(Op::Push, 3, 5);
    assert_eq!(b.dst, 3);
    assert_eq!(b.src, 5);

    let c = Instruction::reg_ty(Op::IAdd, 1, 2, 0xFF);
    assert_eq!(c.type_tag, 0xFF);

    let d = Instruction::imm(Op::IInc, 7, -100);
    assert_eq!(d.immediate, -100);

    let e = Instruction::mem(Op::Load32, 0, 1, 512);
    assert_eq!(e.offset, 512);

    let g = Instruction::var(Op::Call, vec![0x01, 0x00]);
    assert_eq!(g.payload, vec![0x01, 0x00]);
}
