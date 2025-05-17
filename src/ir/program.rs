use super::{
    FunctionTableIndex, StaticFunctionAddress, databuilder::StaticData,
    func::Function,
};

pub struct Program {
    static_data: StaticData,
    functions: Vec<Function>,
}

impl Program {
    pub fn new(static_data: StaticData, functions: Vec<Function>) -> Self {
        Self {
            static_data,
            functions,
        }
    }

    pub fn static_data(&self) -> &StaticData {
        &self.static_data
    }

    pub fn functions(&self) -> &[Function] {
        &self.functions
    }

    pub fn resolve_function_addr(
        &self,
        addr: StaticFunctionAddress,
    ) -> &Function {
        &self.functions[addr.to_i32() as usize]
    }

    pub fn resolve_function_idx(&self, idx: FunctionTableIndex) -> &Function {
        &self.functions[self.static_data.table_entries()[idx.to_u32() as usize]
            .to_i32() as usize]
    }
}
