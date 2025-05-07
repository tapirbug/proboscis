use crate::ir::PlaceAddress;

pub struct VariableScope<'s> {
    /// Bindings, duplicates are allowed. To the right is more local,
    /// and resolving will give the most local.
    bindings: Vec<(&'s str, PlaceAddress)>,
    scope_ends: Vec<usize>
}

impl<'s> VariableScope<'s> {
    pub fn new() -> Self {
        Self {
            bindings: vec![],
            scope_ends: vec![]
        }
    }

    pub fn enter_scope(&mut self) {
        self.scope_ends.push(self.bindings.len());
    }

    pub fn add_binding(&mut self, name: &'s str, address: PlaceAddress) {
        self.bindings.push((name, address));
    }

    pub fn exit_scope(&mut self) {
        let scope_end = self.scope_ends.pop().expect("exited more than entered");
        self.bindings.drain(scope_end..);
    }

    pub fn resolve(&self, name: &'s str) -> Result<PlaceAddress, VariableNotInScope> {
        self.bindings.iter().rev().find(|(binding_name, _)| *binding_name == name).map(|(_, addr)| *addr)
        .ok_or_else(|| VariableNotInScope(name))
    }
}

pub struct VariableNotInScope<'s>(&'s str);
