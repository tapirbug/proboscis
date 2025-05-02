use std::collections::HashMap;

use crate::{analysis::FunctionDefinition, ir::{DataAddress, FunctionsBuilder, Program, StaticDataBuilder, StaticFunctionAddress}};

use super::SemanticAnalysis;


pub struct IrGen {}

impl IrGen {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate<'a: 't, 't, 's>(analysis: &'a SemanticAnalysis<'t, 's>) -> Result<Program, IrGenError> {
        let source = analysis.source();
        let mut static_data = StaticDataBuilder::new();
        let mut functions = FunctionsBuilder::new();
        let nil_address = static_data.static_nil();
        let nil_place = static_data.static_place(nil_address);
        let mut added_static_strings = HashMap::<String, DataAddress>::new();
        let mut function_addresses: HashMap<String, StaticFunctionAddress> = HashMap::<String, StaticFunctionAddress>::new();
        let mut function_definitions: HashMap<StaticFunctionAddress, &'a FunctionDefinition<'s, 't>> = HashMap::new();
        for entry in analysis.strings().entries() {
            let string: &str = entry.data().as_ref();
            if !added_static_strings.contains_key(string) {
                let address = static_data.static_string(string);
                added_static_strings.insert(string.into(), address);
            }
        }

        let mut function_builders = HashMap::new();
        for function in analysis.function_definitions() {
            let name = function.name().source_range().of(source).source();
            let address = functions.add_exported_function(name);
            function_addresses.insert(name.into(), address);
            function_builders.insert(address, function);
        }

        // TODO compile code here

        Ok(Program::new(static_data.build(), functions.build()))
    }

}

pub enum IrGenError {

}
