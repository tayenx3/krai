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
