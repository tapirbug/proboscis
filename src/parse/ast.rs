use super::frag::{Fragment, SourceRange};
use super::source::Source;
use super::token::Token;

pub enum AstNode<'s> {
    Atom(Atom<'s>),
    List(List<'s>),
}

pub struct Atom<'s> {
    token: Token<'s>,
}

pub struct List<'s> {
    source_range: SourceRange<'s>,
    elements: Vec<AstNode<'s>>,
}

impl<'s> AstNode<'s> {
    pub fn fragment<'a>(&'a self, source: &'s Source) -> Fragment<'s> {
        self.source_range().of(source)
    }

    pub fn source_range<'a>(&'a self) -> SourceRange<'s> {
        match self {
            &AstNode::Atom(Atom { ref token }) => token.source_range(),
            &AstNode::List(List { source_range, .. }) => source_range,
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
}
