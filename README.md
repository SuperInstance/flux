<div align="center">

# ██████╗ ███████╗ █████╗ ██╗  ████████╗██╗███╗   ███╗███████╗
# ██╔══██╗██╔════╝██╔══██╗██║  ╚══██╔══╝██║████╗ ████║██╔════╝
# ██████╔╝█████╗  ███████║██║     ██║   ██║██╔████╔██║█████╗
# ██╔══██╗██╔══╝  ██╔══██║██║     ██║   ██║██║╚██╔╝██║██╔══╝
# ██║  ██║███████╗██║  ██║███████╗██║   ██║██║ ╚═╝ ██║███████╗
# ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚══════╝╚═╝   ╚═╝╚═╝     ╚═╝╚══════╝

**Fluid Language Universal eXecution — The DJ Booth for Agent Code**

[![Build](https://img.shields.io/badge/build-passing-brightgreen)]()
[![License](https://img.shields.io/badge/license-MIT-blue)]()
[![Rust](https://img.shields.io/badge/rust-1.75+-orange)]()
[![Tests](https://img.shields.io/badge/tests-286-green)]()

</div>

---

FLUX is a high-performance runtime for agent-oriented code execution, rewritten in
Rust for maximum throughput. It features a 64-register bytecode VM, a full SSA
intermediate representation, and a FLUX.MD structured markdown parser.

## Features

- **100+ opcodes** across 10 categories (control, stack, integer, float, memory, agent, meta, bitwise, vector)
- **64-register VM** with 16 GP + 16 FP + 16 Vec + 16 Sys registers
- **6 encoding formats** (A/B/C/D/E/G) for compact bytecode
- **SSA IR** (FIR) with builder, validator, and type system
- **FLUX.MD parser** for structured markdown → AST → FIR compilation
- **286 tests** with 0 unsafe blocks
- **Zero-copy** bytecode loading and region-based memory manager

## Quick Start

```bash
# Build the CLI
cargo build --release

# Run tests
cargo test

# Show the banner
cargo run --bin flux

# Run the built-in demo
cargo run --bin flux -- demo

# Show subsystem info
cargo run --bin flux -- info

# Compile a simple C file to bytecode
cargo run --bin flux -- compile demo.c

# Execute a bytecode file
cargo run --bin flux -- run demo.bin
```

## Architecture

```
┌─────────────────────────────────────────────┐
│                  CLI (flux)                  │
│         compile / run / demo / info           │
└──────────┬──────────┬───────────┬────────────┘
           │          │           │
     ┌─────▼─────┐    │     ┌─────▼──────┐
     │  Parser   │    │     │    FIR     │
     │ FLUX.MD   │────┼────►│ SSA IR     │
     │ → AST     │    │     │ Builder    │
     └───────────┘    │     └──────┬─────┘
                      │            │
                      │     ┌──────▼─────┐
                      │     │ Bytecode   │
                      │     │ Encoder    │
                      │     │ 100+ ops   │
                      │     └──────┬─────┘
                      │            │
                      │     ┌──────▼─────┐
                      │     │    VM      │
                      └────►│ Interpreter│
                            │ 64 regs    │
                            └────────────┘
```

## Crate Overview

| Crate | Description | Tests |
|-------|-------------|-------|
| `flux-bytecode` | Opcodes, encoder, decoder, validator | 72 |
| `flux-vm` | 64-register virtual machine interpreter | 55 |
| `flux-fir` | SSA IR builder & validator | 67 |
| `flux-parser` | FLUX.MD parser & AST-to-FIR compiler | 92 |

## Code Examples

### Build bytecode

```rust
use flux_bytecode::{BytecodeEncoder, BytecodeHeader, Instruction, Op};

let mut enc = BytecodeEncoder::new();
enc.emit(&Instruction::imm(Op::IInc, 0, 42)).unwrap(); // R0 = 42
enc.emit(&Instruction::nullary(Op::Halt)).unwrap();

let bytecode = enc.finish(BytecodeHeader::default()).unwrap();
```

### Run the VM

```rust
use flux_bytecode::{BytecodeEncoder, Instruction, Op};
use flux_vm::{Interpreter, VmConfig};

let mut enc = BytecodeEncoder::new();
enc.emit(&Instruction::imm(Op::IInc, 0, 42)).unwrap();
enc.emit(&Instruction::nullary(Op::Halt)).unwrap();

let bytecode = enc.into_bytes();
let mut vm = Interpreter::new(&bytecode, VmConfig::default()).unwrap();
vm.execute().unwrap();
assert_eq!(vm.regs.read_gp(0), 42);
```

### Parse FLUX.MD

```rust
use flux_parser::FluxParser;

let source = r#"---
title: My Agent
version: 1.0
---

## Agent: Calculator

```flux
fn add(a: i64, b: i64) -> i64 {
    return a + b;
}
```
"#;

let mut parser = FluxParser::new(source);
let doc = parser.parse().unwrap();
```

### Build FIR

```rust
use flux_fir::{FirBuilder, FirModule, FirType, TypeContext};

let mut types = TypeContext::new();
let i64_ty = types.i64();
let mut module = FirModule::new("demo");

let mut builder = FirBuilder::new(&mut types, &mut module);
let func_id = builder.declare_function("main", vec![], i64_ty);
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Write tests for your changes (`cargo test`)
4. Ensure all 286+ tests pass (`cargo test`)
5. Commit with clear messages
6. Open a pull request

## License

This project is licensed under the [MIT License](LICENSE).

Copyright (c) 2026 SuperInstance
