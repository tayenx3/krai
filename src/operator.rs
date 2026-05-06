#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    Plus, Minus, Star, Slash, Modulo, Assign,
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
        }
    }
}
