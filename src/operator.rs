use crate::sema::ty::{Type, TypeId};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    Plus, Minus, Star, Slash, Modulo, Assign,
    Bang, Eq, Ne
}

impl Operator {
    pub fn binding_power(&self) -> (usize, usize) {
        match self {
            Self::Plus | Self::Minus => (40, 41),
            Self::Star | Self::Slash | Self::Modulo => (50, 51),
            Self::Assign => (10, 11),
            Self::Bang => (0, 0), // infix ops are handled separately
            Self::Eq | Self::Ne => (20, 21),
        }
    }

    pub fn is_prefix(&self) -> bool {
        [Self::Plus, Self::Minus, Self::Bang].contains(self)
    }

    pub fn is_infix(&self) -> bool {
        ![Self::Bang].contains(self)
    }

    pub fn infix_output_type(&self, lhs_id: &TypeId, rhs_id: &TypeId, types: &mut HashMap<TypeId, Type>) -> Option<TypeId> {
        let lhs = &types[lhs_id];
        let rhs = &types[rhs_id];
        match self {
            Self::Plus | Self::Minus | Self::Star
            | Self::Slash | Self::Modulo => if lhs.is_numeric() && rhs.is_numeric() {
                if ((!lhs.is_ambiguous() && !rhs.is_ambiguous())
                    || (lhs.is_ambiguous() && rhs.is_ambiguous()))
                    && lhs == rhs
                {
                    Some(*lhs_id)
                } else if lhs.is_ambiguous() && !rhs.is_ambiguous() && lhs.is_coerceable(rhs) {
                    *types.get_mut(lhs_id).unwrap() = rhs.clone();
                    Some(*lhs_id)
                } else if rhs.is_ambiguous() && !lhs.is_ambiguous() && rhs.is_coerceable(lhs) {
                    *types.get_mut(rhs_id).unwrap() = lhs.clone();
                    Some(*lhs_id)
                } else if lhs.is_ambiguous() && rhs.is_ambiguous() && lhs == rhs {
                    Some(*lhs_id)
                } else { None }
            } else { None },
            Self::Eq | Self::Ne => {
                let bool_id = *types.iter().find(|(_, ty)| **ty == Type::Bool).unwrap().0;
                if ((!lhs.is_ambiguous() && !rhs.is_ambiguous())
                    || (lhs.is_ambiguous() && rhs.is_ambiguous()))
                    && lhs == rhs
                {
                    Some(bool_id)
                } else if lhs.is_coerceable(rhs) {
                    *types.get_mut(lhs_id).unwrap() = rhs.clone();
                    Some(bool_id)
                } else if rhs.is_coerceable(lhs) {
                    *types.get_mut(rhs_id).unwrap() = lhs.clone();
                    Some(bool_id)
                } else { None }
            }
            _ => None,
        }
    }

    pub fn prefix_output_type(&self, id: &TypeId, types: &HashMap<TypeId, Type>) -> Option<TypeId> {
        let ty = &types[id];
        match self {
            Self::Plus => if ty.is_numeric() {
                Some(*id)
            } else { None },
            Self::Minus => if ty.is_numeric() && !ty.is_unsigned() {
                Some(*id)
            } else { None },
            Self::Bang => if ty.is_int() {
                Some(*id)
            } else { None },
            _ => None
        }
    }
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Star => write!(f, "*"),
            Self::Slash => write!(f, "/"),
            Self::Modulo => write!(f, "%"),
            Self::Assign => write!(f, "="),
            Self::Bang => write!(f, "!"),
            Self::Eq => write!(f, "=="),
            Self::Ne => write!(f, "!="),
        }
    }
}
