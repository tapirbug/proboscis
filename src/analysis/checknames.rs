use std::fmt;

use crate::{
    parse::{AstNode, Atom, List, TokenKind},
    source::Source,
};

use super::{FunctionDefinition, GlobalDefinition, form::Form, semantic::SemanticAnalysis};

const GLOBAL_FUNCTIONS: &[&str] = &[
    "concat-string-like-2",
    "type-tag-of",
    "cons",
    "add-2",
    "sub-2",
    "nil-if-0",
    "car",
    "cdr",
    "format",
    "if",
    "let",
    "list",
    "null",
    "apply",
    ">",
    ">=",
    "=",
    ">",
    ">=",
    "/=", // not equal
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
        for &arg in fun.positional_args() {
            let arg = arg.source_range().of(fun.source()).source();
            scope_vars.push(arg);
        }
        if let Some(rest) = fun.rest_arg() {
            let arg = rest.source_range().of(fun.source()).source();
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
        &mut self,
        source: Source<'s>,
        code: &'t Form<'s, 't>,
    ) -> Result<(), NameError<'s, 't>> {
        match code {
            Form::Name(name) => self.check_variable_ident(source, name.ident()),
            Form::FunctionName(func_name) => self.check_fn_ident(source, func_name.ident()),
            Form::Constant(_) => Ok(()), // constants don't have any names that need to be checked
            Form::IfForm(if_form) => {
                self.check_names(source, if_form.test_form())?;
                self.check_names(source, if_form.then_form())?;
                if let Some(else_form) = if_form.else_form() {
                    self.check_names(source, else_form)?;
                }
                Ok(())
            }
            Form::AndForm(and) => {
                for code in and.forms() {
                    self.check_names(source, code)?;
                }
                Ok(())
            }
            Form::OrForm(or) => {
                for code in or.forms() {
                    self.check_names(source, code)?;
                }
                Ok(())
            }
            Form::LetForm(let_form) => {
                // first check the values with the surrounding scope
                for binding in let_form.bindings() {
                    self.check_names(source, binding.value())?;
                }
                // and only then add to scope, because that are the correct semantics for parallel assignment (I think)
                let mut let_scope = vec![];
                for binding in let_form.bindings() {
                    let name = binding.name();
                    let name = name.fragment(source).source();
                    let_scope.push(name);
                }
                self.scope_vars.push(let_scope);
                for body in let_form.body() {
                    self.check_names(source, body)?;
                }
                self.scope_vars.pop();
                Ok(())
            }
            Form::Call(call) => {
                self.check_fn_ident(source, call.function())?;
                for arg in call.args() {
                    self.check_names(source, arg)?;
                }
                Ok(())
            }
            Form::Apply(apply) => {
                self.check_names(source, apply.function())?;
                self.check_names(source, apply.args())
            }
        }
    }

    fn check_fn_ident(
        &self,
        source: Source<'s>,
        ident_atom: &'t Atom<'s>,
    ) -> Result<(), NameError<'s, 't>> {
        let ident_str = if matches!(ident_atom.token().kind(), TokenKind::FuncIdent) {
            ident_atom.source_range().of(source).source()[2..].trim()
        } else {
            // normal identifiers verbatim
            ident_atom.source_range().of(source).source()
        };
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
