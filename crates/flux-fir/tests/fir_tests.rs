use flux_fir::*;

// ── Type system tests ───────────────────────────────────────

#[test]
fn test_type_equality() {
    let ctx = TypeContext::new();
    assert_eq!(ctx.i32(), ctx.i32());
    assert_ne!(ctx.i32(), ctx.i64());
    assert_ne!(ctx.i32(), ctx.f32());
}

#[test]
fn test_nested_array() {
    let ctx = TypeContext::new();
    let nested = ctx.array(ctx.array(ctx.i32()));
    assert_eq!(
        nested,
        FirType::Array(Box::new(FirType::Array(Box::new(FirType::Int(32)))))
    );
}

#[test]
fn test_nested_map() {
    let ctx = TypeContext::new();
    let m = ctx.map(ctx.string(), ctx.array(ctx.i32()));
    assert_eq!(
        m,
        FirType::Map(
            Box::new(FirType::String),
            Box::new(FirType::Array(Box::new(FirType::Int(32))))
        )
    );
}

#[test]
fn test_unify_floats() {
    let ctx = TypeContext::new();
    let result = ctx.unify(&ctx.f32(), &ctx.f64());
    assert_eq!(result, Some(ctx.f64()));
}

#[test]
fn test_unify_incompatible_types() {
    let ctx = TypeContext::new();
    assert!(ctx.unify(&ctx.i32(), &ctx.boolean()).is_none());
    assert!(ctx.unify(&ctx.string(), &ctx.i32()).is_none());
}

// ── Value tests ─────────────────────────────────────────────

#[test]
fn test_value_clone() {
    let v = Value::new(0, "x", FirType::Int(32));
    let v2 = v.clone();
    assert_eq!(v.id, v2.id);
    assert_eq!(v.name, v2.name);
}

#[test]
fn test_constant_types() {
    assert_eq!(Constant::Int(42).ty(), FirType::Int(64));
    assert_eq!(Constant::Float(1.0).ty(), FirType::Float(64));
    assert_eq!(Constant::Bool(true).ty(), FirType::Bool);
    assert_eq!(Constant::String("hello".into()).ty(), FirType::String);
    assert_eq!(Constant::Bytes(vec![1, 2, 3]).ty(), FirType::Bytes);
    assert_eq!(Constant::Unit.ty(), FirType::Void);
}

// ── Builder integration tests ───────────────────────────────

#[test]
fn test_build_add_function() {
    let mut b = FirBuilder::new("test_module");
    b.create_function(
        "add",
        vec![
            ("a".into(), FirType::Int(32)),
            ("b".into(), FirType::Int(32)),
        ],
        FirType::Int(32),
    );
    let a = b.module().functions["add"].params[0].clone();
    let bv = b.module().functions["add"].params[1].clone();
    let sum = b.iadd(a, bv);
    b.ret(Some(sum));

    let m = b.into_module();
    let func = &m.functions["add"];
    assert_eq!(func.name, "add");
    assert_eq!(func.params.len(), 2);
    assert_eq!(func.blocks.len(), 1);
    assert_eq!(func.blocks[0].instructions.len(), 1);
    assert!(func.blocks[0].terminator.is_some());
    assert!(FirValidator::is_valid(&m));
}

#[test]
fn test_build_factorial() {
    let mut b = FirBuilder::new("test_module");
    b.create_function(
        "factorial",
        vec![("n".into(), FirType::Int(32))],
        FirType::Int(32),
    );

    let n = b.module().functions["factorial"].params[0].clone();
    let zero = b.const_int(0, FirType::Int(32));
    let cond = b.icmp(CmpOp::Le, n.clone(), zero);

    let then_bb = b.create_block("factorial", "base");
    let else_bb = b.create_block("factorial", "recurse");
    b.branch(cond, then_bb, else_bb);

    // base case
    b.position_at_end("factorial", then_bb);
    let one = b.const_int(1, FirType::Int(32));
    b.ret(Some(one));

    // recursive case
    b.position_at_end("factorial", else_bb);
    let one2 = b.const_int(1, FirType::Int(32));
    let n_minus_1 = b.isub(n.clone(), one2);
    let result = b.call("factorial", vec![n_minus_1], Some(FirType::Int(32)));
    let final_result = b.imul(n, result.unwrap());
    b.ret(Some(final_result));

    let m = b.into_module();
    assert!(FirValidator::is_valid(&m));
}

