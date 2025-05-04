use crate::parse::{AstNode, Atom, List};
use crate::source::{Source, SourceRange};

pub struct ExpressionSource<'s> {
    /// Source file associated with the expression
    source: Source<'s>,
    /// AST node associated with the expression.
    ast: AstNode<'s>,
}

pub enum ExpressionKind {
    Literal,
    Name,
    Invocation,
}

enum Expression<'a, 's> {
    FunctionDefinition {
        name: &'a Atom<'s>,
        params: &'a List<'s>,
    },
    FunctionCall(List<'s>),
    Name(NameExpression<'a, 's>),
    Literal(AstNode<'s>),
}

struct NameExpression<'a, 's> {
    atom: &'a Atom<'s>,
}

pub struct ExpressionAnalyzer;

impl ExpressionAnalyzer {
    pub fn analyze_ast<'s>(
        &mut self,
        source: Source<'s>,
        ast: &[AstNode<'s>],
    ) -> Result<(), ExpressionAnalyzerError> {
        for node in ast {
            self.analyze_node(source, node)?;
        }
        Ok(())
    }

    fn analyze_node<'s>(
        &mut self,
        source: Source<'s>,
        node: &AstNode<'s>,
    ) -> Result<(), ExpressionAnalyzerError> {
        todo!()
    }
}

enum ExpressionAnalyzerError {}
