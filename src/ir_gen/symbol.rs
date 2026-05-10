use cranelift_codegen::ir::{StackSlot, Value};
use cranelift_module::FuncId as IrFuncId;
use crate::sema::ty::TypeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    SS(StackSlot),
    Func(IrFuncId),
    Arg(Value),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol {
    pub kind: SymbolKind,
    pub ty: TypeId,
}
