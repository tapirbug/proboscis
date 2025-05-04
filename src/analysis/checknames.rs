use std::fmt;

use crate::{parse::{AstNode, Atom, List, TokenKind}, source::Source};

use super::{semantic::SemanticAnalysis, FunctionDefinition, GlobalDefinition};

const GLOBAL_FUNCTIONS: &[&str] = &[
    "car", "cdr", "format", "if", "let", "list", "null", ">", ">=", "=", ">",
    ">=", "/=", // not equal
];
const GLOBAL_VARS: &[&str] = &["nil", "t"];

/// Checks that all names in an AST node refer to defined functions or
/// variables, depending on context.
pub struct NameCheck<'t, 's> {
    functions: &'t [FunctionDefinition<'t, 's>],
    globals: &'t [GlobalDefinition<'t, 's>],
    /// Variables from lexical scope
    scope_vars: Vec<Vec<&'s str>>,
}

impl<'t, 's> NameCheck<'t, 's> {
    pub fn check<'a: 't>(analysis: &'a SemanticAnalysis<'t, 's>) -> Result<(), NameError<'t, 's>> {
        let mut check = Self {
            functions: analysis.function_definitions(),
            globals: analysis.global_definitions(),
            scope_vars: vec![],
        };
        for root_code in analysis.root_code() {
            for code in root_code.code() {
                check.check_names(root_code.source(), code)?;
            }
        }
        for function in analysis.function_definitions() {
            check.push_fn_scope(function);
            for fn_code in function.body() {
                check.check_names(function.source(), fn_code)?;
            }
            check.pop_scope();
        }
        for global in analysis.global_definitions() {
            check.check_names(global.source(), global.value())?;
        }
        Ok(())
    }

    fn push_fn_scope(&mut self, fun: &'t FunctionDefinition<'t, 's>) {
        let mut scope_vars = vec![];
        for arg in fun.args().elements() {
            // can unwrap because the func
            let arg = arg
                .atom()
                .expect("only plain identifiers supported as arguments");
            let arg = arg.source_range().of(fun.source()).source();
            scope_vars.push(arg);
        }
        self.scope_vars.push(scope_vars);
    }

    fn pop_scope(&mut self) {
        self.scope_vars
            .pop()
            .expect("Tried to pop scope but scopes are empty");
    }

    fn check_names(
        &self,
        source: Source<'s>,
        code: &'t AstNode<'s>,
    ) -> Result<(), NameError<'s, 't>> {
        match code {
            AstNode::List(list) => self.check_invocation(source, list),
            // quoted stuff doesn't need to refer to defined names
            AstNode::Quoted(_) => Ok(()),
            AstNode::Atom(atom) => match atom.token().kind() {
                // these are not names, so always ok
                TokenKind::FloatLit
                | TokenKind::IntLit
                | TokenKind::StringLit
                | TokenKind::Comment
                | TokenKind::Ws
                | TokenKind::LeftParen
                | TokenKind::RightParen => Ok(()),
                // names must refer to variables for now (no #'+)
                TokenKind::Ident => self.check_variable_ident(source, atom),
            },
        }
    }

    fn check_fn_ident(
        &self,
        source: Source<'s>,
        ident_atom: &'t Atom<'s>,
    ) -> Result<(), NameError<'s, 't>> {
        let ident_str = ident_atom.source_range().of(source).source();
        for defined_fn in self.functions {
            let defined_fn_ident = defined_fn
                .name()
                .source_range()
                .of(defined_fn.source())
                .source();
            if ident_str == defined_fn_ident {
                return Ok(());
            }
        }
        let is_defined_global_fn = GLOBAL_FUNCTIONS.contains(&ident_str);
        if is_defined_global_fn {
            return Ok(());
        }
        Err(NameError::UndefinedFunctionName {
            source: source,
            name: ident_atom,
        })
    }

    fn check_variable_ident(
        &self,
        source: Source<'s>,
        ident_atom: &'t Atom<'s>,
    ) -> Result<(), NameError<'s, 't>> {
        let ident_str = ident_atom.source_range().of(source).source();
        let is_defined_scope_var = self
            .scope_vars
            .iter()
            .flat_map(|v| v.iter())
            .find(|&&scope_var_ident| scope_var_ident == ident_str)
            .is_some();
        if is_defined_scope_var {
            return Ok(());
        }
        for defined_global in self.globals {
            let defined_global_ident = defined_global
                .name()
                .source_range()
                .of(defined_global.source())
                .source();
            if ident_str == defined_global_ident {
                return Ok(());
            }
        }
        let is_defined_global_var = GLOBAL_VARS.contains(&ident_str);
        if is_defined_global_var {
            return Ok(());
        }
        Err(NameError::UndefinedVariableName {
            source: source,
            name: ident_atom,
        })
    }

    fn check_invocation(
        &self,
        source: Source<'s>,
        invocation: &'t List<'s>,
    ) -> Result<(), NameError<'s, 't>> {
        let invocation = invocation.elements();
        let function_ident = match invocation.get(0) {
            None => return Ok(()), // empty list is okay, it results in nil
            Some(fun) => fun,
        };
        let function_ident = function_ident
            .atom()
            .filter(|a| matches!(a.token().kind(), TokenKind::Ident))
            .ok_or_else(|| NameError::MalformedFunctionName {
                source,
                name: function_ident,
            })?;
        self.check_fn_ident(source, function_ident)?;

        let args = &invocation[1..];
        for arg in args {
            self.check_names(source, arg)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum NameError<'s, 't> {
    UndefinedFunctionName {
        source: Source<'s>,
        name: &'t Atom<'s>,
    },
    UndefinedVariableName {
        source: Source<'s>,
        name: &'t Atom<'s>,
    },
    MalformedFunctionName {
        source: Source<'s>,
        name: &'t AstNode<'s>,
    },
}

impl<'s, 't> fmt::Display for NameError<'s, 't> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NameError::UndefinedFunctionName { source, name } => {
                writeln!(
                    f,
                    "reference to undefined function `{}`:",
                    name.source_range().of(*source).source()
                )?;
                writeln!(f, "{}", name.fragment(*source).source_context())
            }
            NameError::MalformedFunctionName { source, name } => {
                writeln!(f, "expected a function name here:")?;
                writeln!(f, "{}", name.fragment(*source).source_context())
            }
            NameError::UndefinedVariableName { source, name } => {
                writeln!(
                    f,
                    "reference to undefined place `{}`:",
                    name.source_range().of(*source).source()
                )?;
                writeln!(f, "{}", name.fragment(*source).source_context())
            }
        }
    }
}
