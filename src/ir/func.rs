use super::inst::Instruction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StaticFunctionAddress(i32);

impl StaticFunctionAddress {
    pub fn new_unsafe(from: i32) -> Self {
        Self(from)
    }

    pub fn to_i32(self) -> i32 {
        self.0
    }
}

pub struct Function {
    /// The name under which the function is exported, or empty string if
    /// not exported.
    name: String,
    instructions: Vec<Instruction>,
}

impl Function {
    pub fn new(name: String, instructions: Vec<Instruction>) -> Self {
        Function { name, instructions }
    }
}
