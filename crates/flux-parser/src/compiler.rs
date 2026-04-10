use std::collections::HashMap;

use flux_fir::blocks::FirModule;
use flux_fir::builder::FirBuilder;
use flux_fir::instructions::CmpOp;
use flux_fir::types::{FirType, TypeContext};
use flux_fir::values::Value;

use crate::ast::{AstDocument, CodeBlock};

/// Errors from compiling an AST to FIR.
#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("compilation error at line {line}, column {column}: {message}")]
    AtLocation {
        line: usize,
        column: usize,
        message: String,
    },
    #[error("unsupported construct: {0}")]
    Unsupported(String),
    #[error("undefined variable: {0}")]
    UndefinedVariable(String),
    #[error("type error: {0}")]
    TypeError(String),
    #[error("syntax error in code block: {0}")]
    SyntaxError(String),
}

/// Compiles a FLUX.MD AST into a FIR module.
pub struct AstCompiler {
    type_ctx: TypeContext,
}

/// A simple token for the C-like subset compiler.
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    IntLit(i64),
    FloatLit(f64),
    StrLit(String),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semi,
    Comma,
    Equals,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    AmpAmp,
    PipePipe,
    Bang,
    IntKw,
    FloatKw,
    VoidKw,
    BoolKw,
    ReturnKw,
    IfKw,
    ElseKw,
    TrueKw,
    FalseKw,
}

