use std::fmt;

use crate::parse::{ast::Atom, token::TokenKind};

use super::{
    ahead::LookaheadStream,
    ast::{Ast, AstNode, List, Quoted},
    ignore::Ignore,
    lexer::{Lexer, LexerError},
    stream::TokenStream as _,
    token::Token,
};

use crate::source::{SourceRange, Source};

type InnerStream<'s> = LookaheadStream<'s, Ignore<Lexer<'s>, 2>, 1>;

pub struct Parser<'s> {
    lexer: InnerStream<'s>,
}

impl<'s> Parser<'s> {
    pub fn new(source: Source<'s>) -> Self {
        Self {
            lexer: LookaheadStream::new(Ignore::new(
                Lexer::new(source),
                [TokenKind::Ws, TokenKind::Comment],
            )),
        }
    }

    /// Parses a file containing zero or more things, e.g. a sequence of
    /// function definitions.
    ///
    /// This is intended as the entry rule to parse a source code document.
    ///
    /// Only the first call will yield an AST. Subsequent calls will yield an
    /// empty vector because the underlying lexer will be exhausted.
    pub fn parse<'a>(
        &'a mut self,
    ) -> Result<Ast<'s>, ParserError<'s>> {
        let mut items = vec![];
        while let Some(Ok(_)) = self.lexer.max_lookahead()[0] {
            items.push(self.parse_list()?);
        }
        Ok(Ast::new(self.lexer.source(), items))
    }

    /// Parses a single list or atom.
    ///
    /// If the end of string is reached, an error is returned.
    fn parse_one<'a>(&'a mut self) -> Result<AstNode<'s>, ParserError<'s>> {
        let source = self.lexer.source();
        let [ahead0] = self.lexer.max_lookahead();
        let token0 = ahead0
            .ok_or_else(|| ParserError::unexpected_end(source))?
            .map_err(|e| ParserError::lexer_error(source, e.clone()))?;
        match token0.kind() {
            TokenKind::Quote => self.parse_quoted(),
            TokenKind::LeftParen => self.parse_list(),
            TokenKind::FloatLit
            | TokenKind::IntLit
            | TokenKind::StringLit
            | TokenKind::Ident => Ok(Atom::new(self.lexer.next().unwrap().unwrap())),
            TokenKind::Comment | TokenKind::Ws => unreachable!(),
            _ => Err(ParserError::mismatched_token(
                source,
                self.lexer.next().unwrap().unwrap(),
            )),
        }
    }

    fn parse_list<'a>(&'a mut self) -> Result<AstNode<'s>, ParserError<'s>> {
        let source = self.lexer.source();
        let opening = self
            .lexer
            .next()
            .ok_or_else(|| ParserError::unexpected_end(source))?
            .map_err(|e| ParserError::lexer_error(source, e.clone()))?;
        match opening.kind() {
            TokenKind::LeftParen => {}
            _ => return Err(ParserError::mismatched_token(source, opening)),
        }

        let mut closing = None;
        let mut items = vec![];
        while let Some(Ok(token)) = self.lexer.max_lookahead()[0] {
            if matches!(token.kind(), TokenKind::RightParen) {
                // end of list found, consume the closing parenthesis
                closing = self.lexer.next();
                break;
            } else {
                // not end of list yet, parse as an entry of the list
                items.push(self.parse_one()?)
            }
        }

        let closing = closing
            .ok_or_else(|| {
                ParserError::unbalanced_parenthesis(source, opening.clone())
            })?
            .unwrap();

        Ok(List::new(
            SourceRange::union_two(
                opening.source_range(),
                closing.source_range(),
            ),
            items,
        ))
    }

    fn parse_quoted<'a>(
        &'a mut self,
    ) -> Result<AstNode<'s>, ParserError<'s>> {
        let source = self.lexer.source();
        let tick = self
            .lexer
            .next()
            .ok_or_else(|| ParserError::unexpected_end(source))?
            .map_err(|e| ParserError::lexer_error(source, e.clone()))?;
        match (tick.kind(), tick.fragment(self.lexer.source()).source()) {
            (TokenKind::Quote, "'") => {}
            _ => return Err(ParserError::mismatched_token(source, tick)),
        }

        let quoted = self.parse_one()?;
        
        Ok(Quoted::new(
            SourceRange::union_two(
                tick.source_range(),
                quoted.source_range(),
            ),
            quoted
        ))
    }
}

