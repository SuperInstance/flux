use flux_parser::*;

// ── Basic parsing tests ─────────────────────────────────────

#[test]
fn test_empty_document() {
    let doc = FluxParser::parse("").unwrap();
    assert!(doc.frontmatter.is_none());
    assert!(doc.code_blocks.is_empty());
    assert!(doc.text_sections.is_empty());
    assert!(doc.agent_directives.is_empty());
}

#[test]
fn test_plain_text() {
    let doc = FluxParser::parse("Hello world").unwrap();
    assert_eq!(doc.text_sections.len(), 1);
    assert_eq!(doc.text_sections[0].content, "Hello world");
}

#[test]
fn test_frontmatter_only() {
    let input = "---\ntitle: My Module\nversion: 0.1.0\n---\n";
    let doc = FluxParser::parse(input).unwrap();
    let fm = doc.frontmatter.as_ref().unwrap();
    assert_eq!(fm.title.as_deref(), Some("My Module"));
    assert_eq!(fm.version.as_deref(), Some("0.1.0"));
}

#[test]
fn test_frontmatter_with_imports() {
    let input = "---\nimport: std.io, std.fs\n---\n";
    let doc = FluxParser::parse(input).unwrap();
    let fm = doc.frontmatter.as_ref().unwrap();
    assert_eq!(fm.imports, vec!["std.io", "std.fs"]);
}

#[test]
fn test_frontmatter_with_metadata() {
    let input = "---\ntitle: Test\nauthor: flux\ndescription: A test module\n---\n";
    let doc = FluxParser::parse(input).unwrap();
    let fm = doc.frontmatter.as_ref().unwrap();
    assert_eq!(fm.metadata.get("author").unwrap(), "flux");
    assert_eq!(fm.metadata.get("description").unwrap(), "A test module");
}

#[test]
fn test_code_block() {
    let input = "```c\nint main() { return 0; }\n```\n";
    let doc = FluxParser::parse(input).unwrap();
    assert_eq!(doc.code_blocks.len(), 1);
    assert_eq!(doc.code_blocks[0].language, "c");
    assert_eq!(doc.code_blocks[0].source, "int main() { return 0; }");
}

#[test]
fn test_code_block_with_name() {
    let input = "```c my_func\nint foo() { return 1; }\n``` name=foo\n";
    let doc = FluxParser::parse(input).unwrap();
    assert_eq!(doc.code_blocks.len(), 1);
    assert_eq!(doc.code_blocks[0].name.as_deref(), Some("name=foo"));
}

#[test]
fn test_multiple_code_blocks() {
    let input = "```flux\nfn a() {}\n```\n```c\nint b() {}\n```\n";
    let doc = FluxParser::parse(input).unwrap();
    assert_eq!(doc.code_blocks.len(), 2);
    assert_eq!(doc.code_blocks[0].language, "flux");
    assert_eq!(doc.code_blocks[1].language, "c");
}

#[test]
fn test_agent_directive_send() {
    let input = "@send agent_a : hello world\n";
    let doc = FluxParser::parse(input).unwrap();
    assert_eq!(doc.agent_directives.len(), 1);
    let d = &doc.agent_directives[0];
    assert_eq!(d.kind, DirectiveKind::Send);
    assert_eq!(d.target, "agent_a");
    assert_eq!(d.payload.as_deref(), Some("hello world"));
}

#[test]
fn test_agent_directive_ask() {
    let input = "@ask oracle : what is 42?\n";
    let doc = FluxParser::parse(input).unwrap();
    assert_eq!(doc.agent_directives.len(), 1);
    assert_eq!(doc.agent_directives[0].kind, DirectiveKind::Ask);
    assert_eq!(doc.agent_directives[0].target, "oracle");
}

#[test]
fn test_agent_directive_tell() {
    let input = "@tell worker : process this\n";
    let doc = FluxParser::parse(input).unwrap();
    assert_eq!(doc.agent_directives[0].kind, DirectiveKind::Tell);
}

#[test]
fn test_agent_directive_delegate() {
    let input = "@delegate helper : compute(42)\n";
    let doc = FluxParser::parse(input).unwrap();
    assert_eq!(doc.agent_directives[0].kind, DirectiveKind::Delegate);
}

#[test]
fn test_agent_directive_trust() {
    let input = "@trust cert_authority\n";
    let doc = FluxParser::parse(input).unwrap();
    assert_eq!(doc.agent_directives[0].kind, DirectiveKind::Trust);
    assert!(doc.agent_directives[0].payload.is_none());
}

#[test]
fn test_full_document() {
    let input = r#"---
title: Test Module
version: 1.0
---

# Introduction

This is a test module.

```c
int add(int a, int b) {
    return a + b;
}
```

@send processor : start

```flux
fn main() { return 0; }
```
"#;
    let doc = FluxParser::parse(input).unwrap();
    assert!(doc.frontmatter.is_some());
    assert_eq!(doc.code_blocks.len(), 2);
    assert!(!doc.text_sections.is_empty());
    assert_eq!(doc.agent_directives.len(), 1);
}

