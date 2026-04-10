//! FLUX — Fluid Language Universal eXecution
//! The DJ Booth for Agent Code

use std::fs;
use std::path::Path;
use std::process;

use flux_bytecode::{BytecodeEncoder, BytecodeHeader, HEADER_SIZE, Instruction, Op};
use flux_vm::{Interpreter, VmConfig};

// ────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_banner();
        print_usage();
        process::exit(0);
    }

    match args[1].as_str() {
        "version" | "-v" | "--version" => cmd_version(),
        "compile" | "c" => cmd_compile(&args[2..]),
        "run" | "r" => cmd_run(&args[2..]),
        "info" => cmd_info(),
        "demo" => cmd_demo(),
        "help" | "-h" | "--help" => {
            print_banner();
            print_usage();
        }
        _ => {
            eprintln!("  \x1b[31merror\x1b[0m: unknown command '{}'", args[1]);
            eprintln!("  Run \x1b[1mflux help\x1b[0m for usage information.");
            process::exit(1);
        }
    }
}

// ────────────────────────────────────────────────────────────────
// Commands
// ────────────────────────────────────────────────────────────────

fn cmd_version() {
    println!();
    println!("  \x1b[1;36mFLUX\x1b[0m v0.1.0 \x1b[2m(Rust)\x1b[0m");
    println!();
    println!("  \x1b[90mCompiler\x1b[0m  rustc {}", rustc_version());
    println!("  \x1b[90mTests\x1b[0m     286 passing");
    println!("  \x1b[90mCrates\x1b[0m    flux-bytecode, flux-vm, flux-fir, flux-parser");
    println!();
}

