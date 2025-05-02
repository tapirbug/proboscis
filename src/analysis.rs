mod checknames;
mod datatype;
mod expression;
mod funcdef;
mod globaldef;
mod irgen;
mod multitable;
mod place;
mod semantic;
mod strings;

pub use irgen::{IrGen, IrGenError};
pub use checknames::{NameCheck, NameError};
pub use funcdef::{FunctionDefinition, FunctionDefinitionError};
pub use globaldef::{GlobalDefinition, GlobalDefinitionError};
pub use multitable::MultiStringTable;
pub use semantic::{SemanticAnalysis, SemanticAnalysisError};
pub use strings::StringTable;
