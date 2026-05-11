use std::collections::{HashMap, HashSet};
use super::ty::TypeId;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SymbolInitState {
    Definitely, Not
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MapScope {
    Root,
    Function(FuncId)
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionData {
    pub param_tys: Vec<TypeId>,
    pub ret_ty: TypeId,
    pub fty: TypeId,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct FuncId(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolMap {
    pub scope: MapScope,
    pub mutables: HashSet<lasso::Spur>,
    pub types: HashMap<lasso::Spur, TypeId>,
    pub init_states: HashMap<lasso::Spur, SymbolInitState>,
}

impl SymbolMap {
    pub fn new(scope: MapScope) -> Self {
        Self {
            scope,
            mutables: HashSet::new(),
            types: HashMap::new(),
            init_states: HashMap::new(),
        }
    }

    pub fn define_symbol(&mut self, name: lasso::Spur, mutability: bool, ty: TypeId, init_states: SymbolInitState) {
        if mutability {
            self.mutables.insert(name);
        }
        self.types.insert(name, ty);
        self.init_states.insert(name, init_states);
    }

    pub fn get_type(&self, name: &lasso::Spur) -> Option<&TypeId> {
        self.types.get(name)
    }

    pub fn iter_types(&self) -> std::collections::hash_map::Iter<'_, lasso::Spur, TypeId> {
        self.types.iter()
    }
}
