mod datatype;
mod form;
mod funcdef;
mod globaldef;
mod irgen;
mod semantic;
mod strings;

pub use funcdef::{FunctionDefinition, FunctionDefinitionError};
pub use globaldef::{GlobalDefinition, GlobalDefinitionError};
pub use irgen::{IrGen, IrGenError};
pub use semantic::SemanticAnalysis;
