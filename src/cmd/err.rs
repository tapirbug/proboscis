use std::fmt::Display;

use crate::parse::ParserError;

pub type CommandResult<T> = Result<T, CommandError>;

/// Top level for presentation to users, without borrowed data.
pub struct CommandError {
    msg: String
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {}", &self.msg)
    }
}

impl<'s> From<ParserError<'s>> for CommandError {
    fn from(value: ParserError<'s>) -> Self {
        CommandError { msg: value.to_string() }
    }
}

impl<'s> From<std::io::Error> for CommandError {
    fn from(value: std::io::Error) -> Self {
        CommandError { msg: value.to_string() }
    }
}
