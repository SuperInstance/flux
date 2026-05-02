# FLUX

High-performance Rust runtime for agent-oriented code execution. Features a 64-register bytecode VM, SSA IR, and FLUX.MD structured markdown parser.

## Brand Line
> The DJ booth for agent code — FLUX.MD in, bytecode out, production-ready.

## Installation

```bash
git clone https://github.com/SuperInstance/flux.git
cd flux
cargo build --release
```

## Usage

```bash
# Run the hello world demo
cargo run --bin flux -- hello

# Compile a C file to bytecode
cargo run --bin flux -- compile demo.c

# Execute bytecode
cargo run --bin flux -- run demo.bin
```

## Fleet Context

Part of the Cocapn fleet. Related repos:
- [flux-os](https://github.com/SuperInstance/flux-os) — C microkernel OS where the kernel IS the compiler
- [flux-runtime](https://github.com/SuperInstance/flux-runtime) — Python reference implementation for research and prototyping
- [flux-runtime-c](https://github.com/SuperInstance/flux-runtime-c) — C port of the FLUX runtime

---
🦐 Cocapn fleet — lighthouse keeper architecture