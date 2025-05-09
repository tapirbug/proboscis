use std::fmt;

use crate::parse::{AstNode, Atom, List, TokenKind};
use crate::source::Source;

use super::form::{Form, FormError};

pub struct FunctionDefinition<'s, 't> {
    source: Source<'s>,
    name: &'t Atom<'s>,
    positional_args: Vec<&'t Atom<'s>>,
    rest_arg: Option<&'t Atom<'s>>,
    doc_string: Option<&'t Atom<'s>>,
    body: Vec<Form<'s, 't>>,
}

impl<'s, 't> FunctionDefinition<'s, 't> {
    /// Try to parse the ast node as a function definition.
    ///
    /// Ok(None) if not a function definition.
    ///
    /// Error if a function definition, but malformed.
    ///
    /// Ok(Some) is the case when it's actually a function definition.
    pub fn extract(
        source: Source<'s>,
        node: &'t AstNode<'s>,
    ) -> Result<Option<FunctionDefinition<'s, 't>>, FunctionDefinitionError<'s, 't>> {
        let list = match node.list() {
            None => return Ok(None), // ignore non-list root-level thingy
            Some(l) => l,
        };

        let mut elements = list.elements().iter();

        let head = elements.next();
        let head = match head {
            None => return Ok(None), // ignore empty root-level definition
            Some(head) => head,
        };
        let is_definition = match head {
            AstNode::Atom(first) if first.source_range().of(source).source() == "defun" => true,
            _ => false,
        };
        if !is_definition {
            // ignore root-level directive that is not defun
            return Ok(None);
        }

        let name_node = elements
            .next()
            .ok_or_else(|| FunctionDefinitionError::MissingName { source, node })?;
        let name = name_node
            .atom()
            .ok_or_else(|| FunctionDefinitionError::MalformedName {
                source,
                node: name_node,
            })?;
        if !matches!(name.token().kind(), TokenKind::Ident) {
            return Err(FunctionDefinitionError::MalformedName {
                source,
                node: name_node,
            });
        }

        let param_node = elements
            .next()
            .ok_or_else(|| FunctionDefinitionError::MissingParams { source, node })?;
        let param_list =
            param_node
                .list()
                .ok_or_else(|| FunctionDefinitionError::MalformedParams {
                    source,
                    node: param_node,
                })?;
        let found_non_ident_param = param_list.elements().iter().any(|p| match p {
            AstNode::Atom(a) if matches!(a.token().kind(), TokenKind::Ident) => false,
            _ => true,
        });
        if found_non_ident_param {
            return Err(FunctionDefinitionError::MalformedParams {
                source,
                node: param_node,
            });
        }
        let rest_pos = param_list
            .elements()
            .iter()
            .enumerate()
            .find(|(_, a)| {
                a.atom()
                    .map(|a| a.fragment(source).source() == "&rest")
                    .unwrap_or(false)
            })
            .map(|(idx, _)| idx);
        let (positional_params, rest_param) = match rest_pos {
            // rest and maybe also positional
            Some(rest_idx) => {
                let elements = param_list.elements();
                let after_rest_count = elements.len() - rest_idx;
                if after_rest_count == 0 {
                    return Err(FunctionDefinitionError::RestMissingName {
                        source,
                        rest: &param_list.elements()[rest_idx],
                    });
                } else if after_rest_count > 2 {
                    return Err(FunctionDefinitionError::RestAdditionalName {
                        source,
                        additional: &param_list.elements()[rest_idx + 2],
                    });
                }
                let positional = &elements[0..rest_idx];
                let rest = &elements[rest_idx + 1];
                (positional, Some(rest))
            }
            // only positional arguments
            None => (param_list.elements(), None),
        };
        let positional_params = positional_params
            .iter()
            .map(|p| p.atom().unwrap())
            .collect();
        let rest_param = rest_param.map(|r| r.atom().unwrap());

        let mut rest = &list.elements()[3..];

        let doc_string_node = rest.get(0);
        let doc_string = match doc_string_node {
            Some(AstNode::Atom(a)) if matches!(a.token().kind(), TokenKind::StringLit) => {
                rest = &rest[1..];
                Some(a)
            }
            _ => None,
        };

        let body = rest;
        let mut body_forms = vec![];
        for body in body {
            body_forms.push(Form::extract(source, body)?);
        }

        Ok(Some(FunctionDefinition {
            source,
            name,
            positional_args: positional_params,
            rest_arg: rest_param,
            doc_string,
            body: body_forms,
        }))
    }

