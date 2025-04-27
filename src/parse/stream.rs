use super::{lexer::LexerError, source::Source, token::Token};

/// Something that yields tokens sequentially, with a lifetime for the source
/// `'s` and tokens in the source that is separate from the mutable lifetime
/// used for self `'l`.
pub trait TokenStream<'s> {
    /// Tries to get the next token, if any, returning `None` if the end is
    /// reached.
    ///
    /// If an error is encountered, returns the error, and then returns only
    /// `None` on future calls, as if finished parsing.
    fn next<'l>(&'l mut self) -> Option<Result<Token<'s>, LexerError<'s>>>;

    /// Gets the source that the tokens refer to.
    fn source<'l>(&'l self) -> &'s Source;
}
