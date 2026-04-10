use std::collections::HashMap;

use crate::instructions::Instruction;
use crate::types::FirType;
use crate::values::Value;

/// Opaque identifier for a basic block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

/// A basic block in SSA form.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub name: String,
    pub params: Vec<Value>,            // block parameters (phi-like)
    pub instructions: Vec<Instruction>,
    pub terminator: Option<Terminator>,
}

impl BasicBlock {
    pub fn new(id: BlockId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            params: Vec::new(),
            instructions: Vec::new(),
            terminator: None,
        }
    }

    /// Appends an instruction. If the instruction is a terminator, it replaces
    /// the block's terminator rather than being added to the instruction list.
    pub fn push_instruction(&mut self, inst: Instruction) {
        if inst.is_terminator() {
            self.terminator = Some(Terminator::from_instruction(&inst));
        } else {
            self.instructions.push(inst);
        }
    }
}

/// The terminator instruction that ends a basic block.
#[derive(Debug, Clone)]
pub enum Terminator {
    Jump(BlockId),
    Branch {
        condition: Value,
        true_block: BlockId,
        false_block: BlockId,
    },
    Return(Option<Value>),
    Unreachable,
}

impl Terminator {
    fn from_instruction(inst: &Instruction) -> Self {
        match inst {
            Instruction::Jump { target } => Terminator::Jump(BlockId(*target)),
            Instruction::Branch { condition, true_block, false_block } => Terminator::Branch {
                condition: condition.clone(),
                true_block: BlockId(*true_block),
                false_block: BlockId(*false_block),
            },
            Instruction::Return { value } => Terminator::Return(value.clone()),
            _ => Terminator::Unreachable,
        }
    }

    /// Returns the block IDs that this terminator branches to.
    pub fn successors(&self) -> Vec<BlockId> {
        match self {
            Terminator::Jump(target) => vec![*target],
            Terminator::Branch { true_block, false_block, .. } => {
                vec![*true_block, *false_block]
            }
            Terminator::Return(_) | Terminator::Unreachable => vec![],
        }
    }
}

/// A function in FIR SSA form.
#[derive(Debug, Clone)]
pub struct FirFunction {
    pub name: String,
    pub params: Vec<Value>,
    pub return_type: FirType,
    pub blocks: Vec<BasicBlock>,
    pub entry_block: BlockId,
}

impl FirFunction {
    pub fn new(name: impl Into<String>, params: Vec<Value>, return_type: FirType) -> Self {
        Self {
            name: name.into(),
            params,
            return_type,
            blocks: Vec::new(),
            entry_block: BlockId(0),
        }
    }

    /// Returns the block with the given ID, if it exists.
    pub fn block_by_id(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.iter().find(|b| b.id == id)
    }

    /// Returns a mutable reference to the block with the given ID.
    pub fn block_by_id_mut(&mut self, id: BlockId) -> Option<&mut BasicBlock> {
        self.blocks.iter_mut().find(|b| b.id == id)
    }
}

/// A FIR module containing functions, globals, and metadata.
#[derive(Debug, Clone)]
pub struct FirModule {
    pub name: String,
    pub functions: HashMap<String, FirFunction>,
    pub globals: HashMap<String, (Value, crate::values::Constant)>,
    pub metadata: HashMap<String, String>,
}

impl FirModule {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            functions: HashMap::new(),
            globals: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_id_display() {
        assert_eq!(format!("{}", BlockId(0)), "bb0");
        assert_eq!(format!("{}", BlockId(42)), "bb42");
    }

    #[test]
    fn test_basic_block_new() {
        let bb = BasicBlock::new(BlockId(0), "entry");
        assert_eq!(bb.id, BlockId(0));
        assert_eq!(bb.name, "entry");
        assert!(bb.instructions.is_empty());
        assert!(bb.terminator.is_none());
    }

    #[test]
    fn test_push_non_terminator() {
        let mut bb = BasicBlock::new(BlockId(0), "entry");
        let inst = Instruction::Nop;
        bb.push_instruction(inst);
        assert_eq!(bb.instructions.len(), 1);
        assert!(bb.terminator.is_none());
    }

    #[test]
    fn test_push_return_as_terminator() {
        let mut bb = BasicBlock::new(BlockId(0), "entry");
        bb.push_instruction(Instruction::Return { value: None });
        assert!(bb.instructions.is_empty());
        assert!(bb.terminator.is_some());
    }

    #[test]
    fn test_terminator_successors_jump() {
        let t = Terminator::Jump(BlockId(1));
        assert_eq!(t.successors(), vec![BlockId(1)]);
    }

    #[test]
    fn test_terminator_successors_branch() {
        let t = Terminator::Branch {
            condition: Value::with_id(0, FirType::Bool),
            true_block: BlockId(1),
            false_block: BlockId(2),
        };
        assert_eq!(t.successors(), vec![BlockId(1), BlockId(2)]);
    }

    #[test]
    fn test_terminator_successors_return() {
        let t = Terminator::Return(None);
        assert!(t.successors().is_empty());
    }

    #[test]
    fn test_fir_function_new() {
        let f = FirFunction::new("foo", vec![], FirType::Void);
        assert_eq!(f.name, "foo");
        assert!(f.params.is_empty());
        assert!(f.blocks.is_empty());
    }

    #[test]
    fn test_fir_module_new() {
        let m = FirModule::new("test");
        assert_eq!(m.name, "test");
        assert!(m.functions.is_empty());
        assert!(m.globals.is_empty());
    }

    #[test]
    fn test_fir_function_block_lookup() {
        let mut f = FirFunction::new("foo", vec![], FirType::Void);
        let bb = BasicBlock::new(BlockId(0), "entry");
        f.blocks.push(bb);
        assert!(f.block_by_id(BlockId(0)).is_some());
        assert!(f.block_by_id(BlockId(1)).is_none());
    }
}
