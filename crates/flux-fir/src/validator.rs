use std::collections::{HashMap, HashSet};

use crate::blocks::{BlockId, FirFunction, FirModule};
// use crate::instructions::Instruction;
// use crate::values::Value;

/// Errors reported by the FIR validator.
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("undefined value used: %{0} (\"{1}\")")]
    UndefinedValue(u32, String),

    #[error("block {0} has no terminator")]
    MissingTerminator(u32),

    #[error("block {0} has multiple terminators")]
    MultipleTerminators(u32),

    #[error("block {0} references non-existent successor {1}")]
    InvalidSuccessor(u32, u32),

    #[error("type mismatch in {0}: expected {1}, got {2}")]
    TypeMismatch(String, String, String),

    #[error("function \"{0}\" has no entry block")]
    MissingEntryBlock(String),

    #[error("unreachable block: bb{0}")]
    UnreachableBlock(u32),

    #[error("function \"{0}\" is empty (no blocks)")]
    EmptyFunction(String),

    #[error("critical edge detected: bb{0} -> bb{1} (bb{1} has multiple predecessors)")]
    CriticalEdge(u32, u32),
}

/// Validates SSA properties of a FIR module.
pub struct FirValidator;

impl FirValidator {
    /// Validates an entire FIR module. Returns a list of all errors found.
    pub fn validate(module: &FirModule) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for func in module.functions.values() {
            errors.extend(Self::validate_function(func));
        }
        errors
    }

    /// Validates a single function's SSA properties.
    pub fn validate_function(func: &FirFunction) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // 1. Check function has blocks
        if func.blocks.is_empty() {
            errors.push(ValidationError::EmptyFunction(func.name.clone()));
            return errors;
        }

        // 2. Check entry block exists
        let entry_exists = func.blocks.iter().any(|bb| bb.id == func.entry_block);
        if !entry_exists {
            errors.push(ValidationError::MissingEntryBlock(func.name.clone()));
        }

        // 3. Collect all defined values (params + instruction results)
        let mut defined_values: HashSet<u32> = HashSet::new();
        for param in &func.params {
            defined_values.insert(param.id);
        }
        for bb in &func.blocks {
            for inst in &bb.instructions {
                if let Some(result) = inst.result_value() {
                    if defined_values.contains(&result.id) {
                        // Value defined twice — not caught here but noted
                    }
                    defined_values.insert(result.id);
                }
            }
        }

        // 4. Check every operand is defined before use, per block in order
        // (This is a simplified check: we verify all referenced values exist
        //  somewhere in the function.)
        let mut all_defined: HashSet<u32> = defined_values.clone();
        for bb in &func.blocks {
            for inst in &bb.instructions {
                for operand in inst.operand_values() {
                    if !all_defined.contains(&operand.id) {
                        errors.push(ValidationError::UndefinedValue(
                            operand.id,
                            operand.name.clone(),
                        ));
                    }
                }
                // After this instruction, its result is defined
                if let Some(result) = inst.result_value() {
                    all_defined.insert(result.id);
                }
            }
        }

        // 5. Check terminators
        let all_block_ids: HashSet<BlockId> = func.blocks.iter().map(|bb| bb.id).collect();
        for bb in &func.blocks {
            match &bb.terminator {
                None => {
                    errors.push(ValidationError::MissingTerminator(bb.id.0));
                }
                Some(term) => {
                    for succ in term.successors() {
                        if !all_block_ids.contains(&succ) {
                            errors.push(ValidationError::InvalidSuccessor(
                                bb.id.0,
                                succ.0,
                            ));
                        }
                    }
                }
            }
            // Check for terminator in instruction list (shouldn't happen with builder)
            let term_count = bb.instructions.iter().filter(|i| i.is_terminator()).count();
            if term_count > 0 {
                errors.push(ValidationError::MultipleTerminators(bb.id.0));
            }
        }

        // 6. Reachability check from entry block
        if entry_exists {
            let mut visited: HashSet<BlockId> = HashSet::new();
            let mut worklist = vec![func.entry_block];
            while let Some(current) = worklist.pop() {
                if visited.insert(current) {
                    if let Some(bb) = func.block_by_id(current) {
                        if let Some(term) = &bb.terminator {
                            for succ in term.successors() {
                                if !visited.contains(&succ) {
                                    worklist.push(succ);
                                }
                            }
                        }
                    }
                }
            }
            for bb in &func.blocks {
                if !visited.contains(&bb.id) {
                    errors.push(ValidationError::UnreachableBlock(bb.id.0));
                }
            }
        }

        // 7. Critical edge detection
        let mut predecessors: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
        for bb in &func.blocks {
            if let Some(term) = &bb.terminator {
                for succ in term.successors() {
                    predecessors.entry(succ).or_default().push(bb.id);
                }
            }
        }
        for bb in &func.blocks {
            if let Some(term) = &bb.terminator {
                let succs = term.successors();
                if succs.len() > 1 {
                    // Branch: check if any successor has >1 predecessor
                    for succ in succs {
                        if let Some(preds) = predecessors.get(&succ) {
                            if preds.len() > 1 {
                                errors.push(ValidationError::CriticalEdge(bb.id.0, succ.0));
                            }
                        }
                    }
                }
            }
        }

        errors
    }

    /// Quick check: returns true if the module is valid.
    pub fn is_valid(module: &FirModule) -> bool {
        Self::validate(module).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::BasicBlock;
    use crate::builder::FirBuilder;
    use crate::types::FirType;
    use crate::values::Value;
    use crate::blocks::Terminator;

    fn build_simple_module() -> FirModule {
        let mut b = FirBuilder::new("test");
        b.create_function("add", vec![("a".into(), FirType::Int(32)), ("b".into(), FirType::Int(32))], FirType::Int(32));
        let a = b.module().functions["add"].params[0].clone();
        let bv = b.module().functions["add"].params[1].clone();
        let sum = b.iadd(a, bv);
        b.ret(Some(sum));
        b.into_module()
    }

    #[test]
    fn test_valid_module() {
        let m = build_simple_module();
        assert!(FirValidator::is_valid(&m));
    }

    #[test]
    fn test_missing_terminator() {
        let mut m = build_simple_module();
        let func = m.functions.get_mut("add").unwrap();
        func.blocks[0].terminator = None;
        let errors = FirValidator::validate_function(func);
        assert!(errors.iter().any(|e| matches!(e, ValidationError::MissingTerminator(_))));
    }

    #[test]
    fn test_empty_function() {
        let func = FirFunction::new("empty", vec![], FirType::Void);
        let errors = FirValidator::validate_function(&func);
        assert!(errors.iter().any(|e| matches!(e, ValidationError::EmptyFunction(_))));
    }

    #[test]
    fn test_invalid_successor() {
        let mut m = build_simple_module();
        let func = m.functions.get_mut("add").unwrap();
        func.blocks[0].terminator = Some(crate::blocks::Terminator::Jump(BlockId(999)));
        let errors = FirValidator::validate_function(func);
        assert!(errors.iter().any(|e| matches!(e, ValidationError::InvalidSuccessor(_, _))));
    }

    #[test]
    fn test_unreachable_block() {
        let mut m = build_simple_module();
        let func = m.functions.get_mut("add").unwrap();
        // Add a block that no one jumps to
        let mut bb = BasicBlock::new(BlockId(42), "orphan");
        bb.terminator = Some(crate::blocks::Terminator::Return(None));
        func.blocks.push(bb);
        let errors = FirValidator::validate_function(func);
        assert!(errors.iter().any(|e| matches!(e, ValidationError::UnreachableBlock(_))));
    }

    #[test]
    fn test_validate_module_multiple_functions() {
        let mut b = FirBuilder::new("test");
        b.create_function("foo", vec![], FirType::Void);
        b.ret(None);
        b.create_function("bar", vec![], FirType::Void);
        b.ret(None);
        let m = b.into_module();
        let errors = FirValidator::validate(&m);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_missing_entry_block() {
        let mut func = FirFunction::new("no_entry", vec![], FirType::Void);
        // Add a block so it's not empty, but set entry to a non-existent block
        let mut bb = BasicBlock::new(BlockId(0), "bb0");
        bb.terminator = Some(Terminator::Return(None));
        func.blocks.push(bb);
        func.entry_block = BlockId(999);
        let errors = FirValidator::validate_function(&func);
        assert!(errors.iter().any(|e| matches!(e, ValidationError::MissingEntryBlock(_))));
    }

    #[test]
    fn test_critical_edge_detection() {
        // Critical edge: an edge from a node with multiple successors
        // to a node with multiple predecessors.
        // CFG: entry -> A (branch) -> merge
        //        entry -> B (jump)  -> merge
        // Edge entry->merge is NOT critical (merge is not a direct successor of entry).
        // But if A branches to merge and exit, and B jumps to merge,
        // then edge A->merge IS critical (A has >1 successor, merge has >1 predecessor).
        let mut func = FirFunction::new("crit", vec![], FirType::Void);

        // entry block: branches to A and B
        let mut entry_bb = BasicBlock::new(BlockId(0), "entry");
        entry_bb.terminator = Some(Terminator::Branch {
            condition: Value::with_id(0, FirType::Bool),
            true_block: BlockId(1),
            false_block: BlockId(2),
        });
        func.entry_block = BlockId(0);
        func.blocks.push(entry_bb);

        // A block: branches to merge and exit
        let mut a_bb = BasicBlock::new(BlockId(1), "A");
        a_bb.terminator = Some(Terminator::Branch {
            condition: Value::with_id(1, FirType::Bool),
            true_block: BlockId(3),
            false_block: BlockId(4),
        });
        func.blocks.push(a_bb);

        // B block: jumps to merge
        let mut b_bb = BasicBlock::new(BlockId(2), "B");
        b_bb.terminator = Some(Terminator::Jump(BlockId(3)));
        func.blocks.push(b_bb);

        // merge block: has multiple predecessors (A and B)
        let mut merge_bb = BasicBlock::new(BlockId(3), "merge");
        merge_bb.terminator = Some(Terminator::Return(None));
        func.blocks.push(merge_bb);

        // exit block
        let mut exit_bb = BasicBlock::new(BlockId(4), "exit");
        exit_bb.terminator = Some(Terminator::Return(None));
        func.blocks.push(exit_bb);

        let errors = FirValidator::validate_function(&func);
        // Edge A(1) -> merge(3) is critical: A has multiple successors, merge has multiple predecessors
        assert!(errors.iter().any(|e| matches!(e, ValidationError::CriticalEdge(1, 3))));
    }
}
