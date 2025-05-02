use std::mem;

use super::{
    func::{Function, StaticFunctionAddress},
    inst::InstructionBuilder,
};

pub struct FunctionsBuilder {
    exported_names: Vec<String>,
    function_builders: Vec<InstructionBuilder>,
}

impl FunctionsBuilder {
    pub fn new() -> Self {
        FunctionsBuilder {
            exported_names: vec![],
            function_builders: vec![],
        }
    }

    pub fn add_exported_function(
        &mut self,
        name: &str,
    ) -> StaticFunctionAddress {
        self.add_function(name)
    }

    pub fn add_private_function(
        &mut self,
    ) -> StaticFunctionAddress {
        self.add_function("")
    }

    pub fn implement_private_function(&mut self, address: StaticFunctionAddress) -> &mut InstructionBuilder {
        &mut self.function_builders[address.to_i32() as usize]
    }

    fn add_function(
        &mut self,
        name: &str,
    ) -> StaticFunctionAddress {
        let new_idx = self.function_builders.len();
        let function_address =
            StaticFunctionAddress::new_unsafe(new_idx as i32);
        self.exported_names.push(name.into());
        self.function_builders.push(InstructionBuilder::new());
        function_address
    }

    /// Builds the function, leaving the builder empty.
    pub fn build(&mut self) -> Vec<Function> {
        mem::take(&mut self.exported_names)
            .into_iter()
            .zip(mem::take(&mut self.function_builders).into_iter())
            .map(|(name, mut builder)| Function::new(name, builder.build()))
            .collect()
    }
}
