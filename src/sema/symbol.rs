use std::collections::{HashMap, HashSet};
use super::ty::TypeId;

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolMap {
    pub mutables: HashSet<lasso::Spur>,
    pub types: HashMap<lasso::Spur, TypeId>,
}

#[allow(unused)]
impl SymbolMap {
    pub fn new() -> Self {
        Self {
            mutables: HashSet::new(),
            types: HashMap::new()
        }
    }

    pub fn define_symbol(&mut self, name: lasso::Spur, mutability: bool, ty: TypeId) {
        if mutability {
            self.mutables.insert(name);
        }
        self.types.insert(name, ty);
    }

    pub fn is_mutable(&self, name: &lasso::Spur) -> bool {
        self.mutables.contains(name)
    }

    pub fn get_type(&self, name: &lasso::Spur) -> Option<&TypeId> {
        self.types.get(name)
    }

    pub fn iter_mutables(&self) -> std::collections::hash_set::Iter<'_, lasso::Spur> {
        self.mutables.iter()
    }

    pub fn iter_types(&self) -> std::collections::hash_map::Iter<'_, lasso::Spur, TypeId> {
        self.types.iter()
    }
}