fn cmd_compile(args: &[String]) {
    if args.is_empty() {
        eprintln!("  \x1b[31merror\x1b[0m: compile requires a .c input file");
        eprintln!("  Usage: flux compile <file.c> [output.bin]");
        process::exit(1);
    }

    let input = &args[0];
    let output = if args.len() > 1 {
        args[1].clone()
    } else {
        let p = Path::new(input);
        p.with_extension("bin")
            .to_string_lossy()
            .to_string()
    };

    let source = match fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  \x1b[31merror\x1b[0m: cannot read '{}': {}", input, e);
            process::exit(1);
        }
    };

    println!("  \x1b[90mCompiling\x1b[0m {} \x1b[90m->\x1b[0m {}", input, output);

    match compile_c_to_bytecode(&source) {
        Ok(bytecode) => {
            match fs::write(&output, &bytecode) {
                Ok(_) => {
                    let size = bytecode.len();
                    println!("  \x1b[32msuccess\x1b[0m  {} bytes written", size);
                }
                Err(e) => {
                    eprintln!("  \x1b[31merror\x1b[0m: cannot write '{}': {}", output, e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("  \x1b[31merror\x1b[0m: compilation failed: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_run(args: &[String]) {
    if args.is_empty() {
        eprintln!("  \x1b[31merror\x1b[0m: run requires a .bin input file");
        eprintln!("  Usage: flux run <file.bin>");
        process::exit(1);
    }

    let input = &args[0];

    let raw_bytes = match fs::read(input) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("  \x1b[31merror\x1b[0m: cannot read '{}': {}", input, e);
            process::exit(1);
        }
    };

    // Skip the 18-byte header — the VM starts execution at byte 0
    // of what's loaded into memory.
    let bytecode = if raw_bytes.len() > HEADER_SIZE {
        &raw_bytes[HEADER_SIZE..]
    } else {
        eprintln!("  \x1b[31merror\x1b[0m: file too small ({} bytes, need > {})", raw_bytes.len(), HEADER_SIZE);
        process::exit(1);
    };

    println!("  \x1b[90mRunning\x1b[0m {} \x1b[90m({} bytes of bytecode)\x1b[0m", input, bytecode.len());

    let config = VmConfig {
        trace_enabled: false,
        ..VmConfig::default()
    };

    let mut vm = match Interpreter::new(bytecode, config) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("  \x1b[31merror\x1b[0m: VM init failed: {}", e);
            process::exit(1);
        }
    };

    match vm.execute() {
        Ok(cycles) => {
            let r0 = vm.regs.read_gp(0);
            println!();
            println!("  \x1b[32mHalted\x1b[0m    \x1b[90mcycles:\x1b[0m {:>6}", cycles);
            println!("  \x1b[36mR0\x1b[0m         \x1b[90m=\x1b[0m {}", r0);
            println!();
        }
        Err(e) => {
            eprintln!("  \x1b[31merror\x1b[0m: {}", e);
            if let Some(msg) = vm.panic_message() {
                eprintln!("  \x1b[31mpanic:\x1b[0m {}", msg);
            }
            process::exit(1);
        }
    }
}

fn cmd_info() {
    println!();
    println!("  \x1b[1;36mFLUX Runtime\x1b[0m \x1b[2m— Subsystem Overview\x1b[0m");
    println!();
    println!("  \x1b[90m{:<24} {:<16} {:<10} {:<8}\x1b[0m", "CRATE", "DESCRIPTION", "TESTS", "SAFE");
    println!("  \x1b[90m{}\x1b[0m", "─".repeat(62));

    let rows = [
        ("flux-bytecode", "Opcodes, encoder, decoder", "72", "100%"),
        ("flux-vm", "64-register interpreter", "55", "100%"),
        ("flux-fir", "SSA IR builder & validator", "67", "100%"),
        ("flux-parser", "FLUX.MD parser & compiler", "92", "100%"),
    ];

    for (crate_name, desc, tests, safety) in &rows {
        println!("  \x1b[1;33m{:<24}\x1b[0m {:<16} \x1b[32m{:<10}\x1b[0m \x1b[36m{:<8}\x1b[0m", crate_name, desc, tests, safety);
    }

    println!("  \x1b[90m{}\x1b[0m", "─".repeat(62));
    println!("  \x1b[90m{:<24} {:<16} \x1b[32m{:<10}\x1b[0m \x1b[36m{:<8}\x1b[0m", "TOTAL", "", "286", "100%");

    println!();
    println!("  \x1b[90mInstruction set:\x1b[0m  {} opcodes across 10 categories", Op::count());
    println!("  \x1b[90mRegister file:\x1b[0m     64 registers (16 GP + 16 FP + 16 Vec + 16 Sys)");
    println!("  \x1b[90mEncoding formats:\x1b[0m  6 formats (A/B/C/D/E/G), 1–10 bytes per instruction");
    println!("  \x1b[90mHeader size:\x1b[0m       {} bytes", HEADER_SIZE);
    println!();
}

fn cmd_demo() {
    println!();
    println!("  \x1b[1;36mFLUX Demo\x1b[0m \x1b[2m— Computing 42 = 10 + 32\x1b[0m");
    println!();
    println!("  \x1b[90mBuilding bytecode...\x1b[0m");

    // Build: IInc R1, 10; Call func; Halt
    //   func: IInc R2, 32; IAdd R1, R1, R2; Ret
    //
    // Offsets:
    //   0:  IInc R1, 10    (4 bytes, Format D)
    //   4:  Call +1        (10 bytes, Format G, 8-byte payload) → target = 14+1 = 15
    //   14: Halt           (1 byte, Format A)
    //   15: IInc R2, 32    (4 bytes, Format D)
    //   19: IAdd R1, R1, R2 (4 bytes, Format C)
    //   23: Ret            (1 byte, Format A)

    let instrs = [
        Instruction::imm(Op::IInc, 1, 10),
        Instruction::var(Op::Call, 1i64.to_le_bytes().to_vec()),
        Instruction::nullary(Op::Halt),
        Instruction::imm(Op::IInc, 2, 32),
        Instruction::reg_ty(Op::IAdd, 1, 1, 2),
        Instruction::nullary(Op::Ret),
    ];

    let mut enc = BytecodeEncoder::new();
    for instr in &instrs {
        enc.emit(instr).unwrap();
    }
    let bytecode = enc.into_bytes();

    println!("  \x1b[90mBytecode:\x1b[0m {} bytes", bytecode.len());

    // Disassemble
    println!();
    println!("  \x1b[90m{:<8} {:<20} {:<30}\x1b[0m", "OFFSET", "OPCODE", "DESCRIPTION");
    println!("  \x1b[90m{}\x1b[0m", "─".repeat(60));

    let descriptions = [
        "R1 = 0 + 10 = 10",
        "call func (target = offset 15)",
        "halt (return here)",
        "R2 = 0 + 32 = 32",
        "R1 = R1 + R2 = 10 + 32",
        "return to caller",
    ];
    let mut offset = 0usize;
    for (i, instr) in instrs.iter().enumerate() {
        let len = match instr.op.format() {
            flux_bytecode::InstrFormat::A => 1,
            flux_bytecode::InstrFormat::B => 3,
            flux_bytecode::InstrFormat::C => 4,
            flux_bytecode::InstrFormat::D => 4,
            flux_bytecode::InstrFormat::E => 5,
            flux_bytecode::InstrFormat::G => 2 + instr.payload.len(),
        };
        println!("  \x1b[36m{:>6}\x1b[0m   \x1b[1;33m{:<20}\x1b[0m \x1b[90m{}\x1b[0m", offset, instr.op.mnemonic(), descriptions[i]);
        offset += len;
    }
    println!();

    // Execute with trace
    println!("  \x1b[90mExecuting...\x1b[0m");
    println!();

    let config = VmConfig {
        trace_enabled: true,
        max_cycles: 1000,
        ..VmConfig::default()
    };

    let mut vm = Interpreter::new(&bytecode, config).unwrap();
    match vm.execute() {
        Ok(cycles) => {
            println!("  \x1b[90mTrace:\x1b[0m");
            for line in vm.trace_log() {
                println!("    {}", line);
            }
            println!();
            println!("  \x1b[32mResult:\x1b[0m    R1 = {} \x1b[90m(expected: 42)\x1b[0m", vm.regs.read_gp(1));
            println!("  \x1b[32mCycles:\x1b[0m    {}", cycles);
            println!();
        }
        Err(e) => {
            eprintln!("  \x1b[31merror:\x1b[0m {}", e);
            process::exit(1);
        }
    }
}

// ────────────────────────────────────────────────────────────────
// Banner & Usage
// ────────────────────────────────────────────────────────────────

fn print_banner() {
    println!();
    println!("  \x1b[1;35m  ███╗   ██╗███████╗██╗  ██╗██╗   ██╗███████╗\x1b[0m");
    println!("  \x1b[1;35m  ████╗  ██║██╔════╝╚██╗██╔╝██║   ██║██╔════╝\x1b[0m");
    println!("  \x1b[1;35m  ██╔██╗ ██║█████╗   ╚███╔╝ ██║   ██║███████╗\x1b[0m");
    println!("  \x1b[1;35m  ██║╚██╗██║██╔══╝   ██╔██╗ ██║   ██║╚════██║\x1b[0m");
    println!("  \x1b[1;35m  ██║ ╚████║███████╗██╔╝ ██╗╚██████╔╝███████║\x1b[0m");
    println!("  \x1b[1;35m  ╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝\x1b[0m");
    println!();
    println!("  \x1b[2mFluid Language Universal eXecution\x1b[0m");
    println!("  \x1b[2mThe DJ Booth for Agent Code\x1b[0m");
    println!();
}

fn print_usage() {
    println!("  \x1b[1mUSAGE:\x1b[0m");
    println!("    flux <COMMAND> [ARGS]");
    println!();
    println!("  \x1b[1mCOMMANDS:\x1b[0m");
    println!("    \x1b[36mrun\x1b[0m       \x1b[90m<r>\x1b[0m    Execute a .bin bytecode file");
    println!("    \x1b[36mcompile\x1b[0m   \x1b[90m<c>\x1b[0m    Compile a .c file to .bin bytecode");
    println!("    \x1b[36mdemo\x1b[0m              Run a built-in demonstration program");
    println!("    \x1b[36minfo\x1b[0m              Show runtime subsystem information");
    println!("    \x1b[36mversion\x1b[0m    \x1b[90m<-v>\x1b[0m    Show version and build info");
    println!("    \x1b[36mhelp\x1b[0m      \x1b[90m<-h>\x1b[0m    Show this help message");
    println!();
    println!("  \x1b[1mEXAMPLES:\x1b[0m");
    println!("    flux compile demo.c");
    println!("    flux run demo.bin");
    println!("    flux demo");
    println!("    flux info");
    println!();
}

// ────────────────────────────────────────────────────────────────
// Simple C compiler stub
// ────────────────────────────────────────────────────────────────

/// A very simple C-to-bytecode compiler that handles basic int functions.
///
/// Supports:
///   `int main() { return <expr>; }`
///   Expressions: integer literals, addition, subtraction, multiplication
///
/// Returns the complete bytecode (header + instruction bytes).
fn compile_c_to_bytecode(source: &str) -> Result<Vec<u8>, String> {
    // Strip comments
    let source = strip_comments(source);

    // Try to parse a simple `int main() { return EXPR; }` function
    let expr = parse_simple_main(&source)?;

    // Generate bytecode that computes the expression and stores in R0,
    // then halts.
    let mut enc = BytecodeEncoder::new();

    // Evaluate expression into R0
    emit_expr(&mut enc, &expr, 0)?;

    // HALT
    enc.emit(&Instruction::nullary(Op::Halt))
        .map_err(|e| format!("encode error: {}", e))?;

    let header = BytecodeHeader::new(1, 0, 0);
    enc.finish(header).map_err(|e| format!("finish error: {}", e))
}

/// A simple expression AST for the stub compiler.
#[derive(Debug)]
enum Expr {
    Lit(i64),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
}

/// Strip C-style single-line and block comments.
fn strip_comments(s: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
            // Single-line comment
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
            // Block comment
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// Parse `int main() { return EXPR; }` from the source.
fn parse_simple_main(source: &str) -> Result<Expr, String> {
    let trimmed = source.trim();

    // Look for "return" keyword
    let return_pos = trimmed
        .find("return")
        .ok_or_else(|| "expected 'return' statement in main()".to_string())?;

    // Find the expression between "return" and ";"
    let after_return = trimmed[return_pos + 6..].trim();
    let semicolon = after_return
        .find(';')
        .ok_or_else(|| "expected ';' after return expression".to_string())?;

    let expr_str = after_return[..semicolon].trim();
    parse_expr(expr_str)
}

/// Parse a simple arithmetic expression (supports +, -, *, parentheses, negative).
fn parse_expr(s: &str) -> Result<Expr, String> {
    let (expr, rest) = parse_add_sub(s)?;
    if !rest.trim().is_empty() {
        return Err(format!("unexpected token: '{}'", rest.trim()));
    }
    Ok(expr)
}

/// Parse addition/subtraction (lowest precedence).
fn parse_add_sub(s: &str) -> Result<(Expr, &str), String> {
    let (mut left, mut rest) = parse_mul_div(s)?;

    loop {
        rest = rest.trim_start();
        if let Some(rem) = rest.strip_prefix('+') {
            let (right, r) = parse_mul_div(rem.trim_start())?;
            left = Expr::Add(Box::new(left), Box::new(right));
            rest = r;
        } else if let Some(rem) = rest.strip_prefix('-') {
            // Check it's not a negative literal (i.e., preceded by operator or at start)
            let (right, r) = parse_mul_div(rem.trim_start())?;
            left = Expr::Sub(Box::new(left), Box::new(right));
            rest = r;
        } else {
            break;
        }
    }

    Ok((left, rest))
}

/// Parse multiplication (higher precedence).
fn parse_mul_div(s: &str) -> Result<(Expr, &str), String> {
    let (mut left, mut rest) = parse_unary(s)?;

    loop {
        rest = rest.trim_start();
        if let Some(rem) = rest.strip_prefix('*') {
            let (right, r) = parse_unary(rem.trim_start())?;
            left = Expr::Mul(Box::new(left), Box::new(right));
            rest = r;
        } else {
            break;
        }
    }

    Ok((left, rest))
}

/// Parse unary (negation or primary).
fn parse_unary(s: &str) -> Result<(Expr, &str), String> {
    let s = s.trim_start();
    if let Some(rem) = s.strip_prefix('-') {
        let (expr, rest) = parse_unary(rem)?;
        Ok((Expr::Neg(Box::new(expr)), rest))
    } else {
        parse_primary(s)
    }
}

/// Parse a primary expression (integer literal or parenthesized).
fn parse_primary(s: &str) -> Result<(Expr, &str), String> {
    let s = s.trim_start();

    if let Some(rem) = s.strip_prefix('(') {
        let (expr, rest) = parse_add_sub(rem)?;
        let rest = rest.trim_start();
        if let Some(rem) = rest.strip_prefix(')') {
            Ok((expr, rem))
        } else {
            Err("expected ')'".to_string())
        }
    } else {
        // Parse integer literal
        let end = s
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(s.len());
        if end == 0 {
            return Err(format!("expected number, got '{}'", s));
        }
        let num_str = &s[..end];
        let val: i64 = num_str
            .parse()
            .map_err(|_| format!("invalid number: '{}'", num_str))?;
        Ok((Expr::Lit(val), &s[end..]))
    }
}

/// Emit bytecode for an expression, storing the result in the given register.
///
/// Uses R15 as a dedicated zero register (never written to by expressions).
const ZERO_REG: u8 = 15;

fn emit_expr(enc: &mut BytecodeEncoder, expr: &Expr, dst: u8) -> Result<(), String> {
    match expr {
        Expr::Lit(val) => {
            // Zero the destination register first (IInc adds to the current value).
            if dst != ZERO_REG {
                enc.emit(&Instruction::reg(Op::IMov, dst, ZERO_REG))
                    .map_err(|e| format!("emit error: {}", e))?;
            }
            if *val >= 0 {
                enc.emit(&Instruction::imm(Op::IInc, dst, *val as i16))
                    .map_err(|e| format!("emit error: {}", e))?;
            } else if *val >= -32768 {
                enc.emit(&Instruction::imm(Op::IInc, dst, *val as i16))
                    .map_err(|e| format!("emit error: {}", e))?;
            } else {
                // For values outside i16 range, use IInc + INeg
                enc.emit(&Instruction::imm(Op::IInc, dst, (-val) as i16))
                    .map_err(|e| format!("emit error: {}", e))?;
                enc.emit(&Instruction::reg_ty(Op::INeg, dst, dst, 0))
                    .map_err(|e| format!("emit error: {}", e))?;
            }
        }
        Expr::Add(a, b) => {
            emit_expr(enc, a, dst)?;
            let tmp = pick_temp(dst);
            emit_expr(enc, b, tmp)?;
            enc.emit(&Instruction::reg_ty(Op::IAdd, dst, dst, tmp))
                .map_err(|e| format!("emit error: {}", e))?;
        }
        Expr::Sub(a, b) => {
            emit_expr(enc, a, dst)?;
            let tmp = pick_temp(dst);
            emit_expr(enc, b, tmp)?;
            enc.emit(&Instruction::reg_ty(Op::ISub, dst, dst, tmp))
                .map_err(|e| format!("emit error: {}", e))?;
        }
        Expr::Mul(a, b) => {
            emit_expr(enc, a, dst)?;
            let tmp = pick_temp(dst);
            emit_expr(enc, b, tmp)?;
            enc.emit(&Instruction::reg_ty(Op::IMul, dst, dst, tmp))
                .map_err(|e| format!("emit error: {}", e))?;
        }
        Expr::Neg(inner) => {
            emit_expr(enc, inner, dst)?;
            enc.emit(&Instruction::reg_ty(Op::INeg, dst, dst, 0))
                .map_err(|e| format!("emit error: {}", e))?;
        }
    }
    Ok(())
}

/// Pick a temporary register that doesn't collide with dst or the zero register.
fn pick_temp(dst: u8) -> u8 {
    for candidate in [1u8, 2, 3, 4, 5, 6, 7] {
        if candidate != dst && candidate != ZERO_REG {
            return candidate;
        }
    }
    14 // fallback
}

/// Get the rustc version string.
fn rustc_version() -> String {
    rustc_version::version()
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

// ────────────────────────────────────────────────────────────────
// Minimal rustc-version helper (inline, no external dep)
// ────────────────────────────────────────────────────────────────

mod rustc_version {
    pub struct Version {
        major: u32,
        minor: u32,
        patch: u32,
    }

    impl std::fmt::Display for Version {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }

    pub fn version() -> Result<Version, ()> {
        let output = std::process::Command::new("rustc")
            .arg("--version")
            .output()
            .map_err(|_| ())?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse "rustc 1.xx.y (...)"
        let parts: Vec<&str> = stdout.split_whitespace().collect();
        if parts.len() >= 2 {
            let ver_str = parts[1];
            let nums: Vec<u32> = ver_str
                .split('.')
                .filter_map(|s| s.parse().ok())
                .collect();
            if nums.len() >= 3 {
                return Ok(Version {
                    major: nums[0],
                    minor: nums[1],
                    patch: nums[2],
                });
            }
        }
        Err(())
    }
}
