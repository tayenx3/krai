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

impl<'a> TokenKind<'a> {
    pub fn format(&self, rodeo: &lasso::Rodeo) -> String {
        match self {
            Self::Int(i) => format!("`{i}`"),
            Self::Float(i) => format!("`{i}`"),
            Self::String(i) => format!("`\"{i}\"`"),
            Self::Operator(i) => format!("`{i}`"),
            Self::Dollar => "`$`".to_string(),
            Self::LParen => "`(`".to_string(),
            Self::RParen => "`)`".to_string(),
            Self::LCurly => "`{`".to_string(),
            Self::RCurly => "`}`".to_string(),
            Self::Arrow => format!("`->`"),
            Self::Semicolon => format!("`;`"),
            Self::Identifier(i) => format!("`{}`", rodeo.resolve(i)),
            Self::Let => "`let`".to_string(),
            Self::Var => "`var`".to_string()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub span: Span,
}
