//! FLUX bytecode validator — checks structural invariants of a decoded module.
//!
//! Validation is performed on a fully decoded [`DecodedModule`] and checks:
//!
//! - Header magic is `b"FLUX"`
//! - All opcodes are valid
//! - Register IDs are < 64
//! - Instructions do not appear after a terminator within a function body
//! - Each function body contains at least one terminator instruction

use crate::decoder::DecodedModule;
use crate::encoder::MAX_REGISTER;
use crate::error::ValidationError;
use crate::opcodes::Op;

/// Validates a decoded bytecode module, returning a list of all found errors.
///
/// If the returned list is empty, the module is valid.
///
/// # Arguments
///
/// * `module` - The decoded module to validate.
///
/// # Returns
///
/// A vector of [`ValidationError`]s. An empty vector means the module is valid.
pub fn validate(module: &DecodedModule) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // ── Check header magic ───────────────────────────────────────────
    if module.header.magic != *b"FLUX" {
        errors.push(ValidationError::InvalidMagic {
            found: module.header.magic,
        });
    }

    // ── Check each instruction ───────────────────────────────────────
    let mut found_terminator = false;
    let mut terminator_offset: usize = 0;

    for (idx, instr) in module.instructions.iter().enumerate() {
        let offset = compute_offset(&module.instructions, idx);

        // Check opcodes are valid (should always be true if decoded
        // through BytecodeDecoder, but validate defensively).
        if Op::try_from(instr.op.byte()).is_err() {
            errors.push(ValidationError::InvalidOpcode {
                offset,
                byte: instr.op.byte(),
            });
        }

        // Check register IDs for formats that use them.
        let fmt = instr.op.format();
        match fmt {
            crate::format::InstrFormat::B
            | crate::format::InstrFormat::C
            | crate::format::InstrFormat::E => {
                if instr.dst >= MAX_REGISTER {
                    errors.push(ValidationError::InvalidRegister {
                        offset,
                        reg: instr.dst,
                    });
                }
                if instr.src >= MAX_REGISTER {
                    errors.push(ValidationError::InvalidRegister {
                        offset,
                        reg: instr.src,
                    });
                }
            }
            crate::format::InstrFormat::D => {
                if instr.dst >= MAX_REGISTER {
                    errors.push(ValidationError::InvalidRegister {
                        offset,
                        reg: instr.dst,
                    });
                }
            }
            crate::format::InstrFormat::A | crate::format::InstrFormat::G => {}
        }

        // Check for instructions after a terminator.
        if found_terminator {
            errors.push(ValidationError::InstructionsAfterTerminator {
                terminator_offset,
                after_offset: offset,
            });
            // Don't set found_terminator to false; continue reporting.
        }

        if instr.op.is_terminator() {
            if !found_terminator {
                found_terminator = true;
                terminator_offset = offset;
                // after_offset updated for next iteration
            }
        }
    }

    // ── Check at least one terminator exists ─────────────────────────
    if !found_terminator && !module.instructions.is_empty() {
        // We treat the entire instruction list as one "function" for this check.
        errors.push(ValidationError::MissingTerminator {
            function_index: 0,
        });
    }

    errors
}

/// Computes the byte offset of the instruction at the given index within the
/// instruction list by summing the encoded sizes of all preceding instructions.
fn compute_offset(instructions: &[crate::encoder::Instruction], index: usize) -> usize {
    let mut offset = 0usize;
    for i in 0..index {
        let fmt = instructions[i].op.format();
        let payload_len = if fmt == crate::format::InstrFormat::G {
            instructions[i].payload.len()
        } else {
            0
        };
        offset += fmt.total_size(payload_len);
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::Instruction;
    use crate::header::BytecodeHeader;
    use crate::opcodes::Op;

    fn make_module(instructions: Vec<Instruction>) -> DecodedModule {
        DecodedModule {
            header: BytecodeHeader::new(1, 0, 0),
            instructions,
        }
    }

    #[test]
    fn valid_single_halt() {
        let module = make_module(vec![Instruction::nullary(Op::Halt)]);
        let errors = validate(&module);
        assert!(errors.is_empty());
    }

    #[test]
    fn valid_sequence() {
        let module = make_module(vec![
            Instruction::reg(Op::Push, 1, 0),
            Instruction::nullary(Op::Ret),
        ]);
        let errors = validate(&module);
        assert!(errors.is_empty());
    }

    #[test]
    fn missing_terminator() {
        let module = make_module(vec![
            Instruction::reg(Op::Push, 1, 0),
        ]);
        let errors = validate(&module);
        assert!(errors.iter().any(|e| matches!(
            e,
            ValidationError::MissingTerminator { .. }
        )));
    }

    #[test]
    fn instructions_after_terminator() {
        let module = make_module(vec![
            Instruction::nullary(Op::Halt),
            Instruction::reg(Op::Push, 1, 0),
        ]);
        let errors = validate(&module);
        assert!(errors.iter().any(|e| matches!(
            e,
            ValidationError::InstructionsAfterTerminator { .. }
        )));
    }

    #[test]
    fn empty_module_valid() {
        let module = make_module(vec![]);
        let errors = validate(&module);
        assert!(errors.is_empty());
    }

    #[test]
    fn invalid_magic() {
        let mut module = make_module(vec![Instruction::nullary(Op::Halt)]);
        module.header.magic = [0, 0, 0, 0];
        let errors = validate(&module);
        assert!(errors.iter().any(|e| matches!(
            e,
            ValidationError::InvalidMagic { .. }
        )));
    }
}
