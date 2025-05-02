use std::fmt;

use crate::parse::{AstNode, Atom, List, Source, TokenKind};

pub struct FunctionDefinition<'t, 's> {
    source: &'s Source,
    name: &'t Atom<'s>,
    args: &'t List<'s>,
    doc_string: Option<&'t Atom<'s>>,
    body: &'t[AstNode<'s>]
}

impl<'t, 's> FunctionDefinition<'t, 's> {
    /// Try to parse the ast node as a function definition.
    /// 
    /// Ok(None) if not a function definition.
    /// 
    /// Error if a function definition, but malformed.
    /// 
    /// Ok(Some) is the case when it's actually a function definition.
    pub fn extract(source: &'s Source, node: &'t AstNode<'s>) -> Result<Option<FunctionDefinition<'t, 's>>, FunctionDefinitionError<'t, 's>> {
        let list = match node.list() {
            None => return Ok(None), // ignore non-list root-level thingy
            Some(l) => l
        };

        let mut elements = list.elements().iter();

        let head = elements.next();
        let head = match head {
            None => return Ok(None), // ignore empty root-level definition
            Some(head) => head
        };
        let is_definition = match head {
            AstNode::Atom(first) if first.source_range().of(source).source() == "defun" => true,
            _ => false
        };
        if !is_definition {
            // ignore root-level directive that is not defun
            return Ok(None);
        }

        let name_node = elements.next()
            .ok_or_else(|| FunctionDefinitionError::MissingName { source, node })?;
        let name = name_node.atom().ok_or_else(|| FunctionDefinitionError::MalformedName { source, node: name_node })?;
        if !matches!(name.token().kind(), TokenKind::Ident) {
            return Err(FunctionDefinitionError::MalformedName { source, node: name_node });
        }

        let param_node = elements.next()
            .ok_or_else(|| FunctionDefinitionError::MissingParams { source, node })?;
        let params = param_node.list().ok_or_else(|| FunctionDefinitionError::MalformedParams { source, node: param_node })?;
        let found_non_ident = params.elements().iter().any(|p| match p {
            AstNode::Atom(a) if !matches!(a.token().kind(), TokenKind::Ident) => false,
            AstNode::Atom(_) => true,
            _ => false
        });
        if found_non_ident {
            return Err(FunctionDefinitionError::MalformedParams { source, node: param_node })
        }

        let mut rest = &list.elements()[3..];
        let doc_string_node = rest.get(0);
        let doc_string = match doc_string_node {
            Some(AstNode::Atom(a)) if matches!(a.token().kind(), TokenKind::StringLit) => {
                rest = &rest[1..];
                Some(a)
            },
            _ => None
        };

        let body = rest;

        Ok(Some(FunctionDefinition {
            source,
            name,
            args: params,
            doc_string,
            body
        }))
    }
}

#[derive(Debug)]
pub enum FunctionDefinitionError<'t, 's> {
    MissingName {
        source: &'s Source,
        node: &'t AstNode<'s>
    },
    MissingParams {
        source: &'s Source,
        node: &'t AstNode<'s>
    },
    MalformedName {
        source: &'s Source,
        node: &'t AstNode<'s>
    },
    MalformedParams {
        source: &'s Source,
        node: &'t AstNode<'s>
    },
}

impl<'t, 's> fmt::Display for FunctionDefinitionError<'t, 's> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionDefinitionError::MissingName { source, node } => {
                writeln!(
                    f,
                    "function definition is lacking a name:"
                )?;
                writeln!(
                    f,
                    "{}",
                    node.fragment(source).source_context()
                )
            },
            FunctionDefinitionError::MalformedName { source, node } => {
                writeln!(
                    f,
                    "not a valid function name:"
                )?;
                writeln!(
                    f,
                    "{}",
                    node.fragment(source).source_context()
                )
            },
            FunctionDefinitionError::MissingParams { source, node } => {
                writeln!(
                    f,
                    "function definition is lacking the parameter list:"
                )?;
                writeln!(
                    f,
                    "{}",
                    node.fragment(source).source_context()
                )
            },
            FunctionDefinitionError::MalformedParams { source, node } => {
                writeln!(
                    f,
                    "not a valid function parameter list:"
                )?;
                writeln!(
                    f,
                    "{}",
                    node.fragment(source).source_context()
                )
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parse::Parser;

    use super::*;

    #[test]
    fn two_fn_defs() {
        let source = &Source::new("
(defun max-2 (one two)
    (if (> one two) one two))

(defun max-inner (acc rest)
    (if (null rest)
        acc
        (max-inner
            (max-2 acc (car rest))
            (cdr rest))))");
        let ast = Parser::new(source).parse().unwrap();

        let definition = FunctionDefinition::extract(source, &ast[0]).unwrap().unwrap();
        assert_eq!(definition.name.source_range().of(source).source(), "max-2");

        let definition = FunctionDefinition::extract(source, &ast[1]).unwrap().unwrap();
        assert_eq!(definition.name.source_range().of(source).source(), "max-inner");
    }
}