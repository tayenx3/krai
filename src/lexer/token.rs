use crate::operator::Operator;
use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind<'a> {
    Int(i64),
    Float(f64),
    String(&'a str),

    Operator(Operator),

    Dollar,
    LParen, RParen,
    LCurly, RCurly,
    Arrow,
    Semicolon,

    Identifier(lasso::Spur),
    Let, Var,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub span: Span,
}
