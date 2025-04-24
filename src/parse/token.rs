use super::frag::{Fragment, SourceRange};

#[derive(Clone, Debug)]
pub struct Token<'s> {
    range: SourceRange<'s>,
    kind: TokenKind,
}

impl<'s> Token<'s> {
    pub fn new(range: SourceRange<'s>, kind: TokenKind) -> Self {
        Token { range, kind }
    }

    pub fn fragment(&'s self, source: &'s str) -> Fragment<'s> {
        self.range.of(source)
    }

    pub fn kind(&self) -> TokenKind {
        self.kind
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TokenKind {
    LeftParen,
    RightParen,
    /// General identifier like a function name e.g. `map`, `+`
    Ident,
    IntLit,
    FloatLit,
    StringLit,
    /// White-space, including newlines.
    Ws,
}
