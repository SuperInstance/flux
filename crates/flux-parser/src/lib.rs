//! FLUX.MD parser — parses structured markdown into an AST and compiles to FIR.

pub mod ast;
pub mod compiler;
pub mod parser;

// Re-export key types
pub use ast::{AgentDirective, AstDocument, CodeBlock, DirectiveKind, Frontmatter, SourceSpan, TextSection};
pub use compiler::{AstCompiler, CompileError};
pub use parser::{FluxParser, ParseError};
