use crate::blocks::{BasicBlock, BlockId, FirFunction, FirModule};
use crate::instructions::{CmpOp, Instruction};
use crate::types::{FirType, TypeContext};
use crate::values::{Constant, Value};

/// SSA builder that constructs FIR modules instruction by instruction.
pub struct FirBuilder {
    module: FirModule,
    current_function: Option<String>,
    current_block: Option<BlockId>,
    next_value_id: u32,
    next_block_id: u32,
    type_ctx: TypeContext,
}

impl FirBuilder {
    pub fn new(module_name: impl Into<String>) -> Self {
        Self {
            module: FirModule::new(module_name),
            current_function: None,
            current_block: None,
            next_value_id: 0,
            next_block_id: 0,
            type_ctx: TypeContext::new(),
        }
    }

    pub fn type_ctx(&self) -> &TypeContext {
        &self.type_ctx
    }

    /// Allocates the next value ID and returns it.
    fn next_id(&mut self) -> u32 {
        let id = self.next_value_id;
        self.next_value_id += 1;
        id
    }

    /// Allocates the next block ID and returns it.
    fn next_block(&mut self) -> BlockId {
        let id = self.next_block_id;
        self.next_block_id += 1;
        BlockId(id)
    }

    /// Creates a new function and adds it to the module.
    /// Returns a mutable reference (through an unsafe trick — we return () and
    /// rely on the caller not holding references). Instead, we return a
    /// confirmation.
    pub fn create_function(
        &mut self,
        name: &str,
        params: Vec<(String, FirType)>,
        ret: FirType,
    ) {
        let values: Vec<Value> = params
            .iter()
            .enumerate()
            .map(|(_i, (pname, ty))| Value::new(self.next_id(), pname.as_str(), ty.clone()))
            .collect();

        let entry = self.next_block();
        let mut func = FirFunction::new(name, values, ret);
        func.entry_block = entry;

        let bb = BasicBlock::new(entry, "entry");
        func.blocks.push(bb);

        self.module.functions.insert(name.to_string(), func);
        self.current_function = Some(name.to_string());
        self.current_block = Some(entry);
    }

    /// Creates a new block within a function. Returns the block's ID.
    pub fn create_block(&mut self, func_name: &str, block_name: &str) -> BlockId {
        let id = self.next_block();
        let bb = BasicBlock::new(id, block_name);
        if let Some(func) = self.module.functions.get_mut(func_name) {
            func.blocks.push(bb);
        }
        id
    }

    /// Sets the entry block of a function.
    pub fn set_entry_block(&mut self, func_name: &str, block_id: BlockId) {
        if let Some(func) = self.module.functions.get_mut(func_name) {
            func.entry_block = block_id;
        }
    }

    /// Positions the builder at the end of a specific block.
    pub fn position_at_end(&mut self, func_name: &str, block_id: BlockId) {
        self.current_function = Some(func_name.to_string());
        self.current_block = Some(block_id);
    }

    /// Internal helper: append an instruction to the current block.
    fn emit(&mut self, inst: Instruction) {
        if let (Some(func_name), Some(block_id)) =
            (&self.current_function, self.current_block)
        {
            if let Some(func) = self.module.functions.get_mut(func_name.as_str()) {
                if let Some(bb) = func.block_by_id_mut(block_id) {
                    bb.push_instruction(inst);
                }
            }
        }
    }

    /// Creates a fresh result value with an auto-generated name.
    fn fresh_value(&mut self, name_prefix: &str, ty: FirType) -> Value {
        let id = self.next_id();
        let name = format!("{}_{}", name_prefix, id);
        Value::new(id, name, ty)
    }

    // ── Arithmetic ──────────────────────────────────────────

    pub fn iadd(&mut self, lhs: Value, rhs: Value) -> Value {
        let result = self.fresh_value("iadd", lhs.ty.clone());
        self.emit(Instruction::IAdd {
            result: result.clone(),
            lhs,
            rhs,
        });
        result
    }

