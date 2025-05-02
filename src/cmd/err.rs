use std::fmt::Display;

use crate::{
    analysis::{FunctionDefinitionError, GlobalDefinitionError, IrGenError, NameError, SemanticAnalysisError},
    parse::ParserError,
};

pub type CommandResult<T> = Result<T, CommandError>;

/// Top level for presentation to users, without borrowed data.
pub struct CommandError {
    msg: String,
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {}", &self.msg)
    }
}

impl<'s> From<ParserError<'s>> for CommandError {
    fn from(value: ParserError<'s>) -> Self {
        CommandError {
            msg: value.to_string(),
        }
    }
}

impl<'t, 's> From<FunctionDefinitionError<'t, 's>> for CommandError {
    fn from(value: FunctionDefinitionError<'t, 's>) -> Self {
        CommandError {
            msg: value.to_string(),
        }
    }
}

impl<'t, 's> From<GlobalDefinitionError<'t, 's>> for CommandError {
    fn from(value: GlobalDefinitionError<'t, 's>) -> Self {
        CommandError {
            msg: value.to_string(),
        }
    }
}

impl<'s> From<std::io::Error> for CommandError {
    fn from(value: std::io::Error) -> Self {
        CommandError {
            msg: value.to_string(),
        }
    }
}

impl<'s, 't> From<SemanticAnalysisError<'s, 't>> for CommandError {
    fn from(value: SemanticAnalysisError<'s, 't>) -> Self {
        CommandError { msg: value.to_string() }
    }
}

impl<'t, 's> From<NameError<'t, 's>> for CommandError {
    fn from(value: NameError<'t, 's>) -> Self {
        CommandError {
            msg: value.to_string(),
        }
    }
}

impl From<IrGenError> for CommandError {
    fn from(value: IrGenError) -> Self {
        match value {}
    }
}
