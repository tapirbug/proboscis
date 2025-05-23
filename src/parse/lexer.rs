use std::fmt;

use crate::source::{Fragment, Source, SourceRange};

use super::stream::TokenStream;
use super::token::{Token, TokenKind};

pub struct Lexer<'s> {
    source: Source<'s>,
    position: usize,
}

impl<'s> Lexer<'s> {
    pub fn new(source: Source<'s>) -> Self {
        Self {
            source,
            position: 0,
        }
    }

    // Consume `len` characters and return a source range for them.
    fn take<'l>(&'l mut self, len: usize) -> SourceRange<'s> {
        assert!(len > 0);
        let from = self.position;
        let to = self.position + len;
        self.position = to;
        SourceRange::new(from, to)
    }

    /// Consume all remaining characters, if any, and return None if already
    /// at the end, or a source range if with all remaining characters if there
    /// were any.
    fn take_rest<'l>(&'l mut self) -> Option<SourceRange<'s>> {
        let rest_len = self.source.len() - self.position;
        (rest_len > 0).then(|| self.take(rest_len))
    }
}

impl<'s> TokenStream<'s> for Lexer<'s> {
    fn next<'l>(&'l mut self) -> Option<Result<Token<'s>, LexerError<'s>>> {
        let rest = &self.source.as_str()[self.position..];
        if rest.is_empty() {
            return None;
        }

        let (head, ahead1) = {
            let mut source = rest.chars();
            (
                source.next().unwrap(), // safe because non-empty
                source.next(),
            )
        };
        let mut rest = rest.char_indices().skip(1); // skip head but not ahead1
        Some(match (head, ahead1) {
            (' ' | '\n' | '\t' | '\r', _) => {
                let len = rest
                    .find(|(_, char)| {
                        !matches!(char, ' ' | '\n' | '\t' | '\r')
                    })
                    .map(|(idx, _)| idx)
                    .unwrap_or_else(|| self.source.len() - self.position);
                Ok(Token::new(self.take(len), TokenKind::Ws))
            }
            ('(', _) => {
                Ok(Token::new(self.take("(".len()), TokenKind::LeftParen))
            }
            (')', _) => {
                Ok(Token::new(self.take(")".len()), TokenKind::RightParen))
            }
            ('\'', _) => {
                Ok(Token::new(self.take("\'".len()), TokenKind::Quote))
            }
            ('#', Some('\'')) => {
                let after = rest.skip(1).find(|&(_, c)| {
                    !is_identifier_start(c) && !is_identifier_continue(c)
                });
                let range = match after {
                    Some((next_idx, _)) => self.take(next_idx),
                    None => self.take_rest().unwrap(),
                };
                if range.len() <= 2 {
                    return Some(Err(LexerError::EmptyFuncName {
                        fragment: range.of(self.source),
                    }));
                }
                Ok(Token::new(range, TokenKind::FuncIdent))
            }
            (';', _) => {
                let len = rest
                    .find(|&(_, c)| c == '\n')
                    // we don't skip the newline here but exclude it from the comment and
                    // generate another WS token for the newline later (and any follow-up white-space after that)
                    .map(|(nl_idx, _)| nl_idx)
                    .unwrap_or_else(|| self.source.len() - self.position);
                Ok(Token::new(self.take(len), TokenKind::Comment))
            }
            ('"', _) => {
                let mut backslash_prefix = 0;
                for (idx, char) in rest {
                    match char {
                        '\\' => {
                            backslash_prefix += 1;
                        }
                        c if c == head && (backslash_prefix & 1) == 0 => {
                            // end of string
                            let len = idx + "\"".len();
                            let fragment = self.take(len);
                            return Some(Ok(Token::new(
                                fragment,
                                TokenKind::StringLit,
                            )));
                        }
                        _ => {
                            backslash_prefix = 0;
                        }
                    }
                }
                let rest_fragment = self.take_rest().unwrap().of(self.source);
                Err(LexerError::UnterminatedStringLit {
                    fragment: rest_fragment,
                })
            }
            // starts with `0` or `-1` or `.3` (does not support -.3 but -0.3 works)
            (c0, c1)
                if c0.is_ascii_digit()
                    || (matches!(c0, '-' | '+' | '.')
                        && c1
                            .map(|c| c.is_ascii_digit())
                            .unwrap_or(false)) =>
            {
                let next = rest.find(|(_, c)| !c.is_ascii_digit());
                match next {
                    Some((_, '.')) => {
                        // float number with integer part, now followed by the optional fractional part
                        let len = rest
                            .find(|(_, c)| !c.is_ascii_digit())
                            .map(|(idx, _)| idx)
                            .unwrap_or_else(|| {
                                self.source.len() - self.position
                            });
                        let fragment = self.take(len);
                        Ok(Token::new(fragment, TokenKind::FloatLit))
                    }
                    Some((next_idx, _)) => {
                        // int or float without integer part
                        let fragment = self.take(next_idx);
                        let kind = if head == '.' {
                            TokenKind::FloatLit
                        } else {
                            TokenKind::IntLit
                        };
                        Ok(Token::new(fragment, kind))
                    }
                    None => {
                        // no non-digit found, goes all the way to the end
                        // unwrap safe because we checked non-empty at the start// unwrap safe because we checked non-empty at the start
                        let rest_fragment = self.take_rest().unwrap();
                        let kind = if head == '.' {
                            TokenKind::FloatLit
                        } else {
                            TokenKind::IntLit
                        };
                        Ok(Token::new(rest_fragment, kind))
                    }
                }
            }
            (c, _) if is_identifier_start(c) => {
                let len = rest
                    .skip_while(|&(_, c)| is_identifier_continue(c))
                    .next()
                    .map(|(idx, _)| idx)
                    .unwrap_or_else(|| self.source.len() - self.position);
                Ok(Token::new(self.take(len), TokenKind::Ident))
            }
            _ => {
                let unrecognized_range = self.take(1); // consume the unrecognized token
                self.take_rest(); // then consume the rest and ignore to not continue on error
                let fragment = unrecognized_range.of(self.source);
                Err(LexerError::UnrecognizedChar { fragment })
            }
        })
    }

    fn source<'l>(&'l self) -> Source<'s> {
        self.source
    }
}