    pub fn isub(&mut self, lhs: Value, rhs: Value) -> Value {
        let result = self.fresh_value("isub", lhs.ty.clone());
        self.emit(Instruction::ISub {
            result: result.clone(),
            lhs,
            rhs,
        });
        result
    }

    pub fn imul(&mut self, lhs: Value, rhs: Value) -> Value {
        let result = self.fresh_value("imul", lhs.ty.clone());
        self.emit(Instruction::IMul {
            result: result.clone(),
            lhs,
            rhs,
        });
        result
    }

    pub fn idiv(&mut self, lhs: Value, rhs: Value) -> Value {
        let result = self.fresh_value("idiv", lhs.ty.clone());
        self.emit(Instruction::IDiv {
            result: result.clone(),
            lhs,
            rhs,
        });
        result
    }

    pub fn imod(&mut self, lhs: Value, rhs: Value) -> Value {
        let result = self.fresh_value("imod", lhs.ty.clone());
        self.emit(Instruction::IMod {
            result: result.clone(),
            lhs,
            rhs,
        });
        result
    }

    pub fn fadd(&mut self, lhs: Value, rhs: Value) -> Value {
        let result = self.fresh_value("fadd", lhs.ty.clone());
        self.emit(Instruction::FAdd {
            result: result.clone(),
            lhs,
            rhs,
        });
        result
    }

    pub fn fsub(&mut self, lhs: Value, rhs: Value) -> Value {
        let result = self.fresh_value("fsub", lhs.ty.clone());
        self.emit(Instruction::FSub {
            result: result.clone(),
            lhs,
            rhs,
        });
        result
    }

    pub fn fmul(&mut self, lhs: Value, rhs: Value) -> Value {
        let result = self.fresh_value("fmul", lhs.ty.clone());
        self.emit(Instruction::FMul {
            result: result.clone(),
            lhs,
            rhs,
        });
        result
    }

    // ── Comparison ──────────────────────────────────────────

    pub fn icmp(&mut self, op: CmpOp, lhs: Value, rhs: Value) -> Value {
        let result = self.fresh_value("icmp", FirType::Bool);
        let inst = match op {
            CmpOp::Eq => Instruction::ICmpEq { result: result.clone(), lhs, rhs },
            CmpOp::Ne => Instruction::ICmpNe { result: result.clone(), lhs, rhs },
            CmpOp::Lt => Instruction::ICmpLt { result: result.clone(), lhs, rhs },
            CmpOp::Le => Instruction::ICmpLe { result: result.clone(), lhs, rhs },
            CmpOp::Gt => Instruction::ICmpGt { result: result.clone(), lhs, rhs },
            CmpOp::Ge => Instruction::ICmpGe { result: result.clone(), lhs, rhs },
        };
        self.emit(inst);
        result
    }

    // ── Memory ──────────────────────────────────────────────

    pub fn load(&mut self, ptr: Value, ty: FirType) -> Value {
        let result = self.fresh_value("load", ty.clone());
        self.emit(Instruction::Load {
            result: result.clone(),
            ptr,
            ty,
        });
        result
    }

    pub fn store(&mut self, ptr: Value, value: Value, ty: FirType) {
        self.emit(Instruction::Store { ptr, value, ty });
    }

    pub fn alloc(&mut self, ty: FirType) -> Value {
        let result = self.fresh_value("alloc", FirType::Int(64)); // pointer
        self.emit(Instruction::Alloc {
            result: result.clone(),
            ty,
        });
        result
    }

    // ── Constants ───────────────────────────────────────────

    pub fn const_int(&mut self, val: i64, ty: FirType) -> Value {
        let result = self.fresh_value("const", ty.clone());
        self.emit(Instruction::Const {
            result: result.clone(),
            constant: Constant::Int(val),
        });
        result
    }

    pub fn const_float(&mut self, val: f64) -> Value {
        let result = self.fresh_value("const", FirType::Float(64));
        self.emit(Instruction::Const {
            result: result.clone(),
            constant: Constant::Float(val),
        });
        result
    }