    pub fn source(&self) -> Source<'s> {
        self.source
    }

    pub fn name(&self) -> &'t Atom<'s> {
        self.name
    }

    pub fn positional_args(&self) -> &[&'t Atom<'s>] {
        &self.positional_args
    }

    pub fn rest_arg(&self) -> Option<&'t Atom<'s>> {
        self.rest_arg
    }

    pub fn doc_string(&self) -> Option<&'t Atom<'s>> {
        self.doc_string
    }

    pub fn body(&self) -> &[Form<'s, 't>] {
        &self.body
    }
}

#[derive(Debug)]
pub enum FunctionDefinitionError<'s, 't> {
    MissingName {
        source: Source<'s>,
        node: &'t AstNode<'s>,
    },
    MissingParams {
        source: Source<'s>,
        node: &'t AstNode<'s>,
    },
    MalformedName {
        source: Source<'s>,
        node: &'t AstNode<'s>,
    },
    MalformedParams {
        source: Source<'s>,
        node: &'t AstNode<'s>,
    },
    RestMissingName {
        source: Source<'s>,
        rest: &'t AstNode<'s>,
    },
    RestAdditionalName {
        source: Source<'s>,
        additional: &'t AstNode<'s>,
    },
    FormError(FormError<'s, 't>),
}

impl<'s, 't> From<FormError<'s, 't>> for FunctionDefinitionError<'s, 't> {
    fn from(value: FormError<'s, 't>) -> Self {
        Self::FormError(value)
    }
}

impl<'s, 't> fmt::Display for FunctionDefinitionError<'s, 't> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionDefinitionError::MissingName { source, node } => {
                writeln!(f, "function definition is lacking a name:")?;
                writeln!(f, "{}", node.fragment(*source).source_context())
            }
            FunctionDefinitionError::MalformedName { source, node } => {
                writeln!(f, "not a valid function name:")?;
                writeln!(f, "{}", node.fragment(*source).source_context())
            }
            FunctionDefinitionError::MissingParams { source, node } => {
                writeln!(f, "function definition is lacking the parameter list:")?;
                writeln!(f, "{}", node.fragment(*source).source_context())
            }
            FunctionDefinitionError::MalformedParams { source, node } => {
                writeln!(f, "not a valid function parameter list:")?;
                writeln!(f, "{}", node.fragment(*source).source_context())
            }
            FunctionDefinitionError::RestMissingName { source, rest } => {
                writeln!(f, "&rest specified but no name given after it:")?;
                writeln!(f, "{}", rest.fragment(*source).source_context())
            }
            FunctionDefinitionError::RestAdditionalName { source, additional } => {
                writeln!(f, "&rest specified with two or more names after it:")?;
                writeln!(f, "{}", additional.fragment(*source).source_context())
            }
            FunctionDefinitionError::FormError(e) => write!(f, "{}", e),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{parse::Parser, source::SourceSet};

    use super::*;

    #[test]
    fn two_fn_defs() {
        let source_set = SourceSet::new_debug(
            "
(defun max-2 (one two)
    (if (> one two) one two))

(defun max-inner (acc rest)
    (if (null rest)
        acc
        (max-inner
            (max-2 acc (car rest))
            (cdr rest))))",
        );
        let source = source_set.one();
        let ast = Parser::new(source).parse().unwrap();

        let definition = FunctionDefinition::extract(source, &ast.root_nodes()[0])
            .unwrap()
            .unwrap();
        assert_eq!(definition.name.source_range().of(source).source(), "max-2");

        let definition = FunctionDefinition::extract(source, &ast.root_nodes()[1])
            .unwrap()
            .unwrap();
        assert_eq!(
            definition.name.source_range().of(source).source(),
            "max-inner"
        );
    }
}