#[derive(Debug)]
pub enum ParserErrorDetails<'s> {
    LexerError { error: LexerError<'s> },
    MismatchedToken { token: Token<'s> },
    UnbalancedParenthesis { opening: Token<'s> },
    UnexpectedEnd,
}

#[derive(Debug)]
pub struct ParserError<'s> {
    source: Source<'s>,
    details: ParserErrorDetails<'s>,
}

impl<'s> ParserError<'s> {
    pub fn lexer_error(source: Source<'s>, error: LexerError<'s>) -> Self {
        Self {
            source,
            details: ParserErrorDetails::LexerError { error },
        }
    }

    pub fn mismatched_token(source: Source<'s>, token: Token<'s>) -> Self {
        Self {
            source,
            details: ParserErrorDetails::MismatchedToken { token },
        }
    }

    pub fn unbalanced_parenthesis(
        source: Source<'s>,
        opening: Token<'s>,
    ) -> Self {
        Self {
            source,
            details: ParserErrorDetails::UnbalancedParenthesis { opening },
        }
    }

    pub fn unexpected_end(source: Source<'s>) -> Self {
        Self {
            source,
            details: ParserErrorDetails::UnexpectedEnd,
        }
    }
}

impl<'s> fmt::Display for ParserError<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.details {
            ParserErrorDetails::LexerError { ref error } => {
                write!(f, "{}", error)?;
            }
            ParserErrorDetails::MismatchedToken { ref token } => {
                writeln!(
                    f,
                    "unexpected token: {}",
                    token.fragment(self.source)
                )?;
                writeln!(
                    f,
                    "{}",
                    token.fragment(self.source).source_context()
                )?;
            }
            ParserErrorDetails::UnbalancedParenthesis { ref opening } => {
                writeln!(
                    f,
                    "list never closed: {}",
                    opening.fragment(self.source)
                )?;
                writeln!(
                    f,
                    "{}",
                    opening.fragment(self.source).source_context()
                )?;
            }
            ParserErrorDetails::UnexpectedEnd => {
                writeln!(f, "unexpected end")?;
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
    fn number() {
        let source_set = SourceSet::new_debug("42");
        let source = source_set.one();
        let mut parser = Parser::new(source);
        let node = parser.parse_one().unwrap();
        let node = node.atom().unwrap();
        assert!(matches!(node.token().kind(), TokenKind::IntLit));
    }

    #[test]
    fn empty_list() {
        let source_set = SourceSet::new_debug("()");
        let source = source_set.one();
        let mut parser = Parser::new(source);
        let node = parser.parse().unwrap().into_iter().next().unwrap();
        let node = node.list().unwrap();
        assert_eq!(node.source_range().of(source).source(), "()");
    }

    #[test]
    fn one_int_list() {
        let source_set = SourceSet::new_debug("( 42)");
        let source = source_set.one();
        let mut parser = Parser::new(source);
        let node = parser.parse().unwrap().into_iter().next().unwrap();
        let node = node.list().unwrap();
        assert_eq!(node.source_range().of(source).source(), "( 42)");
        let elements = node.elements();
        assert_eq!(elements.len(), 1);
        let el = elements[0].source_range().of(source);
        let el = el.source();
        assert_eq!(el, "42")
    }

    #[test]
    fn two_ints_list_list() {
        let source_set = SourceSet::new_debug("( ( 42 128)\t)");
        let source = source_set.one();
        let mut parser = Parser::new(source);
        let node = parser.parse().unwrap().into_iter().next().unwrap();
        let node = node.list().unwrap();
        assert_eq!(node.source_range().of(source).source(), "( ( 42 128)\t)");
        let elements = node.elements();
        assert_eq!(elements.len(), 1);
        let node = elements[0].list().unwrap();
        assert_eq!(node.source_range().of(source).source(), "( 42 128)");
        let elements = node.elements();
        assert_eq!(elements.len(), 2);
        let int0 = elements[0].atom().unwrap();
        let int1 = elements[1].atom().unwrap();
        assert_eq!(int0.source_range().of(source).source(), "42");
        assert_eq!(int1.source_range().of(source).source(), "128");
    }

    #[test]
    fn parse_remove_if_not() {
        let source_set = SourceSet::new_debug("(remove-if-not (lambda (x) (< x 5)) '(0 10))");
        let source = source_set.one();
        let mut parser = Parser::new(source);
        let remove = parser.parse().unwrap().into_iter().next().unwrap();
        let remove = remove.list().unwrap();
        let remove = remove.elements();

        assert_eq!(
            remove[0]
                .atom()
                .unwrap()
                .source_range()
                .of(source)
                .source(),
            "remove-if-not"
        );

        let lambda = remove[1].list().unwrap();
        let lambda = lambda.elements();

        assert_eq!(
            lambda[0]
                .atom()
                .unwrap()
                .source_range()
                .of(source)
                .source(),
            "lambda"
        );

        let lambda_args = lambda[1].list().unwrap();
        let lambda_args = lambda_args.elements();
        assert_eq!(lambda_args.len(), 1);
        assert_eq!(
            lambda_args[0]
                .atom()
                .unwrap()
                .source_range()
                .of(source)
                .source(),
            "x"
        );

        let lt = lambda[2].list().unwrap();
        let lt = lt.elements();
        assert_eq!(
            lt[0].atom().unwrap().source_range().of(source).source(),
            "<"
        );
        assert_eq!(
            lt[1].atom().unwrap().source_range().of(source).source(),
            "x"
        );
        assert_eq!(
            lt[2].atom().unwrap().source_range().of(source).source(),
            "5"
        );

        let quoted_list = &remove[2].quoted().unwrap();
        assert_eq!(quoted_list.source_range().of(source).source(), "'(0 10)");
        assert_eq!(remove[2].source_range().of(source).source(), "'(0 10)");

        let quoted_list = quoted_list.quoted().list().unwrap().elements();
        assert_eq!(quoted_list.len(), 2);
        assert_eq!(
            quoted_list[0]
                .atom()
                .unwrap()
                .source_range()
                .of(source)
                .source(),
            "0"
        );
        assert_eq!(
            quoted_list[1]
                .atom()
                .unwrap()
                .source_range()
                .of(source)
                .source(),
            "10"
        );
    }

    #[test]
    fn parse_two_lists() {
        let source_set = SourceSet::new_debug("(defparameter *x* 1)\n(defparameter *y* 2)\n");
        let source = source_set.one();

        let mut parser = Parser::new(source);
        let two = parser.parse().unwrap();
        assert_eq!(two.len(), 2);

        assert!(two.root_nodes()[0].list().is_some());
        assert!(two.root_nodes()[1].list().is_some());
    }
}
