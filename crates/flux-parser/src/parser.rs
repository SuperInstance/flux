use std::collections::HashMap;

use crate::ast::{
    AgentDirective, AstDocument, CodeBlock, DirectiveKind, Frontmatter, SourceSpan, TextSection,
};

/// Parse errors for the FLUX.MD parser.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("parse error at line {line}, column {column}: {message}")]
    AtLocation {
        line: usize,
        column: usize,
        message: String,
    },
    #[error("invalid frontmatter: {0}")]
    InvalidFrontmatter(String),
    #[error("unclosed code block at line {0}")]
    UnclosedCodeBlock(usize),
    #[error("unexpected token: {0}")]
    UnexpectedToken(String),
}

/// Hand-written recursive descent parser for FLUX.MD format.
pub struct FluxParser;

impl FluxParser {
    /// Parse a FLUX.MD document from the input string.
    pub fn parse(input: &str) -> Result<AstDocument, ParseError> {
        ParserState::new(input).parse()
    }
}

/// Internal parser state.
struct ParserState<'a> {
    input: &'a str,
    chars: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> ParserState<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.pos..]
    }

    #[allow(dead_code)]
    fn is_eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    #[allow(dead_code)]
    fn current_span(&self) -> SourceSpan {
        SourceSpan {
            start: self.pos,
            end: self.pos,
            line: self.line,
            column: self.column,
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.pos += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    #[allow(dead_code)]
    fn peek_ahead(&self, n: usize) -> Option<char> {
        self.chars.get(self.pos + n).copied()
    }

    fn starts_with(&self, s: &str) -> bool {
        self.remaining().starts_with(s)
    }

    fn consume(&mut self, s: &str) -> bool {
        if self.starts_with(s) {
            for _ in s.chars() {
                self.advance();
            }
            true
        } else {
            false
        }
    }

    fn consume_until(&mut self, predicate: impl Fn(char) -> bool) -> String {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if predicate(ch) {
                break;
            }
            self.advance();
        }
        self.input[start..self.pos].to_string()
    }

    #[allow(dead_code)]
    fn consume_line(&mut self) -> String {
        let result = self.consume_until(|c| c == '\n');
        self.advance(); // consume the '\n'
        result
    }

    #[allow(dead_code)]
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    #[allow(dead_code)]
    fn skip_blank_lines(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                self.advance();
            } else if ch == ' ' || ch == '\t' {
                // peek ahead to see if it's a blank line (only whitespace until newline)
                let saved_pos = self.pos;
                let saved_line = self.line;
                let saved_col = self.column;
                self.skip_whitespace();
                if self.peek() == Some('\n') {
                    self.advance();
                    continue;
                }
                // restore
                self.pos = saved_pos;
                self.line = saved_line;
                self.column = saved_col;
                break;
            } else {
                break;
            }
        }
    }

    fn parse(&mut self) -> Result<AstDocument, ParseError> {
        let mut doc = AstDocument {
            frontmatter: None,
            code_blocks: Vec::new(),
            text_sections: Vec::new(),
            agent_directives: Vec::new(),
        };

        // Try to parse frontmatter
        if self.starts_with("---\n") {
            self.consume("---\n");
            let start = self.pos;
            let start_line = self.line;
            let start_col = self.column;
            let content = self.consume_until(|_c| false); // consume everything
            // find the closing ---
            let close_idx = content.find("\n---");
            let (fm_content, rest) = if let Some(idx) = close_idx {
                let fm = &content[..idx];
                let rest_start = idx + 4; // len("\n---")
                (fm.to_string(), content[rest_start..].to_string())
            } else {
                (content.clone(), String::new())
            };

            // Rewind to after the frontmatter content
            // We consumed too much. Let's rewind properly.
            // Actually, let me re-approach this.
            // Reset: we consumed everything, so put pos back to start + fm_content.len() + 4
            // and advance by those characters.

            // Re-do this more carefully
            self.pos = start + fm_content.len() + 4; // skip past \n---
            // Recalculate line/column from the consumed frontmatter
            for ch in &self.chars[start..self.pos] {
                if *ch == '\n' {
                    self.line += 1;
                    self.column = 1;
                } else {
                    self.column += 1;
                }
            }

            doc.frontmatter = Some(Self::parse_frontmatter(&fm_content, SourceSpan {
                start,
                end: self.pos,
                line: start_line,
                column: start_col,
            })?);

            // Parse any remaining content after frontmatter
            self.parse_body(&mut doc, &rest)?;
        } else {
            let content = self.input.to_string();
            self.pos = self.chars.len();
            self.parse_body(&mut doc, &content)?;
        }

        Ok(doc)
    }

    fn parse_body(&mut self, doc: &mut AstDocument, content: &str) -> Result<(), ParseError> {
        // Reset position to parse the body content
        let body_start = self.pos;
        let body_chars: Vec<char> = content.chars().collect();
        let mut bpos: usize = 0;
        let mut bline = self.line;
        let mut bcol = self.column;
        let mut current_text = String::new();
        let mut text_start = SourceSpan {
            start: self.pos,
            end: self.pos,
            line: self.line,
            column: self.column,
        };

        while bpos < body_chars.len() {
            // Check for code block opening
            if bpos + 2 < body_chars.len()
                && body_chars[bpos] == '`'
                && body_chars[bpos + 1] == '`'
                && body_chars[bpos + 2] == '`'
            {
                // Flush text section
                let trimmed = current_text.trim().to_string();
                if !trimmed.is_empty() {
                    doc.text_sections.push(TextSection {
                        content: trimmed,
                        span: text_start,
                    });
                }
                current_text = String::new();

                // Parse code block
                let code_start = body_start + bpos;
                let code_line = bline;
                let code_col = bcol;

                bpos += 3;
                for _ in 0..3 {
                    if bcol > 1 { bcol -= 1; } // rough tracking
                }

                // Parse language tag
                let _lang_start = bpos;
                let lang = {
                    let mut lang_str = String::new();
                    while bpos < body_chars.len() && body_chars[bpos] != '\n' {
                        lang_str.push(body_chars[bpos]);
                        if body_chars[bpos] == '\n' {
                            bline += 1;
                            bcol = 1;
                        } else {
                            bcol += 1;
                        }
                        bpos += 1;
                    }
                    lang_str.trim().to_string()
                };

                // Skip the newline
                if bpos < body_chars.len() && body_chars[bpos] == '\n' {
                    bpos += 1;
                    bline += 1;
                    bcol = 1;
                }

                // Read code content until closing ```
                let mut source = String::new();
                let mut closed = false;
                while bpos + 2 < body_chars.len() {
                    if body_chars[bpos] == '`'
                        && body_chars[bpos + 1] == '`'
                        && body_chars[bpos + 2] == '`'
                    {
                        bpos += 3;
                        bcol += 3;
                        closed = true;
                        break;
                    }
                    if body_chars[bpos] == '\n' {
                        bline += 1;
                        bcol = 1;
                    } else {
                        bcol += 1;
                    }
                    source.push(body_chars[bpos]);
                    bpos += 1;
                }

                // Check for name after closing ```
                let name = if closed {
                    let mut name_str = String::new();
                    while bpos < body_chars.len() && body_chars[bpos] != '\n' {
                        name_str.push(body_chars[bpos]);
                        bpos += 1;
                        bcol += 1;
                    }
                    let n = name_str.trim().to_string();
                    if n.is_empty() { None } else { Some(n) }
                } else {
                    return Err(ParseError::UnclosedCodeBlock(code_line));
                };

                doc.code_blocks.push(CodeBlock {
                    language: lang,
                    source: source.trim_end().to_string(),
                    span: SourceSpan {
                        start: code_start,
                        end: body_start + bpos,
                        line: code_line,
                        column: code_col,
                    },
                    name,
                });

                text_start = SourceSpan {
                    start: body_start + bpos,
                    end: body_start + bpos,
                    line: bline,
                    column: bcol,
                };
                continue;
            }

            // Check for agent directive: @send, @ask, @tell, etc.
            if body_chars[bpos] == '@' && bpos + 1 < body_chars.len() {
                // Flush text
                let trimmed = current_text.trim().to_string();
                if !trimmed.is_empty() {
                    doc.text_sections.push(TextSection {
                        content: trimmed,
                        span: text_start,
                    });
                }
                current_text = String::new();

                let dir_start = body_start + bpos;
                let dir_line = bline;
                let dir_col = bcol;

                bpos += 1; // skip @
                bcol += 1;

                // Parse directive kind
                let mut kind_str = String::new();
                while bpos < body_chars.len()
                    && (body_chars[bpos].is_alphanumeric() || body_chars[bpos] == '_')
                {
                    kind_str.push(body_chars[bpos]);
                    bpos += 1;
                    bcol += 1;
                }

                // Skip whitespace
                while bpos < body_chars.len() && (body_chars[bpos] == ' ' || body_chars[bpos] == '\t') {
                    bpos += 1;
                    bcol += 1;
                }

                // Parse target
                let mut target = String::new();
                while bpos < body_chars.len() && body_chars[bpos] != '\n' && body_chars[bpos] != ' ' && body_chars[bpos] != ':' {
                    target.push(body_chars[bpos]);
                    bpos += 1;
                    bcol += 1;
                }

                // Skip whitespace after target
                while bpos < body_chars.len() && (body_chars[bpos] == ' ' || body_chars[bpos] == '\t') {
                    bpos += 1;
                    bcol += 1;
                }

                // Parse payload (rest of line after :, if present)
                let payload = if bpos < body_chars.len() && body_chars[bpos] == ':' {
                    bpos += 1; // skip :
                    bcol += 1;
                    // Skip whitespace
                    while bpos < body_chars.len() && (body_chars[bpos] == ' ' || body_chars[bpos] == '\t') {
                        bpos += 1;
                        bcol += 1;
                    }
                    let mut payload_str = String::new();
                    while bpos < body_chars.len() && body_chars[bpos] != '\n' {
                        payload_str.push(body_chars[bpos]);
                        bpos += 1;
                        bcol += 1;
                    }
                    let p = payload_str.trim().to_string();
                    if p.is_empty() { None } else { Some(p) }
                } else {
                    None
                };

                // Skip newline
                if bpos < body_chars.len() && body_chars[bpos] == '\n' {
                    bpos += 1;
                    bline += 1;
                    bcol = 1;
                }

                if let Some(kind) = DirectiveKind::from_str(&kind_str) {
                    doc.agent_directives.push(AgentDirective {
                        kind,
                        target,
                        payload,
                        span: SourceSpan {
                            start: dir_start,
                            end: body_start + bpos,
                            line: dir_line,
                            column: dir_col,
                        },
                    });
                } else {
                    // Unknown directive, treat as text
                    current_text = format!("@{}", kind_str);
                }

                text_start = SourceSpan {
                    start: body_start + bpos,
                    end: body_start + bpos,
                    line: bline,
                    column: bcol,
                };
                continue;
            }

            // Regular character
            current_text.push(body_chars[bpos]);
            if body_chars[bpos] == '\n' {
                bline += 1;
                bcol = 1;
            } else {
                bcol += 1;
            }
            bpos += 1;
        }

        // Flush remaining text
        let trimmed = current_text.trim().to_string();
        if !trimmed.is_empty() {
            doc.text_sections.push(TextSection {
                content: trimmed,
                span: text_start,
            });
        }

        // Advance the main parser position
        self.pos = body_start + bpos;

        Ok(())
    }

    fn parse_frontmatter(content: &str, _span: SourceSpan) -> Result<Frontmatter, ParseError> {
        let mut fm = Frontmatter {
            title: None,
            version: None,
            language: None,
            imports: Vec::new(),
            metadata: HashMap::new(),
        };

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "title" => fm.title = Some(value.to_string()),
                    "version" => fm.version = Some(value.to_string()),
                    "language" => fm.language = Some(value.to_string()),
                    "import" | "imports" => {
                        // Can be comma-separated
                        for imp in value.split(',') {
                            let imp = imp.trim();
                            if !imp.is_empty() {
                                fm.imports.push(imp.to_string());
                            }
                        }
                    }
                    _ => {
                        fm.metadata.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        Ok(fm)
    }
}