    pub fn const_bool(&mut self, val: bool) -> Value {
        let result = self.fresh_value("const", FirType::Bool);
        self.emit(Instruction::Const {
            result: result.clone(),
            constant: Constant::Bool(val),
        });
        result
    }

    // ── Control flow ────────────────────────────────────────

    pub fn call(&mut self, func: &str, args: Vec<Value>, ret: Option<FirType>) -> Option<Value> {
        let result = ret.map(|ty| self.fresh_value("call", ty));
        self.emit(Instruction::Call {
            result: result.clone(),
            function: func.to_string(),
            args,
        });
        result
    }

    pub fn ret(&mut self, value: Option<Value>) {
        self.emit(Instruction::Return { value });
    }

    pub fn jump(&mut self, target: BlockId) {
        self.emit(Instruction::Jump { target: target.0 });
    }

    pub fn branch(&mut self, cond: Value, true_block: BlockId, false_block: BlockId) {
        self.emit(Instruction::Branch {
            condition: cond,
            true_block: true_block.0,
            false_block: false_block.0,
        });
    }

    // ── Meta ────────────────────────────────────────────────

    pub fn cast(&mut self, value: Value, target: FirType) -> Value {
        let result = self.fresh_value("cast", target.clone());
        self.emit(Instruction::Cast {
            result: result.clone(),
            value,
            target_ty: target,
        });
        result
    }

    // ── Accessors ───────────────────────────────────────────

    pub fn module(&self) -> &FirModule {
        &self.module
    }

