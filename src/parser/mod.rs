pub mod ast;
pub mod ty;

use ast::*;
use ty::*;
use crate::lexer::token::{Token, TokenKind};
use crate::diagnostic::Diagnostic;
use crate::operator::Operator;
use crate::span::Span;

pub struct Parser<'a> {
    pub rodeo: &'a lasso::Rodeo,
    pub path: &'a str,
    pub tokens: &'a [Token<'a>],
    pub pos: usize,
    pub no_color: bool,
    pub next_node_id: usize,
}

impl<'a> Parser<'a> {
    pub fn new(rodeo: &'a lasso::Rodeo, path: &'a str, tokens: &'a [Token<'a>], no_color: bool,) -> Self {
        Self {
            rodeo, path, tokens, pos: 0, no_color, next_node_id: 0
        }
    }

    fn create_node(&mut self, kind: ExprKind, span: Span) -> Expr {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        Expr { id, kind, span }
    }

    fn advance(&mut self) { self.pos += 1 }
    fn peek(&mut self) -> Option<&Token<'a>> { self.tokens.get(self.pos) }
    fn expect(&mut self, expected: TokenKind) -> Result<&Token<'a>, Diagnostic> {
        match self.tokens.get(self.pos) {
            Some(tok) if tok.kind == expected => {
                self.advance();
                Ok(tok)
            },
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
    fn expect_ident(&mut self) -> Result<(lasso::Spur, Span), Diagnostic> {
        match self.tokens.get(self.pos) {
            Some(tok) => if let TokenKind::Identifier(s) = tok.kind {
                self.advance();
                Ok((s, tok.span))
            } else {
                Err(Diagnostic {
                    path: self.path.to_string(),
                    msg: format!("expected identifier, found `{}`", tok.kind.format(self.rodeo)),
                    span: tok.span,
                    no_color: self.no_color
                })
            }
            None => Err(Diagnostic {
                path: self.path.to_string(),
                msg: format!("expected identifier, found end of input"),
                span: self.tokens.last().map(|t| t.span.splat()).unwrap_or(Span { start: 0, end: 1 }),
                no_color: self.no_color
            }),
        }
    }

    pub fn parse(&mut self) -> Result<Ast, Diagnostic> {
        let mut nodes = vec![];

        while self.peek().is_some() {
            nodes.push(self.parse_statement()?);
        }

        Ok(Ast(nodes.into()))
    }

    fn parse_statement(&mut self) -> Result<Expr, Diagnostic> {
        let mut node = self.parse_expression(0)?;

        if let Ok(&Token { span: semicolon_span, .. }) = self.expect(TokenKind::Semicolon) {
            let span = node.span + semicolon_span;
            node = self.create_node(ExprKind::Semi(Box::new(node)), span);
        }

        Ok(node)
    }

    fn parse_expression(&mut self, min_bp: usize) -> Result<Expr, Diagnostic> {
        let mut lhs = self.parse_primary()?;

        while let Some(tok) = self.peek() {
            match tok.kind {
                TokenKind::Operator(op) => {
                    let bp = {
                        if op.is_infix() {
                            let (lbp, rbp) = op.binding_power();
                            if lbp < min_bp { break }
                            rbp
                        } else { break }
                    };
                    self.advance();
                    let rhs = self.parse_expression(bp)?;
                    let span = lhs.span + rhs.span;
                    lhs = self.create_node(
                        ExprKind::BinaryOp {
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                            op
                        },
                        span
                    );
                },
                TokenKind::LParen => {
                    self.advance();
                    let mut args = vec![];
                    while let Some(tok) = self.peek() {
                        if tok.kind == TokenKind::RParen { break }

                        args.push(self.parse_expression(0)?);

                        if self.expect(TokenKind::Comma).is_err() { break }
                    }
                    let span = lhs.span + self.expect(TokenKind::RParen)?.span;
                    lhs = self.create_node(
                        ExprKind::FunctionCall { callee: Box::new(lhs), args },
                        span
                    );                 
                },
                _ => break
            }
        }

        Ok(lhs)
    }

    fn parse_primary(&mut self) -> Result<Expr, Diagnostic> {
        let tok = match self.tokens.get(self.pos) {
            Some(tok) => tok,
            None => return Err(Diagnostic {
                path: self.path.to_string(),
                msg: format!("expected expression, found end of input"),
                span: self.tokens.last().map(|t| t.span.splat()).unwrap_or(Span { start: 0, end: 1 }),
                no_color: self.no_color
            }),
        };

        match &tok.kind {
            TokenKind::Int(i) => {
                self.advance();
                Ok(self.create_node(
                    ExprKind::Int(*i),
                    tok.span
                ))
            },
            TokenKind::Float(i) => {
                self.advance();
                Ok(self.create_node(
                    ExprKind::Float(*i),
                    tok.span
                ))
            },
            TokenKind::String(i) => {
                self.advance();
                Ok(self.create_node(
                    ExprKind::String(i.to_string()),
                    tok.span
                ))
            },
            TokenKind::Identifier(i) => {
                self.advance();
                Ok(self.create_node(
                    ExprKind::Identifier(*i),
                    tok.span
                ))
            },
            TokenKind::Operator(op) if op.is_prefix() => {
                self.advance();
                let inner = self.parse_primary()?;
                let span = tok.span + inner.span;
                Ok(self.create_node(
                    ExprKind::UnaryOp { operand: Box::new(inner), op: *op },
                    span
                ))
            },
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression(0)?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            },
            TokenKind::LCurly => self.parse_block(),
            TokenKind::Let => self.parse_let(),
            TokenKind::Var => self.parse_var(),
            TokenKind::Dollar => self.parse_function(),
            _ => Err(Diagnostic {
                path: self.path.to_string(),
                msg: format!("expected expression, found `{}`", tok.kind.format(self.rodeo)),
                span: tok.span,
                no_color: self.no_color
            })
        }
    }

    fn parse_let(&mut self) -> Result<Expr, Diagnostic> {
        let mut span = self.expect(TokenKind::Let)?.span;
        let name = self.expect_ident()?.0;
        let ty = if self.peek()
            .map(|t| t.kind != TokenKind::Operator(Operator::Assign))
            .unwrap_or(false)
        {
            let ty = self.parse_type()?;
            span += ty.span;
            Some(ty)
        } else {
            None
        };
        let init = if self.expect(TokenKind::Operator(Operator::Assign)).is_ok() {
            let expr = self.parse_expression(0)?;
            span += expr.span;
            Some(Box::new(expr))
        } else {
            None
        };
        Ok(self.create_node(
            ExprKind::Let { name, ty, init },
            span
        ))
    }

    fn parse_var(&mut self) -> Result<Expr, Diagnostic> {
        let mut span = self.expect(TokenKind::Var)?.span;
        let name = self.expect_ident()?.0;
        let ty = if self.peek()
            .map(|t| t.kind != TokenKind::Operator(Operator::Assign))
            .unwrap_or(false)
        {
            let ty = self.parse_type()?;
            span += ty.span;
            Some(ty)
        } else {
            None
        };
        let init = if self.expect(TokenKind::Operator(Operator::Assign)).is_ok() {
            let expr = self.parse_expression(0)?;
            span += expr.span;
            Some(Box::new(expr))
        } else {
            None
        };
        Ok(self.create_node(
            ExprKind::Var { name, ty, init },
            span
        ))
    }

    fn parse_function(&mut self) -> Result<Expr, Diagnostic> {
        let mut span = self.expect(TokenKind::Dollar)?.span;
        let name = self.expect_ident()?.0;
        let unwrap = self.expect(TokenKind::Operator(Operator::Bang)).is_ok();

        self.expect(TokenKind::LParen)?;
        let mut params = vec![];
        while let Some(tok) = self.peek() {
            if tok.kind == TokenKind::RParen { break }

            let mutability = self.expect(TokenKind::Var).is_ok();
            let name = self.expect_ident()?;
            let ty = self.parse_type()?;
            params.push(Param { mutability, name, ty });

            if self.expect(TokenKind::Comma).is_err() { break }
        }
        self.expect(TokenKind::RParen)?;
        let return_ty = if matches!(self.peek(), Some(Token { kind: TokenKind::Arrow, .. }) | None) {
            None
        } else {
            let ty = self.parse_type()?;
            span += ty.span;
            Some(ty)
        };
        if self.expect(TokenKind::Arrow).is_ok() {
            let body = Box::new(self.parse_expression(0)?);
            span += body.span;

            Ok(self.create_node(
                ExprKind::FunctionDef { name, params, return_ty, body, unwrap },
                span
            ))
        } else {
            Ok(self.create_node(
                ExprKind::FunctionDecl { name, params, return_ty, unwrap },
                span
            ))
        }
    }

    fn parse_type(&mut self) -> Result<ParseType, Diagnostic> {
        let tok = match self.tokens.get(self.pos) {
            Some(tok) => tok,
            None => return Err(Diagnostic {
                path: self.path.to_string(),
                msg: format!("expected type, found end of input"),
                span: self.tokens.last().map(|t| t.span.splat()).unwrap_or(Span { start: 0, end: 1 }),
                no_color: self.no_color
            }),
        };

        match &tok.kind {
            TokenKind::Identifier(i) => {
                self.advance();
                Ok(ParseType {
                    kind: ParseTypeKind::Identifier(*i),
                    span: tok.span
                })
            },
            TokenKind::LParen => {
                self.advance();
                let inner = self.parse_type()?;
                self.expect(TokenKind::RParen)?;
                Ok(inner)
            },
            _ => Err(Diagnostic {
                path: self.path.to_string(),
                msg: format!("expected type, found `{}`", tok.kind.format(self.rodeo)),
                span: tok.span,
                no_color: self.no_color
            })
        }
    }

    fn parse_block(&mut self) -> Result<Expr, Diagnostic> {
        let mut span = self.expect(TokenKind::LCurly)?.span;
        let mut stmts = vec![];
        while let Some(tok) = self.peek() {
            if tok.kind == TokenKind::RCurly { break }
            stmts.push(self.parse_expression(0)?);
            if self.expect(TokenKind::Semicolon).is_err() { break }
        }
        span += self.expect(TokenKind::RCurly)?.span;
        Ok(self.create_node(
            ExprKind::Block(stmts),
            span
        ))    
    }
}
