use crate::{diagnostic::Diagnostics, parse::AstSet, source::Source};

use super::{FunctionDefinition, GlobalDefinition, form::Form};

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
        diagnostics: &mut Diagnostics,
        asts: &'t AstSet<'s>,
    ) -> SemanticAnalysis<'s, 't> {
        // TODO find constant numbers too

        let mut root_codes = vec![];
        let mut function_definitions = vec![];
        let mut global_definitions = vec![];

        for ast in asts.iter() {
            let mut root_code = vec![];
            for root_node in ast.iter() {
                // try parsing root-level element as a function first
                let def = FunctionDefinition::extract(ast.source(), root_node);
                match def {
                    Ok(Some(def)) => {
                        function_definitions.push(def);
                        continue;
                    }
                    Ok(None) => {}
                    Err(ref error) => {
                        // error in function definition, continue with other root elements and stop later
                        diagnostics.report(error);
                        continue;
                    }
                }

                // then as a global
                let def = GlobalDefinition::extract(ast.source(), root_node);
                match def {
                    Ok(Some(def)) => {
                        global_definitions.push(def);
                        continue;
                    }
                    Ok(None) => {}
                    Err(ref error) => {
                        diagnostics.report(error);
                        continue;
                    }
                }

                // all other cases are considered to be top-level code
                if let Some(next_root) =
                    diagnostics.ok(Form::extract(ast.source(), root_node))
                {
                    root_code.push(next_root);
                }
            }
            root_codes.push(RootCode {
                source: ast.source(),
                root_code,
            });
        }

        SemanticAnalysis {
            root_code: root_codes,
            function_definitions,
            global_definitions,
        }
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

impl<'s, 't> RootCode<'s, 't> {
    pub fn source(&self) -> Source<'s> {
        self.source
    }

    pub fn code(&self) -> &[Form<'s, 't>] {
        &self.root_code
    }
}