    pub fn into_module(self) -> FirModule {
        self.module
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_new() {
        let b = FirBuilder::new("test");
        assert_eq!(b.module().name, "test");
    }

    #[test]
    fn test_create_function() {
        let mut b = FirBuilder::new("test");
        b.create_function("add", vec![("a".into(), FirType::Int(32)), ("b".into(), FirType::Int(32))], FirType::Int(32));
        let m = b.module();
        assert!(m.functions.contains_key("add"));
        let func = &m.functions["add"];
        assert_eq!(func.params.len(), 2);
        assert_eq!(func.params[0].name, "a");
        assert_eq!(func.params[1].name, "b");
    }

    #[test]
    fn test_create_block() {
        let mut b = FirBuilder::new("test");
        b.create_function("foo", vec![], FirType::Void);
        let bb = b.create_block("foo", "loop_body");
        assert_eq!(bb.0, 1); // 0 is entry
    }

    #[test]
    fn test_set_entry_block() {
        let mut b = FirBuilder::new("test");
        b.create_function("foo", vec![], FirType::Void);
        let bb = b.create_block("foo", "new_entry");
        b.set_entry_block("foo", bb);
        assert_eq!(b.module().functions["foo"].entry_block, bb);
    }

    #[test]
    fn test_position_at_end() {
        let mut b = FirBuilder::new("test");
        b.create_function("foo", vec![], FirType::Void);
        let bb = b.create_block("foo", "bb1");
        b.position_at_end("foo", bb);
        assert_eq!(b.current_block, Some(bb));
    }

    #[test]
    fn test_const_int_and_return() {
        let mut b = FirBuilder::new("test");
        b.create_function("identity", vec![("x".into(), FirType::Int(32))], FirType::Int(32));
        let val = b.const_int(42, FirType::Int(32));
        b.ret(Some(val));
        let m = b.module();
        let func = &m.functions["identity"];
        let entry = &func.blocks[0];
        assert_eq!(entry.instructions.len(), 1);
        assert!(entry.terminator.is_some());
    }

    #[test]
    fn test_arithmetic() {
        let mut b = FirBuilder::new("test");
        b.create_function("add", vec![("a".into(), FirType::Int(32)), ("b".into(), FirType::Int(32))], FirType::Int(32));
        let a = b.module().functions["add"].params[0].clone();
        let bv = b.module().functions["add"].params[1].clone();
        let sum = b.iadd(a, bv);
        b.ret(Some(sum));
        let m = b.module();
        let func = &m.functions["add"];
        assert_eq!(func.blocks[0].instructions.len(), 1);
    }

    #[test]
    fn test_float_ops() {
        let mut b = FirBuilder::new("test");
        b.create_function("fadd", vec![("x".into(), FirType::Float(64)), ("y".into(), FirType::Float(64))], FirType::Float(64));
        let x = b.module().functions["fadd"].params[0].clone();
        let y = b.module().functions["fadd"].params[1].clone();
        let sum = b.fadd(x, y);
        b.ret(Some(sum));
        let m = b.module();
        assert_eq!(m.functions["fadd"].blocks[0].instructions.len(), 1);
    }

    #[test]
    fn test_icmp_and_branch() {
        let mut b = FirBuilder::new("test");
        b.create_function("cmp_test", vec![("x".into(), FirType::Int(32))], FirType::Void);
        let x = b.module().functions["cmp_test"].params[0].clone();
        let zero = b.const_int(0, FirType::Int(32));
        let cond = b.icmp(CmpOp::Gt, x, zero);
        let then_bb = b.create_block("cmp_test", "then");
        let else_bb = b.create_block("cmp_test", "else");
        b.branch(cond, then_bb, else_bb);
        let m = b.module();
        let entry = &m.functions["cmp_test"].blocks[0];
        assert_eq!(entry.instructions.len(), 2); // const + icmp
        assert!(entry.terminator.is_some());
    }

    #[test]
    fn test_alloc_and_store() {
        let mut b = FirBuilder::new("test");
        b.create_function("store_test", vec![], FirType::Void);
        let ptr = b.alloc(FirType::Int(32));
        let val = b.const_int(99, FirType::Int(32));
        b.store(ptr, val, FirType::Int(32));
        b.ret(None);
        let m = b.module();
        let entry = &m.functions["store_test"].blocks[0];
        assert_eq!(entry.instructions.len(), 3); // alloc + const + store
    }

    #[test]
    fn test_call() {
        let mut b = FirBuilder::new("test");
        b.create_function("caller", vec![], FirType::Int(32));
        let result = b.call("helper", vec![], Some(FirType::Int(32)));
        assert!(result.is_some());
        b.ret(result);
    }

    #[test]
    fn test_void_return() {
        let mut b = FirBuilder::new("test");
        b.create_function("void_fn", vec![], FirType::Void);
        b.ret(None);
        let m = b.module();
        let func = &m.functions["void_fn"];
        assert!(func.blocks[0].terminator.is_some());
    }

    #[test]
    fn test_into_module() {
        let mut b = FirBuilder::new("test");
        b.create_function("main", vec![], FirType::Void);
        b.ret(None);
        let m = b.into_module();
        assert!(m.functions.contains_key("main"));
    }

    #[test]
    fn test_multiple_functions() {
        let mut b = FirBuilder::new("test");
        b.create_function("foo", vec![], FirType::Void);
        b.ret(None);
        b.create_function("bar", vec![("x".into(), FirType::Int(32))], FirType::Int(32));
        let x = b.module().functions["bar"].params[0].clone();
        b.ret(Some(x));
        let m = b.module();
        assert_eq!(m.functions.len(), 2);
    }

    #[test]
    fn test_jump() {
        let mut b = FirBuilder::new("test");
        b.create_function("jump_test", vec![], FirType::Void);
        let target = b.create_block("jump_test", "target");
        b.jump(target);
        let m = b.module();
        assert!(m.functions["jump_test"].blocks[0].terminator.is_some());
    }

    #[test]
    fn test_cast() {
        let mut b = FirBuilder::new("test");
        b.create_function("cast_test", vec![("x".into(), FirType::Int(32))], FirType::Float(64));
        let x = b.module().functions["cast_test"].params[0].clone();
        let f = b.cast(x, FirType::Float(64));
        b.ret(Some(f));
        let m = b.module();
        let entry = &m.functions["cast_test"].blocks[0];
        assert_eq!(entry.instructions.len(), 1);
    }
}
