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
    attributes: Vec<FunctionAttribute>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FunctionAttribute {
    /// Creates space for persistent places used in the function and also for
    /// functions it creates that inherit the places, all in one go.
    CreatesPersistentPlaces,
    /// Persistent scope is taken over from the parameter.
    ///
    /// No new space is reserved for additional places, it is assumed that
    /// this was considered in the function that creates the places.
    AcceptsPersistentPlaces,
    /// The function is a public interface that can be called from the outside
    /// via JavaScript.
    Exported,
}

impl Function {
    pub fn new(
        name: String,
        instructions: Vec<Instruction>,
        attributes: Vec<FunctionAttribute>,
    ) -> Self {
        Function {
            name,
            instructions,
            attributes,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn export_name(&self) -> Option<&str> {
        (self.attributes.contains(&FunctionAttribute::Exported))
            .then(|| self.name.as_ref())
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn attributes(&self) -> &[FunctionAttribute] {
        &self.attributes
    }
}
