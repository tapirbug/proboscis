mod checknames;
mod datatype;
mod expression;
mod funcdef;
mod globaldef;
mod irgen;
mod strings;
mod place;
mod semantic;

pub use irgen::{IrGen, IrGenError};
pub use checknames::{NameCheck, NameError};
pub use funcdef::{FunctionDefinition, FunctionDefinitionError};
pub use globaldef::{GlobalDefinition, GlobalDefinitionError};
pub use semantic::{SemanticAnalysis, SemanticAnalysisError};