// ── Error handling tests ────────────────────────────────────

#[test]
fn test_unclosed_code_block_error() {
    let input = "```c\nint main() { return 0; }\n";
    let result = FluxParser::parse(input);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("unclosed"));
}

// ── SourceSpan tests ────────────────────────────────────────

#[test]
fn test_source_span_display() {
    let span = SourceSpan {
        start: 0,
        end: 10,
        line: 5,
        column: 3,
    };
    assert_eq!(format!("{span}"), "line 5, column 3");
}

// ── Compiler tests ──────────────────────────────────────────

#[test]
fn test_compile_empty_document() {
    let doc = FluxParser::parse("").unwrap();
    let mut compiler = AstCompiler::new();
    let module = compiler.compile(&doc).unwrap();
    assert!(module.functions.is_empty());
}

#[test]
fn test_compile_simple_function() {
    let input = "```c\nint add(int a, int b) { return a + b; }\n```\n";
    let doc = FluxParser::parse(input).unwrap();
    let mut compiler = AstCompiler::new();
    let module = compiler.compile(&doc).unwrap();
    assert!(module.functions.contains_key("add"));
    let func = &module.functions["add"];
    assert_eq!(func.params.len(), 2);
}

#[test]
fn test_compile_void_function() {
    let input = "```c\nvoid noop() { return; }\n```\n";
    let doc = FluxParser::parse(input).unwrap();
    let mut compiler = AstCompiler::new();
    let module = compiler.compile(&doc).unwrap();
    assert!(module.functions.contains_key("noop"));
}

#[test]
fn test_compile_with_var_decl() {
    let input = "```c\nint test() { int x = 42; return x; }\n```\n";
    let doc = FluxParser::parse(input).unwrap();
    let mut compiler = AstCompiler::new();
    let module = compiler.compile(&doc).unwrap();
    assert!(module.functions.contains_key("test"));
}

#[test]
fn test_compile_if_statement() {
    let input = "```c\nint test(int x) { if (x > 0) { return 1; } else { return 0; } return 0; }\n```\n";
    let doc = FluxParser::parse(input).unwrap();
    let mut compiler = AstCompiler::new();
    let module = compiler.compile(&doc).unwrap();
    assert!(module.functions.contains_key("test"));
}

#[test]
fn test_compile_with_arithmetic() {
    let input = "```c\nint calc(int a, int b) { int c = a * b + 10; return c - a; }\n```\n";
    let doc = FluxParser::parse(input).unwrap();
    let mut compiler = AstCompiler::new();
    let module = compiler.compile(&doc).unwrap();
    assert!(module.functions.contains_key("calc"));
}

#[test]
fn test_non_c_blocks_ignored() {
    let input = "```python\nprint('hello')\n```\n";
    let doc = FluxParser::parse(input).unwrap();
    let mut compiler = AstCompiler::new();
    let module = compiler.compile(&doc).unwrap();
    assert!(module.functions.is_empty());
}

#[test]
fn test_compile_multiple_functions() {
    let input = "```c\nint foo() { return 1; }\nint bar() { return 2; }\n```\n";
    let doc = FluxParser::parse(input).unwrap();
    let mut compiler = AstCompiler::new();
    let module = compiler.compile(&doc).unwrap();
    assert_eq!(module.functions.len(), 2);
    assert!(module.functions.contains_key("foo"));
    assert!(module.functions.contains_key("bar"));
}

#[test]
fn test_compile_with_comparison() {
    let input = "```c\nint cmp(int a, int b) { if (a == b) { return 1; } return 0; }\n```\n";
    let doc = FluxParser::parse(input).unwrap();
    let mut compiler = AstCompiler::new();
    let module = compiler.compile(&doc).unwrap();
    assert!(module.functions.contains_key("cmp"));
}

#[test]
fn test_compile_default() {
    let compiler = AstCompiler::default();
    let doc = AstDocument {
        frontmatter: None,
        code_blocks: vec![],
        text_sections: vec![],
        agent_directives: vec![],
    };
    // Can't compile with default because `compile` takes &mut self
    let mut compiler = compiler;
    let module = compiler.compile(&doc).unwrap();
    assert_eq!(module.name, "flux_module");
}

#[test]
fn test_compile_module_name_from_frontmatter() {
    let input = "---\ntitle: my_crate\n---\n```c\nint main() { return 0; }\n```\n";
    let doc = FluxParser::parse(input).unwrap();
    let mut compiler = AstCompiler::new();
    let module = compiler.compile(&doc).unwrap();
    assert_eq!(module.name, "my_crate");
}
