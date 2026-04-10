use crate::types::FirType;
use crate::values::{Constant, Value};

/// Comparison operations for integer/floating-point comparison instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl std::fmt::Display for CmpOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CmpOp::Eq => write!(f, "eq"),
            CmpOp::Ne => write!(f, "ne"),
            CmpOp::Lt => write!(f, "lt"),
            CmpOp::Le => write!(f, "le"),
            CmpOp::Gt => write!(f, "gt"),
            CmpOp::Ge => write!(f, "ge"),
        }
    }
}

/// All FIR instructions in SSA form.
#[derive(Debug, Clone)]
pub enum Instruction {
    // ── Arithmetic ──────────────────────────────────────────
    IAdd { result: Value, lhs: Value, rhs: Value },
    ISub { result: Value, lhs: Value, rhs: Value },
    IMul { result: Value, lhs: Value, rhs: Value },
    IDiv { result: Value, lhs: Value, rhs: Value },
    IMod { result: Value, lhs: Value, rhs: Value },
    INeg { result: Value, operand: Value },
    IAbs { result: Value, operand: Value },
    FAdd { result: Value, lhs: Value, rhs: Value },
    FSub { result: Value, lhs: Value, rhs: Value },
    FMul { result: Value, lhs: Value, rhs: Value },
    FDiv { result: Value, lhs: Value, rhs: Value },
    FNeg { result: Value, operand: Value },
    FAbs { result: Value, operand: Value },

    // ── Comparison ──────────────────────────────────────────
    ICmpEq { result: Value, lhs: Value, rhs: Value },
    ICmpNe { result: Value, lhs: Value, rhs: Value },
    ICmpLt { result: Value, lhs: Value, rhs: Value },
    ICmpLe { result: Value, lhs: Value, rhs: Value },
    ICmpGt { result: Value, lhs: Value, rhs: Value },
    ICmpGe { result: Value, lhs: Value, rhs: Value },

    // ── Conversion ──────────────────────────────────────────
    IToF { result: Value, operand: Value },
    FToI { result: Value, operand: Value },

    // ── Memory ──────────────────────────────────────────────
    Load { result: Value, ptr: Value, ty: FirType },
    Store { ptr: Value, value: Value, ty: FirType },
    Alloc { result: Value, ty: FirType },
    StackAlloc { result: Value, ty: FirType },
    GEP { result: Value, ptr: Value, indices: Vec<Value> },

    // ── Control flow ────────────────────────────────────────
    Jump { target: u32 },
    Branch { condition: Value, true_block: u32, false_block: u32 },
    Call { result: Option<Value>, function: String, args: Vec<Value> },
    Return { value: Option<Value> },

    // ── Constants ───────────────────────────────────────────
    Const { result: Value, constant: Constant },

    // ── Agent operations ────────────────────────────────────
    ASend { target: Value, message: Value },
    ARecv { result: Value },
    AAsk { result: Value, target: Value, question: Value },
    ATell { target: Value, message: Value },
    ADelegate { result: Value, target: Value, task: Value },
    ABroadcast { message: Value },

    // ── Meta ────────────────────────────────────────────────
    Cast { result: Value, value: Value, target_ty: FirType },
    SizeOf { result: Value, ty: FirType },
    Nop,
}

impl Instruction {
    /// Returns the result value of this instruction, if any.
    pub fn result_value(&self) -> Option<&Value> {
        match self {
            Instruction::IAdd { result, .. }
            | Instruction::ISub { result, .. }
            | Instruction::IMul { result, .. }
            | Instruction::IDiv { result, .. }
            | Instruction::IMod { result, .. }
            | Instruction::INeg { result, .. }
            | Instruction::IAbs { result, .. }
            | Instruction::FAdd { result, .. }
            | Instruction::FSub { result, .. }
            | Instruction::FMul { result, .. }
            | Instruction::FDiv { result, .. }
            | Instruction::FNeg { result, .. }
            | Instruction::FAbs { result, .. }
            | Instruction::ICmpEq { result, .. }
            | Instruction::ICmpNe { result, .. }
            | Instruction::ICmpLt { result, .. }
            | Instruction::ICmpLe { result, .. }
            | Instruction::ICmpGt { result, .. }
            | Instruction::ICmpGe { result, .. }
            | Instruction::IToF { result, .. }
            | Instruction::FToI { result, .. }
            | Instruction::Load { result, .. }
            | Instruction::Alloc { result, .. }
            | Instruction::StackAlloc { result, .. }
            | Instruction::GEP { result, .. }
            | Instruction::ARecv { result }
            | Instruction::AAsk { result, .. }
            | Instruction::ADelegate { result, .. }
            | Instruction::Const { result, .. }
            | Instruction::Cast { result, .. }
            | Instruction::SizeOf { result, .. } => Some(result),
            Instruction::Call { result, .. } => result.as_ref(),
            _ => None,
        }
    }

    /// Returns true if this instruction is a terminator (ends a basic block).
    pub fn is_terminator(&self) -> bool {
        matches!(
            self,
            Instruction::Jump { .. }
                | Instruction::Branch { .. }
                | Instruction::Return { .. }
        )
    }

