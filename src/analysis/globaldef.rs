use std::fmt;

use crate::parse::{AstNode, Atom, Source, TokenKind};

pub struct GlobalDefinition<'t, 's> {
    source: &'s Source,
    name: &'t Atom<'s>,
    value: &'t AstNode<'s>,
}

impl<'t, 's> GlobalDefinition<'t, 's> {
    pub fn extract(
        source: &'s Source,
        node: &'t AstNode<'s>,
    ) -> Result<Option<GlobalDefinition<'t, 's>>, GlobalDefinitionError<'t, 's>>
    {
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
            AstNode::Atom(first)
                if first.source_range().of(source).source()
                    == "defparameter" =>
            {
                true
            }
            _ => false,
        };
        if !is_definition {
            // ignore root-level directive that is not defun
            return Ok(None);
        }

        let name_node = elements.next().ok_or_else(|| {
            GlobalDefinitionError::MissingName { source, node }
        })?;
        let name = name_node.atom().ok_or_else(|| {
            GlobalDefinitionError::MalformedName {
                source,
                node: name_node,
            }
        })?;
        if !matches!(name.token().kind(), TokenKind::Ident) {
            return Err(GlobalDefinitionError::MalformedName {
                source,
                node: name_node,
            });
        }

        let value = elements.next().ok_or_else(|| {
            GlobalDefinitionError::MissingValue { source, node }
        })?;

        match elements.next() {
            None => {}
            Some(superflous) => {
                return Err(GlobalDefinitionError::SuperfluousValue {
                    source,
                    node: superflous,
                });
            }
        };

        Ok(Some(GlobalDefinition { source, name, value }))
    }

    pub fn source(&self) -> &'s Source {
        self.source
    }

    pub fn name(&self) -> &'t Atom<'s> {
        self.name
    }

    pub fn value(&self) -> &'t AstNode<'s> {
        self.value
    }
}

#[derive(Debug)]
pub enum GlobalDefinitionError<'t, 's> {
    MissingName {
        source: &'s Source,
        node: &'t AstNode<'s>,
    },
    MalformedName {
        source: &'s Source,
        node: &'t AstNode<'s>,
    },
    MissingValue {
        source: &'s Source,
        node: &'t AstNode<'s>,
    },
    SuperfluousValue {
        source: &'s Source,
        node: &'t AstNode<'s>,
    },
}

impl<'t, 's> fmt::Display for GlobalDefinitionError<'t, 's> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GlobalDefinitionError::MissingName { source, node } => {
                writeln!(f, "global definition is lacking a name:")?;
                writeln!(f, "{}", node.fragment(source).source_context())
            }
            GlobalDefinitionError::MalformedName { source, node } => {
                writeln!(f, "not a valid global name:")?;
                writeln!(f, "{}", node.fragment(source).source_context())
            }
            GlobalDefinitionError::MissingValue { source, node } => {
                writeln!(f, "global definition is lacking an initial value:")?;
                writeln!(f, "{}", node.fragment(source).source_context())
            }
            GlobalDefinitionError::SuperfluousValue { source, node } => {
                writeln!(
                    f,
                    "found superfluous values in global definition after the initial value:"
                )?;
                writeln!(f, "{}", node.fragment(source).source_context())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parse::Parser;

    use super::*;

    #[test]
    fn extract_global() {
        let source = &Source::new("(defparameter *list* '(1 2 3 4))
(format t \"Max of ~a is ~a\" *list* (maximum *list*))");
        let ast = Parser::new(source).parse().unwrap();
        let definition = GlobalDefinition::extract(source, &ast[0]).unwrap().unwrap();
        let name = definition.name().source_range().of(source).source();
        assert_eq!(name, "*list*");
        let value_code = definition.value().source_range().of(source).source();
        assert_eq!(value_code, "'(1 2 3 4)");

        let non_definition = GlobalDefinition::extract(source, &ast[1]).unwrap();
        assert!(non_definition.is_none());
    }
}