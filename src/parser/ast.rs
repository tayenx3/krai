use std::sync::Arc;
use crate::operator::Operator;

#[derive(Debug, Clone, PartialEq)]
pub struct Ast<'a>(pub Arc<[Expr<'a>]>);

#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'a> {
    Int(i64),
    Float(f64),
    String(&'a str),
    Identifier(lasso::Spur),

    BinaryOp {
        lhs: Box<Expr<'a>>,
        rhs: Box<Expr<'a>>,
        op: Operator,
    },
}
