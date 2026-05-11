use std::collections::HashMap;
use crate::span::Span;
use crate::diagnostic::Diagnostic;
use super::symbol::FuncId;
use cranelift_codegen::ir::{Type as ClifType, types};
use target_lexicon::{PointerWidth, Triple};

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
    Bool,
    Unknown,
    Function(bool, Vec<TypeId>, TypeId, FuncId)
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

    pub fn is_signed(&self) -> bool {
        matches!(self, Self::I8 | Self::I16 | Self::I32 | Self::I64 | Self::Isz | Self::AmbiguousInt)
    }

    pub fn is_unsigned(&self) -> bool {
        matches!(self, Self::U8 | Self::U16 | Self::U32 | Self::U64 | Self::Usz)
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Self::F32 | Self::F64 | Self::AmbiguousFloat)
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
            Self::Bool => "bool".to_string(),
            Self::Unknown => "<unknown>".to_string(),
            Self::Function(unwrap, params, ret, _) => {
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

    pub fn into_clif_type(&self, target: &Triple, path: &str, span: Span, no_color: bool) -> Result<ClifType, Diagnostic> {
        Ok(match self {
            Self::I8 | Self::U8 | Self::Unit | Self::Bool => types::I8,
            Self::I16 | Self::U16 => types::I16,
            Self::I32 | Self::U32 | Self::AmbiguousInt => types::I32,
            Self::I64 | Self::U64 => types::I64,
            Self::Isz | Self::Usz => match target.pointer_width().unwrap() {
                PointerWidth::U16 => types::I16,
                PointerWidth::U32 => types::I32,
                PointerWidth::U64 => types::I64,
            },
            Self::F32 => types::F32,
            Self::F64 | Self::AmbiguousFloat => types::F64,
            Self::Unknown => return Err(Diagnostic {
                path: path.to_string(),
                msg: "cannot infer type".to_string(),
                span, no_color
            }),
            Self::Function(..) => match target.pointer_width().unwrap() {
                PointerWidth::U16 => types::I16,
                PointerWidth::U32 => types::I32,
                PointerWidth::U64 => types::I64,
            },
        })
    }

    pub fn size(&self, target: &Triple, path: &str, span: Span, no_color: bool) -> Result<u32, Diagnostic> {
        Ok(match self {
            Self::I8 | Self::U8 | Self::Unit | Self::Bool => 1,
            Self::I16 | Self::U16 => 2,
            Self::I32 | Self::U32 | Self::AmbiguousInt | Self::F32 => 4,
            Self::I64 | Self::U64 | Self::F64 | Self::AmbiguousFloat => 8,
            Self::Isz | Self::Usz | Self::Function(..) => match target.pointer_width().unwrap() {
                PointerWidth::U16 => 2,
                PointerWidth::U32 => 4,
                PointerWidth::U64 => 8,
            },
            Self::Unknown => return Err(Diagnostic {
                path: path.to_string(),
                msg: "cannot infer type".to_string(),
                span, no_color
            }),
        })
    }

    pub fn align(&self, target: &Triple, path: &str, span: Span, no_color: bool) -> Result<u8, Diagnostic> {
        // struct/arrays will need different alignments, we'll add them later
        self.size(target, path, span, no_color).map(|x| x as u8)
    }
}
