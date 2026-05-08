use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub usize);

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    I8, I16, I32, I64, Isz,
    U8, U16, U32, U64, Usz,
    F32, F64,
    Unit,
    AmbiguousInt, AmbiguousFloat,
    Unknown,
    Function(bool, Vec<TypeId>, TypeId)
}

impl Type {
    pub fn is_numeric(&self) -> bool {
        matches!(self,
            Self::I8 | Self::I16 | Self::I32 | Self::I64 | Self::Isz
            | Self::U8 | Self::U16 | Self::U32 | Self::U64 | Self::Usz
            | Self::F32 | Self::F64
            | Self::AmbiguousInt | Self::AmbiguousFloat
        )
    }

    pub fn is_int(&self) -> bool {
        matches!(self,
            Self::I8 | Self::I16 | Self::I32 | Self::I64 | Self::Isz
                | Self::U8 | Self::U16 | Self::U32 | Self::U64 | Self::Usz
                | Self::AmbiguousInt
        )
    }

    pub fn is_coerceable(&self, other: &Type) -> bool {
        matches!((self, other),
            (Self::AmbiguousInt, Self::I8 | Self::I16 | Self::I32 | Self::I64 | Self::Isz
            | Self::U8 | Self::U16 | Self::U32 | Self::U64 | Self::Usz
            | Self::AmbiguousInt) |
            (Self::AmbiguousFloat, Self::F32 | Self::F64 | Self::AmbiguousFloat)
        ) || (*self == Self::Unknown)
    }

    pub fn is_ambiguous(&self) -> bool {
        matches!(self, Self::AmbiguousInt | Self::AmbiguousFloat | Self::Unknown)
    }

    pub fn debug(&self, types: &HashMap<TypeId, Type>) -> String {
        match self {
            Self::I8 => "i8".to_string(),
            Self::I16 => "i16".to_string(),
            Self::I32 => "i32".to_string(),
            Self::I64 => "i64".to_string(),
            Self::Isz => "isz".to_string(),
            Self::U8 => "u8".to_string(),
            Self::U16 => "u16".to_string(),
            Self::U32 => "u32".to_string(),
            Self::U64 => "u64".to_string(),
            Self::Usz => "usz".to_string(),
            Self::F32 => "f32".to_string(),
            Self::F64 => "f64".to_string(),
            Self::Unit => "()".to_string(),
            Self::AmbiguousInt => "<int>".to_string(),
            Self::AmbiguousFloat => "<float>".to_string(),
            Self::Unknown => "<unknown>".to_string(),
            Self::Function(unwrap, params, ret) => {
                let mut params_fmt = String::new();
                for (idx, param) in params.iter().enumerate() {
                    if idx > 0 {
                        params_fmt.push_str(", ");
                    }
                    params_fmt.push_str(&types[param].debug(types));
                }
                format!("${}({params_fmt}) {}", if *unwrap { "!" } else { "" }, types[ret].debug(types))
            },
        }
    }
}
