use super::{stream::TokenStream, token::TokenKind};

use crate::source::Source;

pub struct Ignore<I, const N: usize> {
    inner: I,
    ignore_kinds: [TokenKind; N],
}

impl<'s, I: TokenStream<'s>, const N: usize> Ignore<I, N> {
    pub fn new(inner: I, ignore: [TokenKind; N]) -> Self {
        Self {
            inner,
            ignore_kinds: ignore,
        }
    }
}

impl<'s, I: TokenStream<'s>, const N: usize> TokenStream<'s> for Ignore<I, N> {
    fn next<'l>(
        &'l mut self,
    ) -> Option<Result<super::token::Token<'s>, super::lexer::LexerError<'s>>>
    {
        let token = match self.inner.next()? {
            Err(e) => return Some(Err(e)),
            Ok(token) => token,
        };

        if self.ignore_kinds.contains(&token.kind()) {
            // ignored kind, skip the token and skip to the next one
            return self.next();
        }

        // not an ignored token kind
        Some(Ok(token))
    }

    fn source<'l>(&'l self) -> Source<'s> {
        self.inner.source()
    }
}

#[cfg(test)]
mod test {
    use crate::source::SourceSet;

    use super::super::lexer::Lexer;
    use super::*;

    #[test]
    fn ignore_floats_and_whitespace() {
        let source_set = SourceSet::new_debug("4 14.4 3 12. 213 .11 ok cool");
        let source = source_set.one();
        let mut ignore = Ignore::new(
            Lexer::new(source),
            [TokenKind::Ws, TokenKind::FloatLit],
        );

        let next_token = ignore.next().unwrap().unwrap();
        let next_fragment = next_token.fragment(source);
        let next_src = next_fragment.source();
        assert_eq!(next_src, "4");

        let next_token = ignore.next().unwrap().unwrap();
        let next_fragment = next_token.fragment(source);
        let next_src = next_fragment.source();
        assert_eq!(next_src, "3");

        let next_token = ignore.next().unwrap().unwrap();
        let next_fragment = next_token.fragment(source);
        let next_src = next_fragment.source();
        assert_eq!(next_src, "213");

        let next_token = ignore.next().unwrap().unwrap();
        let next_fragment = next_token.fragment(source);
        let next_src = next_fragment.source();
        assert_eq!(next_src, "ok");

        let next_token = ignore.next().unwrap().unwrap();
        let next_fragment = next_token.fragment(source);
        let next_src = next_fragment.source();
        assert_eq!(next_src, "cool");
    }
}
