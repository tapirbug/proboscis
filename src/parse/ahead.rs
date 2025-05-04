use super::{
    lexer::{Lexer, LexerError},
    stream::TokenStream,
    token::Token,
};
use crate::source::Source;
use std::mem;

/// Provides token lookahead for a lexer to be used by a parser.
pub struct LookaheadStream<'s, I, const N: usize> {
    /// Inner token stream to get tokens from
    inner: I,
    /// The lookahead buffer, unused entries are None
    buf: [LookaheadEntry<'s>; N],
}

enum LookaheadEntry<'s> {
    /// Not filled yet, or used as a temporary placeholder value when taking
    /// out an item from the front and then shifting the rest to the left.
    Unused,
    /// results of calls to next_token()
    Used(Option<Result<Token<'s>, LexerError<'s>>>),
}

/*pub struct Lookahead<'b, 's, const N: usize> {
    buf: &'b [LookaheadEntry<'s>; N]
}*/

impl<'s, I: TokenStream<'s>, const N: usize> LookaheadStream<'s, I, N> {
    pub fn new(inner: I) -> Self {
        assert!(N >= 1);
        Self {
            inner,
            buf: [const { LookaheadEntry::Unused }; N],
        }
    }

    /// Ensures entries in the buffer up to and including the given index are
    /// set to used entry, by calling next on the lexer if needed.
    ///
    /// This method will never remove any entries.
    fn ensure_used<'b>(&'b mut self, ahead: usize) {
        assert!(ahead < N);
        for offset in 0..=ahead {
            let unused = match self.buf[offset] {
                LookaheadEntry::Unused => true,
                _ => false,
            };
            if unused {
                // we could avoid calling lexer again when it returned none but why bother
                self.buf[offset] = LookaheadEntry::Used(self.inner.next())
            }
        }
    }

    /// Gets the full lookahead buffer statically sized.
    ///
    /// Same as iter_lookahead otherwise.
    pub fn max_lookahead<'b>(
        &'b mut self,
    ) -> [Option<Result<&'b Token<'s>, &'b LexerError<'s>>>; N] {
        self.ensure_used(N - 1);
        let mut ahead_refs = self.iter_lookahead();
        [(); N].map(|_| ahead_refs.next().unwrap())
    }

    /// Get a reference to the next results that would be consumed,
    /// without advancing the position or consuming the token, but
    /// calling the lexer for more tokens as needed to satisfy the lookahead.
    ///
    /// The returned references are all disjoint and remain live until the
    /// next mutable action (including lookahead, but also calling next or
    /// consume), so this cannot be called multiple times and returns all
    /// at once.
    pub fn iter_lookahead<'b>(
        &'b mut self,
    ) -> impl Iterator<Item = Option<Result<&'b Token<'s>, &'b LexerError<'s>>>> + 'b
    {
        self.ensure_used(N - 1);
        self.buf.iter().map(|ahead| match ahead {
            LookaheadEntry::Unused => unreachable!(),
            LookaheadEntry::Used(None) => None,
            LookaheadEntry::Used(Some(Err(e))) => Some(Err(e)),
            LookaheadEntry::Used(Some(Ok(token))) => Some(Ok(token)),
        })
    }

    /// Get the next token from the front and remove it, advancing the position
    /// for future calls to peek and next.
    fn consume<'b>(&'b mut self) -> Option<Result<Token<'s>, LexerError<'s>>> {
        assert!(N >= 1);
        self.ensure_used(0);
        // remove the first entry and replace it with Unused, then let it bubble
        // up by swapping every element with its left neighbour
        let head = mem::take(&mut self.buf[0]);
        for swap_right in 1..N {
            self.buf.swap(swap_right - 1, swap_right);
        }
        match head {
            LookaheadEntry::Unused => unreachable!(),
            LookaheadEntry::Used(u) => u,
        }
    }
}

impl<'s, I: TokenStream<'s>, const N: usize> TokenStream<'s>
    for LookaheadStream<'s, I, N>
{
    fn next<'l>(&'l mut self) -> Option<Result<Token<'s>, LexerError<'s>>> {
        self.consume()
    }

    fn source<'l>(&'l self) -> Source<'s> {
        self.inner.source()
    }
}

impl<'s> Default for LookaheadEntry<'s> {
    fn default() -> Self {
        LookaheadEntry::Unused
    }
}

#[cfg(test)]
mod test {
    use crate::source::SourceSet;

    use super::*;

    #[should_panic]
    #[test]
    fn cannot_create_lookahead_0() {
        let source_set = SourceSet::new_debug("a 12 dead beef");
        let source = source_set.one();
        let _ahead: LookaheadStream<_, 0> =
            LookaheadStream::new(Lexer::new(source));
    }

    #[test]
    fn lookahead_1_lookahead_all_at_once() {
        let source_set = SourceSet::new_debug("a 12 dead beef");
        let source = source_set.one();
        let mut ahead: LookaheadStream<_, 1> =
            LookaheadStream::new(Lexer::new(source));
        let [ahead0] = ahead.max_lookahead();
        let ahead0 = ahead0.unwrap().unwrap();
        let ahead0_fragment = ahead0.fragment(source);
        let ahead0_src = ahead0_fragment.source();

        assert_eq!(ahead0_src, "a");

        // consume a and space
        ahead.next();
        ahead.next();

        let [ahead0] = ahead.max_lookahead();
        let ahead0 = ahead0.unwrap().unwrap();
        let ahead0_fragment = ahead0.fragment(source);
        let ahead0_src = ahead0_fragment.source();

        assert_eq!(ahead0_src, "12");
    }

    #[test]
    fn lookahead_2_iter() {
        let source_set = SourceSet::new_debug("a 12 dead beef");
        let source = source_set.one();
        let mut ahead: LookaheadStream<_, 2> =
            LookaheadStream::new(Lexer::new(source));
        {
            let mut ahead = ahead.iter_lookahead();

            let ahead0 = ahead.next().unwrap().unwrap().unwrap();
            let ahead0_fragment = ahead0.fragment(source);
            let ahead0_src = ahead0_fragment.source();

            let ahead1 = ahead.next().unwrap().unwrap().unwrap();
            let ahead1_fragment = ahead1.fragment(source);
            let ahead1_src = ahead1_fragment.source();

            assert_eq!(ahead0_src, "a");
            assert_eq!(ahead1_src, " ");
            assert!(ahead.next().is_none());
            assert!(ahead.next().is_none());
        } // all borrows are dropped here and we can continue reading and looking ahead

        // drop 3
        ahead.next();
        ahead.next();
        ahead.next();

        let mut ahead = ahead.iter_lookahead();

        let ahead0 = ahead.next().unwrap().unwrap().unwrap();
        let ahead0_fragment = ahead0.fragment(source);
        let ahead0_src = ahead0_fragment.source();

        let ahead1 = ahead.next().unwrap().unwrap().unwrap();
        let ahead1_fragment = ahead1.fragment(source);
        let ahead1_src = ahead1_fragment.source();

        assert_eq!(ahead0_src, " ");
        assert_eq!(ahead1_src, "dead");
        assert!(ahead.next().is_none());
        assert!(ahead.next().is_none());
    }
}
