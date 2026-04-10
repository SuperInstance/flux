# Changelog

All notable changes to the FLUX runtime will be documented in this file.

## [0.1.0] - 2026-01-01

### Added

#### flux-bytecode
- 100+ opcodes across 10 categories
- 6 encoding formats (A/B/C/D/E/G)
- `BytecodeEncoder` for instruction serialization
- `BytecodeDecoder` for instruction deserialization
- `BytecodeHeader` — 18-byte file preamble with magic, version, metadata
- `BytecodeValidator` for static bytecode verification
- Zero-copy design with `Instruction` value type
- 72 tests

#### flux-vm
- `Interpreter` — fetch-decode-execute loop with full opcode dispatch
- `RegisterFile` — 64 registers (16 GP + 16 FP + 16 Vec + 16 Sys) with SP/FP/LR aliasing
- `MemoryManager` — region-based linear memory with permissions
- `VmConfig` — configurable cycle limit, stack/heap sizes, trace mode
- Support for conditional branches, calls, returns, and stack operations
- Cycle counting and execution trace logging
- 55 tests including integration tests for all arithmetic, memory, and control flow

#### flux-fir
- `FirModule`, `FirFunction`, `BasicBlock` — SSA IR data structures
- `FirBuilder` — ergonomic API for building IR programs
- `FirValidator` — dominance, type, and SSA property verification
- `Instruction` enum with 20+ IR operations
- `TypeContext` with `FirType` (int, float, bool, void, ptr, function)
- `Value` representation (constant, parameter, instruction result)
- 67 tests

#### flux-parser
- `FluxParser` — recursive descent parser for FLUX.MD structured markdown
- `AstDocument`, `CodeBlock`, `TextSection`, `AgentDirective` — AST types
- `Frontmatter` parsing (YAML-style key-value metadata)
- `AstCompiler` — AST-to-FIR lowering pass
- `SourceSpan` for error reporting with line/column information
- 92 tests

#### CLI (`flux` binary)
- `flux run <file.bin>` — execute FLUX bytecode files
- `flux compile <file.c>` — compile simple C functions to bytecode
- `flux demo` — built-in demonstration program with trace output
- `flux info` — subsystem overview with formatted table
- `flux version` — version and build information
- `flux hello` — self-contained hello world demo (3+4=7)
- Beautiful ASCII art banner with ANSI color output
- Quick start section in help text

## [0.1.1] - 2026-04-10

### Added
- `flux hello` command — self-contained VM demo with architecture overview
- Quick start section in help text with link to Python sibling repo
- 0 unsafe blocks, Send+Sync VM

### Changed
- Updated README with cross-references to Python runtime
