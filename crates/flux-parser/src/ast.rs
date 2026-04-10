use std::collections::HashMap;

/// A parsed FLUX.MD document.
#[derive(Debug, Clone)]
pub struct AstDocument {
    pub frontmatter: Option<Frontmatter>,
    pub code_blocks: Vec<CodeBlock>,
    pub text_sections: Vec<TextSection>,
    pub agent_directives: Vec<AgentDirective>,
}

/// YAML-like frontmatter at the start of a FLUX.MD document.
#[derive(Debug, Clone)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub version: Option<String>,
    pub language: Option<String>,
    pub imports: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// A fenced code block within the document.
#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub language: String,
    pub source: String,
    pub span: SourceSpan,
    pub name: Option<String>,
}

/// A plain text section.
#[derive(Debug, Clone)]
pub struct TextSection {
    pub content: String,
    pub span: SourceSpan,
}

/// An agent directive (e.g., `@send`, `@ask`).
#[derive(Debug, Clone)]
pub struct AgentDirective {
    pub kind: DirectiveKind,
    pub target: String,
    pub payload: Option<String>,
    pub span: SourceSpan,
}

/// The kind of agent directive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectiveKind {
    Send,
    Ask,
    Tell,
    Delegate,
    Subscribe,
    Trust,
}

impl DirectiveKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "send" => Some(DirectiveKind::Send),
            "ask" => Some(DirectiveKind::Ask),
            "tell" => Some(DirectiveKind::Tell),
            "delegate" => Some(DirectiveKind::Delegate),
            "subscribe" => Some(DirectiveKind::Subscribe),
            "trust" => Some(DirectiveKind::Trust),
            _ => None,
        }
    }
}

/// A location in the source text.
#[derive(Debug, Clone, Copy)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}
