pub mod token;

use token::{Token, TokenKind};
use crate::operator::Operator;
use crate::span::Span;
use crate::diagnostic::Diagnostic;

pub fn tokenize<'a>(path: &str, source: &'a str, no_color: bool, rodeo: &mut lasso::Rodeo) -> Result<Vec<Token<'a>>, Diagnostic> {
    let mut tokens = vec![];
    let mut source_chars = source.char_indices().peekable();

    while let Some((start, ch)) = source_chars.next() {
        match ch {
            ' ' | '\t' | '\r' | '\n' => continue,
            '"' => {
                let mut end = start + ch.len_utf8();
                let mut closed = false;
                while let Some((start, ch)) = source_chars.next() {
                    end = start + ch.len_utf8();
                    if ch == '"' {
                        closed = true;
                        break;
                    }
                }
                if !closed {
                    return Err(Diagnostic {
                        path: path.to_string(),
                        msg: format!("unclosed string literal"),
                        span: Span { start, end },
                        no_color
                    });
                }
                tokens.push(Token { kind: TokenKind::String(&source[(start + 1)..(end - 1)]), span: Span { start, end: start + ch.len_utf8() } });
            },
            '+' => tokens.push(Token { kind: TokenKind::Operator(Operator::Plus), span: Span { start, end: start + ch.len_utf8() } }),
            '-' => if let Some(&(start, '>')) = source_chars.peek() {
                tokens.push(Token { kind: TokenKind::Arrow, span: Span { start, end: start + ch.len_utf8() } });
                source_chars.next();
            } else {
                tokens.push(Token { kind: TokenKind::Operator(Operator::Minus), span: Span { start, end: start + ch.len_utf8() } });
            },
            '*' => tokens.push(Token { kind: TokenKind::Operator(Operator::Star), span: Span { start, end: start + ch.len_utf8() } }),
            '/' => tokens.push(Token { kind: TokenKind::Operator(Operator::Slash), span: Span { start, end: start + ch.len_utf8() } }),
            '%' => tokens.push(Token { kind: TokenKind::Operator(Operator::Modulo), span: Span { start, end: start + ch.len_utf8() } }),
            '=' => tokens.push(Token { kind: TokenKind::Operator(Operator::Assign), span: Span { start, end: start + ch.len_utf8() } }),
            '$' => tokens.push(Token { kind: TokenKind::Dollar, span: Span { start, end: start + ch.len_utf8() } }),
            '(' => tokens.push(Token { kind: TokenKind::LParen, span: Span { start, end: start + ch.len_utf8() } }),
            ')' => tokens.push(Token { kind: TokenKind::RParen, span: Span { start, end: start + ch.len_utf8() } }),
            '{' => tokens.push(Token { kind: TokenKind::LCurly, span: Span { start, end: start + ch.len_utf8() } }),
            '}' => tokens.push(Token { kind: TokenKind::RCurly, span: Span { start, end: start + ch.len_utf8() } }),
            ';' => tokens.push(Token { kind: TokenKind::Semicolon, span: Span { start, end: start + ch.len_utf8() } }),
            ch if ch.is_ascii_digit() => {
                let mut end = start;
                let mut last_offset = ch.len_utf8();
                let mut is_float = false;

                while let Some(&(start, ch)) = source_chars.peek() {
                    if ch.is_ascii_digit() || ch == '_' {
                        source_chars.next();
                    } else if ch == '.' && !is_float {
                        is_float = true;
                        source_chars.next();
                    } else {
                        break;
                    }

                    end = start;
                    last_offset = ch.len_utf8();
                }
                end += last_offset;

                if is_float {
                    tokens.push(Token {
                        kind: TokenKind::Float(source[start..end].parse().unwrap()),
                        span: Span { start, end: start + ch.len_utf8() }
                    });
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Int(source[start..end].parse().unwrap()),
                        span: Span { start, end: start + ch.len_utf8() }
                    });
                }
            },
            ch if ch.is_alphabetic() || ch == '_' => {
                let mut end = start;
                let mut last_offset = ch.len_utf8();

                while let Some(&(start, ch)) = source_chars.peek() {
                    if ch.is_alphanumeric() {
                        source_chars.next();
                    } else {
                        break;
                    }
                    
                    end = start;
                    last_offset = ch.len_utf8();
                }
                end += last_offset;

                tokens.push(Token {
                    kind: lookup_ident(&source[start..end], rodeo),
                    span: Span { start, end: start + ch.len_utf8() }
                });
            },
            _ => return Err(Diagnostic {
                path: path.to_string(),
                msg: format!("unrecognized char `{ch}`"),
                span: Span { start, end: start + ch.len_utf8() },
                no_color
            })
        }
    }

    Ok(tokens)
}

fn lookup_ident<'a>(ident: &'a str, rodeo: &mut lasso::Rodeo) -> TokenKind<'a> {
    match ident {
        "let" => TokenKind::Let,
        "var" => TokenKind::Var,
        _ => TokenKind::Identifier(rodeo.get_or_intern(ident)),
    }
}
