use std::fmt;

use crate::{
    parse::{AstNode, AstSet},
    source::Source,
};

use super::{
    FunctionDefinition, FunctionDefinitionError, GlobalDefinition,
    GlobalDefinitionError, NameError,
    form::{Form, FormError},
};

pub struct SemanticAnalysis<'s, 't> {
    // REVIEW could it be a problem that function definitions and root code are not ordered with respect to each other?
    root_code: Vec<RootCode<'s, 't>>,
    function_definitions: Vec<FunctionDefinition<'s, 't>>,
    global_definitions: Vec<GlobalDefinition<'s, 't>>,
}

pub struct RootCode<'s, 't> {
    source: Source<'s>,
    root_code: Vec<Form<'s, 't>>,
}

impl<'s, 't> SemanticAnalysis<'s, 't> {
    /// Semantically analyses the asts of a group of source files that form
    /// a unit.
    pub fn analyze(
        asts: &'t AstSet<'s>,
    ) -> Result<SemanticAnalysis<'s, 't>, SemanticAnalysisError<'s, 't>> {
        // TODO find constant numbers too

        let mut root_codes = vec![];
        let mut function_definitions = vec![];
        let mut global_definitions = vec![];

        for ast in asts.iter() {
            let mut root_code = vec![];
            for root_node in ast.iter() {
                // try parsing root-level element as a function first
                let def =
                    FunctionDefinition::extract(ast.source(), root_node)?;
                match def {
                    Some(def) => {
                        function_definitions.push(def);
                        continue;
                    }
                    None => {}
                }

                // then as a global
                let def = GlobalDefinition::extract(ast.source(), root_node)?;
                match def {
                    Some(def) => {
                        global_definitions.push(def);
                        continue;
                    }
                    None => {}
                }

                // all other cases are considered to be top-level code
                root_code.push(
                    Form::extract(ast.source(), root_node).map_err(|e| {
                        SemanticAnalysisError::RootFormError(e)
                    })?,
                );
            }
            root_codes.push(RootCode {
                source: ast.source(),
                root_code,
            });
        }

        Ok(SemanticAnalysis {
            root_code: root_codes,
            function_definitions,
            global_definitions,
        })
    }
}

impl<'s, 't> SemanticAnalysis<'s, 't> {
    pub fn root_code(&self) -> &[RootCode<'s, 't>] {
        &self.root_code
    }

    pub fn function_definitions(&self) -> &[FunctionDefinition<'s, 't>] {
        &self.function_definitions
    }

    pub fn global_definitions(&self) -> &[GlobalDefinition<'s, 't>] {
        &self.global_definitions
    }
}

#[derive(Debug)]
pub enum SemanticAnalysisError<'s, 't> {
    Name(NameError<'s, 't>),
    FunctionDefinition(FunctionDefinitionError<'s, 't>),
    GlobalDefinition(GlobalDefinitionError<'s, 't>),
    RootFormError(FormError<'s, 't>),
}

impl<'s, 't> From<NameError<'s, 't>> for SemanticAnalysisError<'s, 't> {
    fn from(value: NameError<'s, 't>) -> Self {
        Self::Name(value)
    }
}

impl<'s, 't> From<FunctionDefinitionError<'s, 't>>
    for SemanticAnalysisError<'s, 't>
{
    fn from(value: FunctionDefinitionError<'s, 't>) -> Self {
        Self::FunctionDefinition(value)
    }
}

impl<'s, 't> From<GlobalDefinitionError<'s, 't>>
    for SemanticAnalysisError<'s, 't>
{
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
            Self::RootFormError(e) => write!(f, "{e}"),
        }
    }
}

impl<'s, 't> RootCode<'s, 't> {
    pub fn source(&self) -> Source<'s> {
        self.source
    }

    pub fn code(&self) -> &[Form<'s, 't>] {
        &self.root_code
    }
}