fn is_identifier_start(c: char) -> bool {
    matches!(
        c,
        '+' | '-'
            | '/'
            | '*'
            | '.'
            | '_'
            | '\\'
            | '<'
            | '>'
            | '='
            | '?'
            | '&'
            | ':'
    ) || c.is_alphabetic()
}

fn is_identifier_continue(c: char) -> bool {
    is_identifier_start(c) || c.is_ascii_digit()
}

#[derive(Debug, Clone)]
pub enum LexerError<'s> {
    UnterminatedStringLit { fragment: Fragment<'s> },
    // can only occur at the very end
    EmptyFuncName { fragment: Fragment<'s> },
    UnrecognizedChar { fragment: Fragment<'s> },
}

impl<'s> fmt::Display for LexerError<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexerError::UnrecognizedChar { fragment } => {
                writeln!(f, "unrecognized character: {}", fragment)?;
                writeln!(f, "{}", fragment.source_context())?;
            }
            LexerError::EmptyFuncName { fragment } => {
                writeln!(f, "missing function name: {}", fragment)?;
                writeln!(f, "{}", fragment.source_context())?;
            }
            LexerError::UnterminatedStringLit { fragment } => {
                writeln!(
                    f,
                    "unterminated string literal opened at: {}",
                    fragment
                )?;
                writeln!(f, "{}", fragment.source_context())?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::source::SourceSet;

    use super::*;

    #[test]
    fn unrecognized() {
        let source_set = SourceSet::new_debug("\0asdf");
        let source = source_set.one();
        let mut lexer = Lexer::new(source);
        let is_unrecognized_char_for_nul =
            match lexer.next().unwrap().unwrap_err() {
                LexerError::UnrecognizedChar { fragment }
                    if fragment.source() == "\0" =>
                {
                    true
                }
                _ => false,
            };
        assert!(is_unrecognized_char_for_nul);
        assert!(lexer.next().is_none());
        assert!(lexer.next().is_none());
        assert!(lexer.next().is_none());
    }

    #[test]
    fn ints() {
        let source_set = SourceSet::new_debug("12 34");
        let source = source_set.one();
        assert_eq!(source.as_str(), "12 34");
        let mut lexer = Lexer::new(source);
        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::IntLit));
        assert_eq!(token.fragment(source).source(), "12");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::IntLit));
        assert_eq!(token.fragment(source).source(), "34");
    }

    #[test]
    fn positive_ints() {
        let source_set = SourceSet::new_debug("+12 +34");
        let source = source_set.one();
        let mut lexer = Lexer::new(source);
        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::IntLit));
        assert_eq!(token.fragment(source).source(), "+12");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::IntLit));
        assert_eq!(token.fragment(source).source(), "+34");
    }

    #[test]
    fn negative_ints() {
        let source_set = SourceSet::new_debug("-12 -34");
        let source = source_set.one();
        let mut lexer = Lexer::new(source);
        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::IntLit));
        assert_eq!(token.fragment(source).source(), "-12");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::IntLit));
        assert_eq!(token.fragment(source).source(), "-34");
    }

    #[test]
    fn sum() {
        let source_set = SourceSet::new_debug("(+ 12\t34)");
        let source = source_set.one();
        let mut lexer = Lexer::new(source);
        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::LeftParen));
        assert_eq!(token.fragment(source).source(), "(");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), "+");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::IntLit));
        assert_eq!(token.fragment(source).source(), "12");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), "\t");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::IntLit));
        assert_eq!(token.fragment(source).source(), "34");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::RightParen));
        assert_eq!(token.fragment(source).source(), ")");
    }

    #[test]
    fn floats() {
        let source_set = SourceSet::new_debug(".1 0.1 0.");
        let source = source_set.one();

        let mut lexer = Lexer::new(source);

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::FloatLit));
        assert_eq!(token.fragment(source).source(), ".1");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::FloatLit));
        assert_eq!(token.fragment(source).source(), "0.1");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::FloatLit));
        assert_eq!(token.fragment(source).source(), "0.");
    }

    #[test]
    fn positive_floats() {
        let source_set = SourceSet::new_debug("+0.1 +0.");
        let source = source_set.one();

        let mut lexer = Lexer::new(source);

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::FloatLit));
        assert_eq!(token.fragment(source).source(), "+0.1");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::FloatLit));
        assert_eq!(token.fragment(source).source(), "+0.");
    }

    #[test]
    fn negative_floats() {
        let source_set = SourceSet::new_debug("-0.1 -0.");
        let source = source_set.one();

        let mut lexer = Lexer::new(source);

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::FloatLit));
        assert_eq!(token.fragment(source).source(), "-0.1");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::FloatLit));
        assert_eq!(token.fragment(source).source(), "-0.");
    }

    #[test]
    fn dot_as_ident_then_num() {
        let source_set = SourceSet::new_debug(". 0");
        let source = source_set.one();

        let mut lexer = Lexer::new(source);

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), ".");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::IntLit));
        assert_eq!(token.fragment(source).source(), "0");
    }

    #[test]
    fn idents() {
        let source_set =
            SourceSet::new_debug("sum product _ * &rest intrinsic:add-2");
        let source = source_set.one();

        let mut lexer = Lexer::new(source);

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), "sum");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), "product");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), "_");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), "*");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), "&rest");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), "intrinsic:add-2");
    }

    #[test]
    fn escaped_ident() {
        let source_set = SourceSet::new_debug("'sum");
        let source = source_set.one();

        let mut lexer = Lexer::new(source);

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Quote));
        assert_eq!(token.fragment(source).source(), "'");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), "sum");
    }

    #[test]
    fn unterminated_empty_string() {
        let source_set = SourceSet::new_debug("\"");
        let source = source_set.one();

        let mut lexer = Lexer::new(source);
        let is_unterminated_string_lit_error =
            match lexer.next().unwrap().unwrap_err() {
                LexerError::UnterminatedStringLit { fragment }
                    if fragment.source() == "\"" =>
                {
                    true
                }
                _ => false,
            };

        assert!(is_unterminated_string_lit_error);
    }

    #[test]
    fn unterminated_string() {
        let source_set = SourceSet::new_debug("\"asdf");
        let source = source_set.one();

        let mut lexer = Lexer::new(source);
        let is_unterminated_string_lit_error =
            match lexer.next().unwrap().unwrap_err() {
                LexerError::UnterminatedStringLit { fragment }
                    if fragment.source().starts_with("\"") =>
                {
                    true
                }
                _ => false,
            };

        assert!(is_unterminated_string_lit_error);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn strings() {
        let source_set = SourceSet::new_debug("\"\" \"\\\\\" \"\\a\"");
        let source = source_set.one();

        let mut lexer = Lexer::new(source);

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::StringLit));
        assert_eq!(token.fragment(source).source(), "\"\"");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::StringLit));
        assert_eq!(token.fragment(source).source(), "\"\\\\\"");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::StringLit));
        assert_eq!(token.fragment(source).source(), "\"\\a\"");
    }

    #[test]
    fn unterminated_variable() {
        let source_set = SourceSet::new_debug("*asdf");
        let source = source_set.one();

        let mut lexer = Lexer::new(source);
        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));

        assert!(lexer.next().is_none());
    }

    #[test]
    fn variables() {
        let source_set = SourceSet::new_debug("** *\\\\* *\\a*");
        let source = source_set.one();

        let mut lexer = Lexer::new(source);

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), "**");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), "*\\\\*");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), "*\\a*");
    }

    #[test]
    fn comment_only() {
        let source_set = SourceSet::new_debug("; this is a comment");
        let source = source_set.one();

        let mut lexer = Lexer::new(source);

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Comment));
        assert_eq!(token.fragment(source).source(), source.as_str());

        assert!(lexer.next().is_none());
    }

    #[test]
    fn comment_after() {
        let source_set = SourceSet::new_debug(
            "1; this is a comment\n \"asdf\" ; another one\n",
        );
        let source = source_set.one();

        let mut lexer = Lexer::new(source);

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::IntLit));
        assert_eq!(token.fragment(source).source(), "1");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Comment));
        assert_eq!(token.fragment(source).source(), "; this is a comment");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), "\n ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::StringLit));
        assert_eq!(token.fragment(source).source(), "\"asdf\"");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Comment));
        assert_eq!(token.fragment(source).source(), "; another one");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), "\n");

        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_remove_if_not() {
        let source_set = SourceSet::new_debug(
            "(remove-if-not (lambda (x) (< x 5)) '(0 10))",
        );
        let source = source_set.one();

        let mut lexer = Lexer::new(source);

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::LeftParen));
        assert_eq!(token.fragment(source).source(), "(");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), "remove-if-not");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::LeftParen));
        assert_eq!(token.fragment(source).source(), "(");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), "lambda");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::LeftParen));
        assert_eq!(token.fragment(source).source(), "(");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), "x");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::RightParen));
        assert_eq!(token.fragment(source).source(), ")");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::LeftParen));
        assert_eq!(token.fragment(source).source(), "(");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), "<");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), "x");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::IntLit));
        assert_eq!(token.fragment(source).source(), "5");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::RightParen));
        assert_eq!(token.fragment(source).source(), ")");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::RightParen));
        assert_eq!(token.fragment(source).source(), ")");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Quote));
        assert_eq!(token.fragment(source).source(), "'");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::LeftParen));
        assert_eq!(token.fragment(source).source(), "(");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::IntLit));
        assert_eq!(token.fragment(source).source(), "0");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::IntLit));
        assert_eq!(token.fragment(source).source(), "10");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::RightParen));
        assert_eq!(token.fragment(source).source(), ")");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::RightParen));
        assert_eq!(token.fragment(source).source(), ")");
    }

    #[test]
    fn lex_apply() {
        let source_set = SourceSet::new_debug(
            "(apply #'concatenate '(strings \"A\" \"b\"))",
        );
        let source = source_set.one();

        let mut lexer = Lexer::new(source);

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::LeftParen));
        assert_eq!(token.fragment(source).source(), "(");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), "apply");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert_eq!(token.fragment(source).source(), "#'concatenate");
        assert!(matches!(token.kind(), TokenKind::FuncIdent));

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Quote));
        assert_eq!(token.fragment(source).source(), "'");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::LeftParen));
        assert_eq!(token.fragment(source).source(), "(");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ident));
        assert_eq!(token.fragment(source).source(), "strings");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::StringLit));
        assert_eq!(token.fragment(source).source(), "\"A\"");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::Ws));
        assert_eq!(token.fragment(source).source(), " ");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::StringLit));
        assert_eq!(token.fragment(source).source(), "\"b\"");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::RightParen));
        assert_eq!(token.fragment(source).source(), ")");

        let token = lexer.next().unwrap().unwrap();
        assert!(matches!(token.kind(), TokenKind::RightParen));
        assert_eq!(token.fragment(source).source(), ")");
    }
}
