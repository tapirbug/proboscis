use std::{slice, vec};

use super::ast::Ast;

pub struct AstSet<'s> {
    asts: Vec<Ast<'s>>,
}

impl<'s> AstSet<'s> {
    pub fn new(asts: Vec<Ast<'s>>) -> Self {
        Self { asts }
    }

    pub fn iter(&self) -> slice::Iter<Ast<'s>> {
        self.asts.iter()
    }
}

impl<'s> FromIterator<Ast<'s>> for AstSet<'s> {
    fn from_iter<T: IntoIterator<Item = Ast<'s>>>(iter: T) -> Self {
        Self {
            asts: iter.into_iter().collect(),
        }
    }
}

impl<'s> IntoIterator for AstSet<'s> {
    type IntoIter = vec::IntoIter<Ast<'s>>;
    type Item = Ast<'s>;

    fn into_iter(self) -> Self::IntoIter {
        self.asts.into_iter()
    }
}