struct Tokenizer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Tokenizer {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            self.pos += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Token>, CompileError> {
        let mut tokens = Vec::new();
        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' | '\n' | '\r' => {
                    self.advance();
                }
                '/' if self.chars.get(self.pos + 1) == Some(&'/') => {
                    // Line comment
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                '/' if self.chars.get(self.pos + 1) == Some(&'*') => {
                    // Block comment
                    self.advance();
                    self.advance();
                    loop {
                        match self.peek() {
                            None => break,
                            Some('*') => {
                                self.advance();
                                if self.peek() == Some('/') {
                                    self.advance();
                                    break;
                                }
                            }
                            _ => {
                                self.advance();
                            }
                        }
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut ident = String::new();
                    while let Some(c) = self.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            ident.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    let tok = match ident.as_str() {
                        "int" => Token::IntKw,
                        "float" => Token::FloatKw,
                        "void" => Token::VoidKw,
                        "bool" => Token::BoolKw,
                        "return" => Token::ReturnKw,
                        "if" => Token::IfKw,
                        "else" => Token::ElseKw,
                        "true" => Token::TrueKw,
                        "false" => Token::FalseKw,
                        _ => Token::Ident(ident),
                    };
                    tokens.push(tok);
                }
                '0'..='9' => {
                    let mut num = String::new();
                    let mut is_float = false;
                    while let Some(c) = self.peek() {
                        if c.is_ascii_digit() {
                            num.push(c);
                            self.advance();
                        } else if c == '.' && !is_float {
                            is_float = true;
                            num.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    if is_float {
                        if let Ok(f) = num.parse::<f64>() {
                            tokens.push(Token::FloatLit(f));
                        } else {
                            return Err(CompileError::SyntaxError(format!(
                                "invalid float literal '{}' at line {}",
                                num, self.line
                            )));
                        }
                    } else if let Ok(i) = num.parse::<i64>() {
                        tokens.push(Token::IntLit(i));
                    } else {
                        return Err(CompileError::SyntaxError(format!(
                            "invalid integer literal '{}' at line {}",
                            num, self.line
                        )));
                    }
                }
                '(' => { self.advance(); tokens.push(Token::LParen); }
                ')' => { self.advance(); tokens.push(Token::RParen); }
                '{' => { self.advance(); tokens.push(Token::LBrace); }
                '}' => { self.advance(); tokens.push(Token::RBrace); }
                ';' => { self.advance(); tokens.push(Token::Semi); }
                ',' => { self.advance(); tokens.push(Token::Comma); }
                '+' => { self.advance(); tokens.push(Token::Plus); }
                '-' => { self.advance(); tokens.push(Token::Minus); }
                '*' => { self.advance(); tokens.push(Token::Star); }
                '%' => { self.advance(); tokens.push(Token::Percent); }
                '&' if self.chars.get(self.pos + 1) == Some(&'&') => {
                    self.advance(); self.advance();
                    tokens.push(Token::AmpAmp);
                }
                '|' if self.chars.get(self.pos + 1) == Some(&'|') => {
                    self.advance(); self.advance();
                    tokens.push(Token::PipePipe);
                }
                '!' if self.chars.get(self.pos + 1) == Some(&'=') => {
                    self.advance(); self.advance();
                    tokens.push(Token::Ne);
                }
                '!' => {
                    self.advance();
                    tokens.push(Token::Bang);
                }
                '<' if self.chars.get(self.pos + 1) == Some(&'=') => {
                    self.advance(); self.advance();
                    tokens.push(Token::Le);
                }
                '<' => { self.advance(); tokens.push(Token::Lt); }
                '>' if self.chars.get(self.pos + 1) == Some(&'=') => {
                    self.advance(); self.advance();
                    tokens.push(Token::Ge);
                }
                '>' => { self.advance(); tokens.push(Token::Gt); }
                '=' if self.chars.get(self.pos + 1) == Some(&'=') => {
                    self.advance(); self.advance();
                    tokens.push(Token::Eq);
                }
                '=' => { self.advance(); tokens.push(Token::Equals); }
                '/' => { self.advance(); tokens.push(Token::Slash); }
                '"' => {
                    self.advance(); // skip opening quote
                    let mut s = String::new();
                    while let Some(c) = self.peek() {
                        if c == '"' {
                            self.advance();
                            break;
                        }
                        if c == '\\' {
                            self.advance();
                            if let Some(escaped) = self.advance() {
                                match escaped {
                                    'n' => s.push('\n'),
                                    't' => s.push('\t'),
                                    '\\' => s.push('\\'),
                                    '"' => s.push('"'),
                                    _ => {
                                        s.push('\\');
                                        s.push(escaped);
                                    }
                                }
                            }
                        } else {
                            s.push(c);
                            self.advance();
                        }
                    }
                    tokens.push(Token::StrLit(s));
                }
                _ => {
                    let line = self.line;
                    let col = self.col;
                    self.advance();
                    return Err(CompileError::SyntaxError(format!(
                        "unexpected character '{}' at line {}, column {}",
                        ch, line, col
                    )));
                }
            }
        }
        Ok(tokens)
    }
}

impl AstCompiler {
    pub fn new() -> Self {
        Self {
            type_ctx: TypeContext::new(),
        }
    }

    pub fn compile(&mut self, doc: &AstDocument) -> Result<FirModule, CompileError> {
        let mut builder = FirBuilder::new(
            doc.frontmatter
                .as_ref()
                .and_then(|fm| fm.title.as_deref())
                .unwrap_or("flux_module"),
        );

        for block in &doc.code_blocks {
            if block.language == "c" || block.language == "flux" {
                self.compile_code_block(block, &mut builder)?;
            }
        }

        Ok(builder.into_module())
    }

    fn compile_code_block(
        &mut self,
        block: &CodeBlock,
        builder: &mut FirBuilder,
    ) -> Result<(), CompileError> {
        let tokens = Tokenizer::new(&block.source).tokenize()?;
        let mut parser = CParser {
            tokens,
            pos: 0,
            builder,
            _type_ctx: &self.type_ctx,
            variables: HashMap::new(),
            line: block.span.line,
            current_function: String::new(),
        };
        parser.parse_program()
    }
}

struct CParser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    builder: &'a mut FirBuilder,
    _type_ctx: &'a TypeContext,
    variables: HashMap<String, Value>,
    line: usize,
    current_function: String,
}