#[test]
fn test_build_max_function() {
    let mut b = FirBuilder::new("test_module");
    b.create_function(
        "max",
        vec![
            ("a".into(), FirType::Int(32)),
            ("b".into(), FirType::Int(32)),
        ],
        FirType::Int(32),
    );

    let a = b.module().functions["max"].params[0].clone();
    let bv = b.module().functions["max"].params[1].clone();
    let cond = b.icmp(CmpOp::Gt, a.clone(), bv.clone());

    let a_bb = b.create_block("max", "return_a");
    let b_bb = b.create_block("max", "return_b");
    b.branch(cond, a_bb, b_bb);

    b.position_at_end("max", a_bb);
    b.ret(Some(a));

    b.position_at_end("max", b_bb);
    b.ret(Some(bv));

    let m = b.into_module();
    assert!(FirValidator::is_valid(&m));
}

// ── Validator integration tests ─────────────────────────────

#[test]
fn test_valid_simple_module() {
    let mut b = FirBuilder::new("test");
    b.create_function("main", vec![], FirType::Void);
    b.ret(None);
    let m = b.into_module();
    assert!(FirValidator::is_valid(&m));
}

#[test]
fn test_valid_with_globals() {
    let mut b = FirBuilder::new("test");
    b.create_function("main", vec![], FirType::Void);
    b.ret(None);
    let mut m = b.into_module();
    m.globals.insert(
        "VERSION".to_string(),
        (
            Value::new(999, "VERSION", FirType::Int(32)),
            Constant::Int(1),
        ),
    );
    assert!(FirValidator::is_valid(&m));
}

#[test]
fn test_valid_with_metadata() {
    let mut b = FirBuilder::new("test");
    b.create_function("main", vec![], FirType::Void);
    b.ret(None);
    let mut m = b.into_module();
    m.metadata.insert("author".into(), "flux".into());
    assert!(FirValidator::is_valid(&m));
}

// ── Instruction tests ───────────────────────────────────────

#[test]
fn test_instruction_variants() {
    let v = Value::new(0, "x", FirType::Int(32));
    let _insts: Vec<Instruction> = vec![
        Instruction::Nop,
        Instruction::Jump { target: 0 },
        Instruction::Return { value: None },
        Instruction::Const { result: v.clone(), constant: Constant::Int(42) },
        Instruction::INeg { result: v.clone(), operand: v.clone() },
        Instruction::ABroadcast { message: v },
    ];
}

#[test]
fn test_all_agent_instructions() {
    let t = Value::new(0, "target", FirType::Agent);
    let m = Value::new(1, "msg", FirType::String);
    let q = Value::new(2, "q", FirType::String);

    let _send = Instruction::ASend { target: t.clone(), message: m.clone() };
    let _recv = Instruction::ARecv { result: t.clone() };
    let _ask = Instruction::AAsk { result: t.clone(), target: t.clone(), question: q.clone() };
    let _tell = Instruction::ATell { target: t.clone(), message: m };
    let _delegate = Instruction::ADelegate { result: t.clone(), target: t.clone(), task: q };
    let _broadcast = Instruction::ABroadcast { message: t };
}

// ── Module tests ────────────────────────────────────────────

#[test]
fn test_module_name() {
    let b = FirBuilder::new("my_crate");
    assert_eq!(b.module().name, "my_crate");
}

#[test]
fn test_empty_module_is_valid() {
    let m = FirModule::new("empty");
    // No functions, so nothing to validate — should be valid
    assert!(FirValidator::is_valid(&m));
}
