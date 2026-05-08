use crate::sema::ty::{Type, TypeId};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    Plus, Minus, Star, Slash, Modulo, Assign,
    Bang,
}

impl Operator {
    pub fn binding_power(&self) -> (usize, usize) {
        match self {
            Self::Plus | Self::Minus => (20, 21),
            Self::Star | Self::Slash | Self::Modulo => (30, 31),
            Self::Assign => (10, 11),
            Self::Bang => (0, 0), // infix ops are handled separately
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
                if !lhs.is_ambiguous() && !rhs.is_ambiguous() && lhs == rhs {
                    Some(*lhs_id)
                } else if lhs.is_ambiguous() && !rhs.is_ambiguous() && lhs.is_coerceable(rhs) {
                    types.insert(*lhs_id, rhs.clone());
                    Some(*lhs_id)
                } else if rhs.is_ambiguous() && !lhs.is_ambiguous() && rhs.is_coerceable(lhs) {
                    types.insert(*rhs_id, lhs.clone());
                    Some(*lhs_id)
                } else if lhs.is_ambiguous() && rhs.is_ambiguous() && lhs == rhs {
                    Some(*lhs_id)
                } else { None }
            } else { None },
            _ => None,
        }
    }

    pub fn prefix_output_type(&self, id: &TypeId, types: &HashMap<TypeId, Type>) -> Option<TypeId> {
        let ty = &types[id];
        match self {
            Self::Bang => if ty.is_int() {
                Some(*id)
            } else { None }
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
        }
    }
}
