pub mod ast;

use ast::*;
use crate::lexer::token::{Token, TokenKind};
use crate::diagnostic::Diagnostic;
use crate::span::Span;

pub struct Parser<'a> {
    pub rodeo: &'a lasso::Rodeo,
    pub path: &'a str,
    pub tokens: &'a [Token<'a>],
    pub pos: usize,
    pub no_color: bool,
}

impl<'a> Parser<'a> {
    pub fn new(rodeo: &'a lasso::Rodeo, path: &'a str, tokens: &'a [Token<'a>], no_color: bool,) -> Self {
        Self {
            rodeo, path, tokens, pos: 0, no_color
        }
    }

    fn advance(&mut self) { self.pos += 1 }
    fn peek(&mut self) -> Option<&Token<'a>> { self.tokens.get(self.pos) }
    fn expect(&mut self, expected: TokenKind) -> Result<&Token<'a>, Diagnostic> {
        match self.tokens.get(self.pos) {
            Some(tok) if tok.kind == expected => Ok(tok),
            Some(other) => Err(Diagnostic {
                path: self.path.to_string(),
                msg: format!("expected `{}`, found `{}`", expected.format(self.rodeo), other.kind.format(self.rodeo)),
                span: other.span,
                no_color: self.no_color
            }),
            None => Err(Diagnostic {
                path: self.path.to_string(),
                msg: format!("expected `{}`, found end of input", expected.format(self.rodeo)),
                span: self.tokens.last().map(|t| t.span.splat()).unwrap_or(Span { start: 0, end: 1 }),
                no_color: self.no_color
            }),
        }
    }

    pub fn parse(&mut self) -> Result<Ast<'_>, Diagnostic> {
        let mut nodes = vec![];

        Ok(Ast(nodes.into()))
    }
}
