use std::fmt;

use crate::{diagnostic::Diagnostic, ir::{PlaceAddress, StaticFunctionAddress}};

pub type VariableScope<'s> = Scope<'s, PlaceAddress>;

pub type FunctionScope<'s> = Scope<'s, StaticFunctionAddress>;

pub struct Scope<'s, T> {
    /// Bindings, duplicates are allowed. To the right is more local,
    /// and resolving will give the most local.
    bindings: Vec<(&'s str, T)>,
    scope_ends: Vec<usize>,
}

impl<'s, T> Scope<'s, T> {
    pub fn new() -> Self {
        Self {
            bindings: vec![],
            scope_ends: vec![],
        }
    }

    pub fn enter_scope(&mut self) {
        self.scope_ends.push(self.bindings.len());
    }

    pub fn add_binding(&mut self, name: &'s str, address: T) {
        // test:
        //eprintln!("[{}] {} = {:?}", (self.scope_ends.len()), name, address);
        self.bindings.push((name, address));
    }

    pub fn exit_scope(&mut self) {
        let scope_end =
            self.scope_ends.pop().expect("exited more than entered");
        self.bindings.drain(scope_end..);
    }

    pub fn resolve(
        &self,
        name: &'s str,
    ) -> Result<T, NotInScope> where T: Clone {
        self.bindings
            .iter()
            .rev()
            .find(|(binding_name, _)| *binding_name == name)
            .map(|(_, addr)| addr.clone())
            .ok_or_else(|| NotInScope(name))
    }
}

#[derive(Debug)]
pub struct NotInScope<'s>(&'s str);

impl<'s> fmt::Display for NotInScope<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} not in scope", self.0)
    }
}

impl<'s> Diagnostic for NotInScope<'s> {
    fn kind(&self) -> crate::diagnostic::DiagnosticKind {
        crate::diagnostic::DiagnosticKind::Error
    }
}
