use std::mem;

use super::{
    FunctionAttribute,
    func::{Function, StaticFunctionAddress},
    inst::InstructionBuilder,
};

pub struct FunctionsBuilder {
    exported_names: Vec<String>,
    function_builders: Vec<InstructionBuilder>,
    attributes: Vec<Vec<FunctionAttribute>>,
}

impl FunctionsBuilder {
    pub fn new() -> Self {
        FunctionsBuilder {
            exported_names: vec![],
            function_builders: vec![],
            attributes: vec![],
        }
    }

    pub fn add_exported_function(
        &mut self,
        name: &str,
    ) -> StaticFunctionAddress {
        self.add_function(name, vec![FunctionAttribute::Exported])
    }

    pub fn add_private_function(
        &mut self,
        name: &str,
    ) -> StaticFunctionAddress {
        self.add_function(name, vec![])
    }

    pub fn implement_function(
        &mut self,
        address: StaticFunctionAddress,
    ) -> &mut InstructionBuilder {
        &mut self.function_builders[address.to_i32() as usize]
    }

    pub fn add_attribute(
        &mut self,
        address: StaticFunctionAddress,
        attribute: FunctionAttribute,
    ) -> &mut Self {
        self.attributes[address.to_i32() as usize].push(attribute);
        self
    }

    fn add_function(
        &mut self,
        name: &str,
        initial_attributes: Vec<FunctionAttribute>,
    ) -> StaticFunctionAddress {
        let new_idx = self.function_builders.len();
        let function_address =
            StaticFunctionAddress::new_unsafe(new_idx as i32);
        self.exported_names.push(name.into());
        self.function_builders.push(InstructionBuilder::new());
        self.attributes.push(initial_attributes);
        function_address
    }

    /// Builds the functions, leaving the builder empty.
    pub fn build(&mut self) -> Vec<Function> {
        mem::take(&mut self.exported_names)
            .into_iter()
            .zip(mem::take(&mut self.function_builders).into_iter())
            .zip(mem::take(&mut self.attributes).into_iter())
            .map(|((name, mut builder), attributes)| {
                Function::new(name, builder.build(), attributes)
            })
            .collect()
    }
}
