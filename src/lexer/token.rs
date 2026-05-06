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
    Comma,

    Identifier(lasso::Spur),
    Let, Var,
}

impl<'a> TokenKind<'a> {
    pub fn format(&self, rodeo: &lasso::Rodeo) -> String {
        match self {
            Self::Int(i) => i.to_string(),
            Self::Float(i) => i.to_string(),
            Self::String(i) => format!("\"{i}\""),
            Self::Operator(i) => i.to_string(),
            Self::Dollar => "$".to_string(),
            Self::LParen => "(".to_string(),
            Self::RParen => ")".to_string(),
            Self::LCurly => "{".to_string(),
            Self::RCurly => "}".to_string(),
            Self::Arrow => "->".to_string(),
            Self::Semicolon => ";".to_string(),
            Self::Comma => ",".to_string(),
            Self::Identifier(i) => rodeo.resolve(i).to_string(),
            Self::Let => "let".to_string(),
            Self::Var => "var".to_string()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub span: Span,
}
