use super::frag::{Fragment, SourceRange};
use super::source::Source;
use super::token::Token;

#[derive(Debug)]
pub enum AstNode<'s> {
    Atom(Atom<'s>),
    /// A standard list, usually a function invocation, e.g. `(max 4)` or `()`.
    List(List<'s>),
    /// A quoted list, e.g. `'(1 2)`.`
    QuotedList(QuotedList<'s>),
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
pub struct QuotedList<'s> {
    /// Source range including the tick.
    ///
    /// Note that there could be white-space between the tick and the opening
    /// parenthesis marking the start of list entries.
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
            &AstNode::QuotedList(QuotedList { source_range, .. }) => {
                source_range
            }
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

    pub fn quoted_list<'a>(&'a self) -> Option<&'a QuotedList<'s>> {
        if let &AstNode::QuotedList(ref list) = self {
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

impl<'s> QuotedList<'s> {
    pub fn new(
        source_range: SourceRange<'s>,
        elements: Vec<AstNode<'s>>,
    ) -> AstNode<'s> {
        AstNode::QuotedList(QuotedList {
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
