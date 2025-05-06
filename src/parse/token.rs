use crate::source::{Fragment, Source, SourceRange};

#[derive(Clone, Debug)]
pub struct Token<'s> {
    range: SourceRange<'s>,
    kind: TokenKind,
}

impl<'s> Token<'s> {
    pub fn new(range: SourceRange<'s>, kind: TokenKind) -> Self {
        Token { range, kind }
    }

    pub fn fragment<'a>(&'a self, source: Source<'s>) -> Fragment<'s> {
        self.range.of(source)
    }

    pub fn source_range<'a>(&'a self) -> SourceRange<'s> {
        self.range
    }

    pub fn kind<'a>(&'a self) -> TokenKind {
        self.kind
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    /// Opening parenthesis.
    LeftParen,
    /// Closing parenthesis.
    RightParen,
    /// General identifier like a function name e.g. `map`, `+`
    Ident,
    /// Integer like `5`, `-42`, `+3`.
    IntLit,
    /// Float like `.3`, `0.5`, `6.`
    FloatLit,
    /// String literal like `""` or `"\"asdf\""`
    StringLit,
    /// Comment excluding the newline that ends it e.g. `; this is a comment`.
    Comment,
    /// White-space, including newlines and the newlines directly after
    /// comments.
    Ws,
    /// A single quote, used for escaping the next thing, e.g. the quote
    /// in `'string`.
    Quote,
}
