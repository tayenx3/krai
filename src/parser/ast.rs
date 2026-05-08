use std::sync::Arc;
use crate::operator::Operator;
use crate::span::Span;
use super::ty::ParseType;

#[derive(Debug, Clone, PartialEq)]
pub struct Ast(pub Arc<[Expr]>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Int(i64),
    Float(f64),
    String(String),
    Identifier(lasso::Spur),

    Semi(Box<Expr>),
    Block(Vec<Expr>),

    BinaryOp {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: Operator,
    },
    UnaryOp {
        operand: Box<Expr>,
        op: Operator,
    },

    Let {
        name: lasso::Spur,
        ty: Option<ParseType>,
        init: Option<Box<Expr>>,
    },
    Var {
        name: lasso::Spur,
        ty: Option<ParseType>,
        init: Option<Box<Expr>>,
    },

    FunctionDef {
        name: lasso::Spur,
        params: Vec<Param>,
        return_ty: Option<ParseType>,
        body: Box<Expr>,
        unwrap: bool,
    },
    FunctionDecl {
        name: lasso::Spur,
        params: Vec<Param>,
        return_ty: Option<ParseType>,
        unwrap: bool,
    },
    FunctionCall {
        callee: Box<Expr>,
        args: Vec<Expr>,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub mutability: bool,
    pub name: (lasso::Spur, Span),
    pub ty: ParseType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
    pub span: Span
}