impl<'a> CParser<'a> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn expect(&mut self, expected: &str) -> Result<Token, CompileError> {
        match self.advance() {
            Some(tok) => {
                let got = match &tok {
                    Token::LParen => "(",
                    Token::RParen => ")",
                    Token::LBrace => "{",
                    Token::RBrace => "}",
                    Token::Semi => ";",
                    Token::Comma => ",",
                    Token::Equals => "=",
                    Token::Plus => "+",
                    Token::Minus => "-",
                    Token::Star => "*",
                    Token::Slash => "/",
                    Token::Percent => "%",
                    Token::Lt => "<",
                    Token::Le => "<=",
                    Token::Gt => ">",
                    Token::Ge => ">=",
                    Token::Eq => "==",
                    Token::Ne => "!=",
                    Token::AmpAmp => "&&",
                    Token::PipePipe => "||",
                    Token::Bang => "!",
                    Token::IntKw => "int",
                    Token::FloatKw => "float",
                    Token::VoidKw => "void",
                    Token::BoolKw => "bool",
                    Token::ReturnKw => "return",
                    Token::IfKw => "if",
                    Token::ElseKw => "else",
                    Token::TrueKw => "true",
                    Token::FalseKw => "false",
                    Token::Ident(s) => s.as_str(),
                    Token::IntLit(_) => "integer literal",
                    Token::FloatLit(_) => "float literal",
                    Token::StrLit(_) => "string literal",
                };
                if got == expected {
                    Ok(tok)
                } else {
                    Err(CompileError::SyntaxError(format!(
                        "expected '{}', got '{}' at line {}",
                        expected, got, self.line
                    )))
                }
            }
            None => Err(CompileError::SyntaxError(format!(
                "expected '{}', got end of input at line {}",
                expected, self.line
            ))),
        }
    }

    fn parse_type(&mut self) -> Result<FirType, CompileError> {
        match self.advance() {
            Some(Token::IntKw) => Ok(FirType::Int(32)),
            Some(Token::FloatKw) => Ok(FirType::Float(64)),
            Some(Token::VoidKw) => Ok(FirType::Void),
            Some(Token::BoolKw) => Ok(FirType::Bool),
            other => Err(CompileError::TypeError(format!(
                "expected type, got {:?} at line {}",
                other, self.line
            ))),
        }
    }

    fn parse_program(&mut self) -> Result<(), CompileError> {
        while self.peek().is_some() {
            self.parse_top_level()?;
        }
        Ok(())
    }

    fn parse_top_level(&mut self) -> Result<(), CompileError> {
        let saved_pos = self.pos;
        let ty = self.parse_type();
        let ty = match ty {
            Ok(t) => t,
            Err(_) => {
                self.pos = saved_pos;
                self.advance();
                return Ok(());
            }
        };

        match self.peek() {
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                match self.peek() {
                    Some(Token::LParen) => {
                        self.parse_function(name, ty)?;
                    }
                    Some(Token::Equals) | Some(Token::Semi) => {
                        if self.peek() == Some(&Token::Equals) {
                            self.advance();
                            self.parse_expr()?;
                        }
                        if self.peek() == Some(&Token::Semi) {
                            self.advance();
                        }
                    }
                    _ => {
                        return Err(CompileError::SyntaxError(format!(
                            "expected '(' or '=' after variable name at line {}",
                            self.line
                        )));
                    }
                }
            }
            other => {
                return Err(CompileError::SyntaxError(format!(
                    "expected identifier after type, got {:?} at line {}",
                    other, self.line
                )));
            }
        }
        Ok(())
    }

    fn parse_function(&mut self, name: String, ret_ty: FirType) -> Result<(), CompileError> {
        self.expect("(")?;
        let mut params = Vec::new();
        while self.peek() != Some(&Token::RParen) {
            if !params.is_empty() {
                self.expect(",")?;
            }
            let param_ty = self.parse_type()?;
            let param_name = match self.advance() {
                Some(Token::Ident(n)) => n,
                other => {
                    return Err(CompileError::SyntaxError(format!(
                        "expected parameter name, got {:?} at line {}",
                        other, self.line
                    )));
                }
            };
            params.push((param_name, param_ty));
        }
        self.expect(")")?;
        self.expect("{")?;

        self.current_function = name.clone();
        self.builder.create_function(&name, params, ret_ty);
        // Register function parameters as variables
        let func = self.builder.module().functions[&name].clone();
        for param in &func.params {
            self.variables.insert(param.name.clone(), param.clone());
        }

        self.parse_block_body()?;

        self.expect("}")?;
        self.current_function = String::new();
        Ok(())
    }

    fn parse_block_body(&mut self) -> Result<(), CompileError> {
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            self.parse_statement()?;
        }
        Ok(())
    }

    fn parse_statement(&mut self) -> Result<(), CompileError> {
        match self.peek() {
            Some(Token::ReturnKw) => {
                self.advance();
                if self.peek() == Some(&Token::Semi) {
                    self.advance();
                    self.builder.ret(None);
                } else {
                    let val = self.parse_expr()?;
                    self.builder.ret(Some(val));
                    self.expect(";")?;
                }
            }
            Some(Token::IfKw) => {
                self.parse_if()?;
            }
            Some(Token::LBrace) => {
                self.advance();
                self.parse_block_body()?;
                self.expect("}")?;
            }
            Some(Token::IntKw) | Some(Token::FloatKw) | Some(Token::BoolKw) => {
                self.parse_var_decl()?;
            }
            _ => {
                let _val = self.parse_expr()?;
                self.expect(";")?;
            }
        }
        Ok(())
    }

    fn parse_var_decl(&mut self) -> Result<(), CompileError> {
        let ty = self.parse_type()?;
        let name = match self.advance() {
            Some(Token::Ident(n)) => n,
            other => {
                return Err(CompileError::SyntaxError(format!(
                    "expected variable name, got {:?} at line {}",
                    other, self.line
                )));
            }
        };
        let mut init_val = None;
        if self.peek() == Some(&Token::Equals) {
            self.advance();
            init_val = Some(self.parse_expr()?);
        }
        self.expect(";")?;

        let val = match init_val {
            Some(v) => v,
            None => match &ty {
                FirType::Int(_) => self.builder.const_int(0, ty.clone()),
                FirType::Float(_) => self.builder.const_float(0.0),
                FirType::Bool => self.builder.const_bool(false),
                _ => self.builder.const_int(0, FirType::Int(32)),
            },
        };
        self.variables.insert(name, val);
        Ok(())
    }

    fn parse_if(&mut self) -> Result<(), CompileError> {
        self.expect("if")?;
        self.expect("(")?;
        let cond = self.parse_expr()?;
        self.expect(")")?;

        let func_name = self.current_function.clone();
        let then_bb = self.builder.create_block(&func_name, "if_then");
        let else_bb = self.builder.create_block(&func_name, "if_else");
        let merge_bb = self.builder.create_block(&func_name, "if_merge");

        self.builder.branch(cond, then_bb, else_bb);

        // Then block
        self.builder.position_at_end(&func_name, then_bb);
        self.expect("{")?;
        self.parse_block_body()?;
        self.expect("}")?;
        self.builder.jump(merge_bb);

        // Else block
        self.builder.position_at_end(&func_name, else_bb);
        if self.peek() == Some(&Token::ElseKw) {
            self.advance();
            self.expect("{")?;
            self.parse_block_body()?;
            self.expect("}")?;
        }
        self.builder.jump(merge_bb);

        // Continue in merge block
        self.builder.position_at_end(&func_name, merge_bb);

        Ok(())
    }

    fn parse_expr(&mut self) -> Result<Value, CompileError> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Value, CompileError> {
        let mut left = self.parse_additive()?;

        loop {
            match self.peek() {
                Some(Token::Eq) => {
                    self.advance();
                    let right = self.parse_additive()?;
                    left = self.builder.icmp(CmpOp::Eq, left, right);
                }
                Some(Token::Ne) => {
                    self.advance();
                    let right = self.parse_additive()?;
                    left = self.builder.icmp(CmpOp::Ne, left, right);
                }
                Some(Token::Lt) => {
                    self.advance();
                    let right = self.parse_additive()?;
                    left = self.builder.icmp(CmpOp::Lt, left, right);
                }
                Some(Token::Le) => {
                    self.advance();
                    let right = self.parse_additive()?;
                    left = self.builder.icmp(CmpOp::Le, left, right);
                }
                Some(Token::Gt) => {
                    self.advance();
                    let right = self.parse_additive()?;
                    left = self.builder.icmp(CmpOp::Gt, left, right);
                }
                Some(Token::Ge) => {
                    self.advance();
                    let right = self.parse_additive()?;
                    left = self.builder.icmp(CmpOp::Ge, left, right);
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Value, CompileError> {
        let mut left = self.parse_multiplicative()?;

        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.advance();
                    let right = self.parse_multiplicative()?;
                    left = self.builder.iadd(left, right);
                }
                Some(Token::Minus) => {
                    self.advance();
                    let right = self.parse_multiplicative()?;
                    left = self.builder.isub(left, right);
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Value, CompileError> {
        let mut left = self.parse_unary()?;

        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.advance();
                    let right = self.parse_unary()?;
                    left = self.builder.imul(left, right);
                }
                Some(Token::Slash) => {
                    self.advance();
                    let right = self.parse_unary()?;
                    left = self.builder.idiv(left, right);
                }
                Some(Token::Percent) => {
                    self.advance();
                    let right = self.parse_unary()?;
                    left = self.builder.imod(left, right);
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Value, CompileError> {
        match self.peek() {
            Some(Token::Minus) => {
                self.advance();
                let operand = self.parse_unary()?;
                let zero = self.builder.const_int(0, operand.ty.clone());
                Ok(self.builder.isub(zero, operand))
            }
            Some(Token::Bang) => {
                self.advance();
                let operand = self.parse_unary()?;
                let false_val = self.builder.const_bool(false);
                Ok(self.builder.icmp(CmpOp::Eq, operand, false_val))
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Value, CompileError> {
        match self.peek() {
            Some(Token::IntLit(n)) => {
                let n = *n;
                self.advance();
                Ok(self.builder.const_int(n, FirType::Int(32)))
            }
            Some(Token::FloatLit(f)) => {
                let f = *f;
                self.advance();
                Ok(self.builder.const_float(f))
            }
            Some(Token::TrueKw) => {
                self.advance();
                Ok(self.builder.const_bool(true))
            }
            Some(Token::FalseKw) => {
                self.advance();
                Ok(self.builder.const_bool(false))
            }
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();

                if self.peek() == Some(&Token::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    while self.peek() != Some(&Token::RParen) {
                        if !args.is_empty() {
                            self.expect(",")?;
                        }
                        args.push(self.parse_expr()?);
                    }
                    self.expect(")")?;
                    let result = self.builder.call(&name, args, Some(FirType::Int(32)));
                    result.ok_or_else(|| {
                        CompileError::SyntaxError(format!(
                            "function call should return a value at line {}",
                            self.line
                        ))
                    })
                } else {
                    self.variables.get(&name).cloned().ok_or_else(|| {
                        CompileError::UndefinedVariable(name.clone())
                    })
                }
            }
            Some(Token::LParen) => {
                self.advance();
                let val = self.parse_expr()?;
                self.expect(")")?;
                Ok(val)
            }
            other => Err(CompileError::SyntaxError(format!(
                "unexpected token {:?} at line {}",
                other, self.line
            ))),
        }
    }
}

impl Default for AstCompiler {
    fn default() -> Self {
        Self::new()
    }
}
