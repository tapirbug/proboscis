use std::fmt;

use crate::parse::{AstNode};

use crate::source::Source;

use super::{FunctionDefinition, FunctionDefinitionError, GlobalDefinition, GlobalDefinitionError, NameCheck, NameError, StringTable};

pub struct SemanticAnalysis<'s, 't> {
    source: Source<'s>,
    root_code: Vec<&'t AstNode<'s>>,
    function_definitions: Vec<FunctionDefinition<'t, 's>>,
    global_definitions: Vec<GlobalDefinition<'t, 's>>,
    strings: StringTable<'s>
}

impl<'t, 's> SemanticAnalysis<'t, 's> {
    /// Semantically analyses a single source file and returns the result of
    /// analysis if it appears to be valid.
    pub fn analyze(source: Source<'s>, ast: &'t [AstNode<'s>]) -> Result<SemanticAnalysis<'s, 't>, SemanticAnalysisError<'s, 't>> {
        let strings = StringTable::analyze(source, &ast);
        // TODO find constant numbers too

        let mut root_code = vec![];
        let mut function_definitions = vec![];
        let mut global_definitions = vec![];
        for root_node in ast {
            // try parsing root-level element as a function first
            let def = FunctionDefinition::extract(source, root_node)?;
            match def {
                Some(def) => {
                    function_definitions.push(def);
                    continue;
                }
                None => {}
            }

            // then as a global
            let def = GlobalDefinition::extract(source, root_node)?;
            match def {
                Some(def) => {
                    global_definitions.push(def);
                    continue;
                }
                None => {}
            }

            // all other cases are considered to be top-level code
            root_code.push(root_node);
        }

        Ok(SemanticAnalysis { source, root_code, function_definitions, global_definitions, strings })
    }
}

impl<'t, 's> SemanticAnalysis<'t, 's> {
    pub fn source(&self) -> Source<'s> {
        self.source
    }

    pub fn root_code(&self) -> &[&'t AstNode<'s>] {
        &self.root_code
    }

    pub fn function_definitions(&self) -> &[FunctionDefinition<'t, 's>] {
        &self.function_definitions
    }

    pub fn global_definitions(&self) -> &[GlobalDefinition<'t, 's>] {
        &self.global_definitions
    }

    pub fn strings(&self) -> &StringTable<'s> {
        &self.strings
    }
}

#[derive(Debug)]
pub enum SemanticAnalysisError<'s, 't> {
    Name(NameError<'s, 't>),
    FunctionDefinition(FunctionDefinitionError<'s, 't>),
    GlobalDefinition(GlobalDefinitionError<'s, 't>),
}

impl<'s, 't> From<NameError<'s, 't>> for SemanticAnalysisError<'s, 't> {
    fn from(value: NameError<'s, 't>) -> Self {
        Self::Name(value)
    }
}

impl<'s, 't> From<FunctionDefinitionError<'s, 't>> for SemanticAnalysisError<'s, 't> {
    fn from(value: FunctionDefinitionError<'s, 't>) -> Self {
        Self::FunctionDefinition(value)
    }
}

impl<'s, 't> From<GlobalDefinitionError<'s, 't>> for SemanticAnalysisError<'s, 't> {
    fn from(value: GlobalDefinitionError<'s, 't>) -> Self {
        Self::GlobalDefinition(value)
    }
}

impl<'s, 't> fmt::Display for SemanticAnalysisError<'s, 't> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name(error) => write!(f, "{}", error),
            Self::FunctionDefinition(error) => write!(f, "{}", error),
            Self::GlobalDefinition(error) => write!(f, "{}", error),
        }
    }
}