    /// Returns all operand `Value`s referenced by this instruction.
    pub fn operand_values(&self) -> Vec<&Value> {
        match self {
            Instruction::IAdd { lhs, rhs, .. }
            | Instruction::ISub { lhs, rhs, .. }
            | Instruction::IMul { lhs, rhs, .. }
            | Instruction::IDiv { lhs, rhs, .. }
            | Instruction::IMod { lhs, rhs, .. }
            | Instruction::FAdd { lhs, rhs, .. }
            | Instruction::FSub { lhs, rhs, .. }
            | Instruction::FMul { lhs, rhs, .. }
            | Instruction::FDiv { lhs, rhs, .. }
            | Instruction::ICmpEq { lhs, rhs, .. }
            | Instruction::ICmpNe { lhs, rhs, .. }
            | Instruction::ICmpLt { lhs, rhs, .. }
            | Instruction::ICmpLe { lhs, rhs, .. }
            | Instruction::ICmpGt { lhs, rhs, .. }
            | Instruction::ICmpGe { lhs, rhs, .. } => vec![lhs, rhs],

            Instruction::INeg { operand, .. }
            | Instruction::IAbs { operand, .. }
            | Instruction::FNeg { operand, .. }
            | Instruction::FAbs { operand, .. }
            | Instruction::IToF { operand, .. }
            | Instruction::FToI { operand, .. } => vec![operand],

            Instruction::Load { ptr, .. } => vec![ptr],
            Instruction::Cast { value, .. } => vec![value],

            Instruction::Store { ptr, value, .. } => vec![ptr, value],
            Instruction::Branch { condition, .. } => vec![condition],
            Instruction::Call { args, .. } => args.iter().collect(),
            Instruction::Return { value: Some(v) } => vec![v],
            Instruction::GEP { ptr, indices, .. } => {
                let mut ops: Vec<&Value> = vec![ptr];
                ops.extend(indices);
                ops
            }
            Instruction::ASend { target, message } => vec![target, message],
            Instruction::ATell { target, message } => vec![target, message],
            Instruction::AAsk { target, question, .. } => vec![target, question],
            Instruction::ADelegate { target, task, .. } => vec![target, task],
            Instruction::ABroadcast { message } => vec![message],

            Instruction::Const { .. }
            | Instruction::Alloc { .. }
            | Instruction::StackAlloc { .. }
            | Instruction::SizeOf { .. }
            | Instruction::ARecv { .. }
            | Instruction::Nop
            | Instruction::Jump { .. }
            | Instruction::Return { value: None } => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_val(id: u32, name: &str) -> Value {
        Value::new(id, name, FirType::Int(32))
    }

    #[test]
    fn test_iadd_result() {
        let result = make_val(0, "r");
        let lhs = make_val(1, "a");
        let rhs = make_val(2, "b");
        let inst = Instruction::IAdd { result, lhs, rhs };
        let r = inst.result_value().unwrap();
        assert_eq!(r.id, 0);
    }

    #[test]
    fn test_const_result() {
        let result = make_val(0, "c");
        let inst = Instruction::Const {
            result,
            constant: Constant::Int(42),
        };
        let r = inst.result_value().unwrap();
        assert_eq!(r.id, 0);
    }

    #[test]
    fn test_store_no_result() {
        let ptr = make_val(0, "p");
        let val = make_val(1, "v");
        let inst = Instruction::Store {
            ptr,
            value: val,
            ty: FirType::Int(32),
        };
        assert!(inst.result_value().is_none());
    }

    #[test]
    fn test_is_terminator() {
        let inst = Instruction::Return { value: None };
        assert!(inst.is_terminator());
        let inst2 = Instruction::IAdd {
            result: make_val(0, "r"),
            lhs: make_val(1, "a"),
            rhs: make_val(2, "b"),
        };
        assert!(!inst2.is_terminator());
    }

    #[test]
    fn test_branch_is_terminator() {
        let cond = make_val(0, "c");
        let inst = Instruction::Branch {
            condition: cond,
            true_block: 0,
            false_block: 1,
        };
        assert!(inst.is_terminator());
    }

    #[test]
    fn test_jump_is_terminator() {
        let inst = Instruction::Jump { target: 0 };
        assert!(inst.is_terminator());
    }

    #[test]
    fn test_call_result() {
        let result = make_val(0, "r");
        let inst = Instruction::Call {
            result: Some(result.clone()),
            function: "foo".into(),
            args: vec![],
        };
        assert!(inst.result_value().is_some());

        let inst2 = Instruction::Call {
            result: None,
            function: "bar".into(),
            args: vec![],
        };
        assert!(inst2.result_value().is_none());
    }

    #[test]
    fn test_cmp_op_display() {
        assert_eq!(format!("{}", CmpOp::Eq), "eq");
        assert_eq!(format!("{}", CmpOp::Lt), "lt");
        assert_eq!(format!("{}", CmpOp::Ge), "ge");
    }

    #[test]
    fn test_nop_no_result() {
        let inst = Instruction::Nop;
        assert!(inst.result_value().is_none());
        assert!(!inst.is_terminator());
    }

    #[test]
    fn test_operand_values_binary() {
        let lhs = make_val(1, "a");
        let rhs = make_val(2, "b");
        let inst = Instruction::IAdd {
            result: make_val(0, "r"),
            lhs: lhs.clone(),
            rhs: rhs.clone(),
        };
        let ops = inst.operand_values();
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0].id, 1);
        assert_eq!(ops[1].id, 2);
    }

    #[test]
    fn test_operand_values_unary() {
        let operand = make_val(1, "a");
        let inst = Instruction::INeg {
            result: make_val(0, "r"),
            operand: operand.clone(),
        };
        let ops = inst.operand_values();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].id, 1);
    }

    #[test]
    fn test_operand_values_none_for_const() {
        let inst = Instruction::Const {
            result: make_val(0, "c"),
            constant: Constant::Int(42),
        };
        assert!(inst.operand_values().is_empty());
    }
}
