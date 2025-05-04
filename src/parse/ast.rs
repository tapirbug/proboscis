use std::slice;

use crate::source::{Fragment, SourceRange};
use crate::source::Source;
use super::token::Token;

#[derive(Debug)]
pub struct Ast<'s> {
    source: Source<'s>,
    root_nodes: Vec<AstNode<'s>>
}

#[derive(Debug)]
pub enum AstNode<'s> {
    Atom(Atom<'s>),
    /// A standard list, usually a function invocation, e.g. `(max 4)` or `()`.
    List(List<'s>),
    /// A quoted list or identifier, e.g. `'(1 2)`.`, 'string
    Quoted(Quoted<'s>),
}

#[derive(Debug)]
pub struct Atom<'s> {
    token: Token<'s>,
}

#[derive(Debug)]
pub struct List<'s> {
    source_range: SourceRange<'s>,
    elements: Vec<AstNode<'s>>,
}

#[derive(Debug)]
pub struct Quoted<'s> {
    source_range: SourceRange<'s>,
    quoted: Box<AstNode<'s>>,
}

impl<'s> Ast<'s> {
    pub fn new(source: Source<'s>, root_nodes: Vec<AstNode<'s>>) -> Self {
        Self { source, root_nodes }
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.root_nodes.len()
    }

    pub fn source(&self) -> Source<'s> {
        self.source
    }

    pub fn root_nodes(&self) -> &[AstNode<'s>] {
        &self.root_nodes
    }

    pub fn iter(&self) -> slice::Iter<AstNode<'s>> {
        self.root_nodes.iter()
    }
}

impl<'s> IntoIterator for Ast<'s> {
    type Item = AstNode<'s>;
    type IntoIter = std::vec::IntoIter<AstNode<'s>>;

    fn into_iter(self) -> Self::IntoIter {
        self.root_nodes.into_iter()
    }
}

impl<'s> AstNode<'s> {
    pub fn fragment<'a>(&'a self, source: Source<'s>) -> Fragment<'s> {
        self.source_range().of(source)
    }

    pub fn source_range<'a>(&'a self) -> SourceRange<'s> {
        match self {
            &AstNode::Atom(Atom { ref token }) => token.source_range(),
            &AstNode::List(List { source_range, .. }) => source_range,
            &AstNode::Quoted(Quoted { source_range, .. }) => source_range
        }
    }

    pub fn atom<'a>(&'a self) -> Option<&'a Atom<'s>> {
        if let &AstNode::Atom(ref atom) = self {
            Some(atom)
        } else {
            None
        }
    }

    pub fn list<'a>(&'a self) -> Option<&'a List<'s>> {
        if let &AstNode::List(ref list) = self {
            Some(list)
        } else {
            None
        }
    }

    pub fn quoted<'a>(&'a self) -> Option<&'a Quoted<'s>> {
        if let &AstNode::Quoted(ref list) = self {
            Some(list)
        } else {
            None
        }
    }
}

impl<'s> Atom<'s> {
    pub fn new(token: Token<'s>) -> AstNode<'s> {
        AstNode::Atom(Atom { token })
    }

    pub fn token<'b>(&'b self) -> &'b Token<'s> {
        &self.token
    }

    pub fn source_range<'b>(&'b self) -> SourceRange<'s> {
        self.token.source_range()
    }

    pub fn fragment<'a>(&'a self, source: Source<'s>) -> Fragment<'s> {
        self.source_range().of(source)
    }
}

impl<'s> List<'s> {
    pub fn new(
        source_range: SourceRange<'s>,
        elements: Vec<AstNode<'s>>,
    ) -> AstNode<'s> {
        AstNode::List(List {
            source_range,
            elements,
        })
    }

    pub fn source_range<'b>(&'b self) -> SourceRange<'s> {
        self.source_range
    }

    pub fn elements<'b>(&'b self) -> &'b [AstNode<'s>] {
        &self.elements
    }

    pub fn fragment<'a>(&'a self, source: Source<'s>) -> Fragment<'s> {
        self.source_range().of(source)
    }
}

impl<'s> Quoted<'s> {
    pub fn new(
        source_range: SourceRange<'s>,
        quoted: AstNode<'s>,
    ) -> AstNode<'s> {
        AstNode::Quoted(Quoted {
            source_range,
            quoted: Box::new(quoted),
        })
    }

    pub fn source_range<'b>(&'b self) -> SourceRange<'s> {
        self.source_range
    }

    pub fn quoted<'b>(&'b self) -> &'b AstNode<'s> {
        &self.quoted
    }

    pub fn fragment<'a>(&'a self, source: Source<'s>) -> Fragment<'s> {
        self.source_range().of(source)
    }
}
