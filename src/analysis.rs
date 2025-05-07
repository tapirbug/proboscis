mod builtin;
mod checknames;
mod datatype;
mod expression;
mod funcdef;
mod globaldef;
mod form;
mod irgen;
mod place;
mod semantic;
mod strings;

pub use irgen::{IrGen, IrGenError};
pub use checknames::{NameCheck, NameError};
pub use funcdef::{FunctionDefinition, FunctionDefinitionError};
pub use globaldef::{GlobalDefinition, GlobalDefinitionError};
pub use semantic::{SemanticAnalysis, SemanticAnalysisError};
