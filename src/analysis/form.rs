use std::fmt;

use crate::{parse::{AstNode, Atom, List, TokenKind}, source::Source};

pub enum Form<'s, 't> {
    /// A variable (not function) name.
    Name(Name<'s, 't>),
    Constant(Constant<'s, 't>),
    IfForm(IfForm<'s, 't>),
    Call(Call<'s, 't>)
}

pub struct Name<'s, 't> {
    source: Source<'s>,
    ident: &'t Atom<'s>
}

pub struct Constant<'s, 't> {
    source: Source<'s>,
    node: &'t AstNode<'s>
}

pub struct IfForm<'s, 't> {
    source: Source<'s>,
    test: Box<Form<'s, 't>>,
    then_form: Box<Form<'s, 't>>,
    else_form: Option<Box<Form<'s, 't>>>
}

pub struct Call<'s, 't> {
    source: Source<'s>,
    function: &'t Atom<'s>,
    args: Vec<Form<'s, 't>>
}

impl<'s, 't> Form<'s, 't> {
    pub fn extract(source: Source<'s>, form: &'t AstNode<'s>) -> Result<Form<'s, 't>, FormError<'s, 't>> {
        Ok(match form {
            AstNode::Quoted(quoted) => Self::Constant(Constant {
                source,
                node: quoted.quoted()
            }),
            AstNode::Atom(atom) if matches!(atom.token().kind(), TokenKind::Ident) => Self::Name(Name { source, ident: atom }),
            AstNode::Atom(_) => Self::Constant(Constant { source, node: form }),
            AstNode::List(list) if list.elements().is_empty() => Self::Constant(Constant { source, node: form }),
            AstNode::List(non_empty) => {
                if let Some(if_form) = IfForm::extract_assume_nonempty(source, non_empty)? {
                    return Ok(Form::IfForm(if_form));
                }
                Form::Call(Call::extract_assume_nonempty(source, non_empty)?)
            }
        })
    }

    pub fn name(&self) -> Option<&Name<'s, 't>> {
        match self {
            Self::Name(name) => Some(name),
            _ => None
        }
    }

    pub fn constant(&self) -> Option<&Constant<'s, 't>> {
        match self {
            Self::Constant(c) => Some(c),
            _ => None
        }
    }

    pub fn if_form(&self) -> Option<&IfForm<'s, 't>> {
        match self {
            Self::IfForm(i) => Some(i),
            _ => None
        }
    }

    pub fn call(&self) -> Option<&Call<'s, 't>> {
        match self {
            Self::Call(c) => Some(c),
            _ => None
        }
    }
}

impl<'s, 't> Name<'s, 't> {
    pub fn ident(&self) -> &'t Atom<'s> {
        self.ident
    }

    pub fn as_str(&self) -> &'s str {
        self.ident.fragment(self.source).source()
    }
}

impl<'s, 't> Constant<'s, 't> {
    pub fn source(&self) -> Source<'s> {
        self.source
    }

    pub fn node(&self) -> &'t AstNode<'s> {
        self.node
    }
}

impl<'s, 't> IfForm<'s, 't> {
    fn extract_assume_nonempty(source: Source<'s>, form: &'t List<'s>) -> Result<Option<IfForm<'s, 't>>, FormError<'s, 't>> {
        let mut elements = form.elements().iter();

        let head = elements.next().unwrap();
        let is_if = match head {
            AstNode::Atom(first)
                if first.source_range().of(source).source() == "if" =>
            {
                true
            }
            _ => false,
        };
        if !is_if {
            // ignore root-level directive that is not defun
            return Ok(None);
        }

        let test_form = elements.next()
            .ok_or_else(|| FormError::IfMissingTest { source, form })?;
        let test_form = Form::extract(source, test_form)?;

        let then_form = elements.next()
            .ok_or_else(|| FormError::IfMissingTest { source, form })?;
        let then_form = Form::extract(source, then_form)?;

        let else_form = match elements.next() {
            None => None,
            Some(else_form) => Some(Box::new(Form::extract(source, else_form)?))
        };

        match elements.next() {
            None => {},
            Some(extraneous) => return Err(FormError::IfExtraneousForm { source, form: extraneous })
        };

        Ok(Some(Self {
            source,
            test: Box::new(test_form),
            then_form: Box::new(then_form),
            else_form
        }))
    }

    pub fn source(&self) -> Source<'s> {
        self.source
    }

    pub fn test_form(&self) -> &Form<'s, 't> {
        &self.test
    }

    pub fn then_form(&self) -> &Form<'s, 't> {
        &self.then_form
    }

    pub fn else_form(&self) -> Option<&Form<'s, 't>> {
        self.else_form.as_ref().map(|e| e.as_ref())
    }
}

impl<'s, 't> Call<'s, 't> {
    /// The call form matches everything else, so we have to try it last.
    /// 
    /// # Panics
    /// Panics if given an empty list.
    fn extract_assume_nonempty(source: Source<'s>, form: &'t List<'s>) -> Result<Call<'s, 't>, FormError<'s, 't>> {
        let mut elements = form.elements().iter();

        let head = elements.next().unwrap()
            .atom()
            .ok_or_else(|| FormError::CallTargetNotConstant { source, form: &form.elements()[0] })?;

        let arg_asts = &form.elements()[1..];
        let arg_forms = arg_asts.into_iter()
            .map(|a| Form::extract(source, a))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Call { source, function: head, args: arg_forms })
    }

    pub fn source(&self) -> Source<'s> {
        self.source
    }


    pub fn function(&self) -> &'t Atom<'s> {
        self.function
    }

    pub fn args(&self) -> &[Form<'s, 't>] {
        &self.args
    }
}

#[derive(Debug)]
pub enum FormError<'s, 't> {
    IfMissingTest {
        source: Source<'s>,
        form: &'t List<'s>
    },
    IfMissingThen {
        source: Source<'s>,
        form: &'t List<'s>
    },
    IfExtraneousForm {
        source: Source<'s>,
        form: &'t AstNode<'s>
    },
    CallTargetNotConstant {
        source: Source<'s>,
        form: &'t AstNode<'s>
    }
}

impl<'s, 't> fmt::Display for FormError<'s, 't> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormError::IfMissingTest { source, form } => {
                writeln!(f, "if form is missing the test part:")?;
                writeln!(f, "{}", form.fragment(*source).source_context())
            },
            FormError::IfMissingThen { source, form } => {
                writeln!(f, "if form is missing the then part:")?;
                writeln!(f, "{}", form.fragment(*source).source_context())
            },
            FormError::IfExtraneousForm { source, form } => {
                writeln!(f, "if form has extra element after then part:")?;
                writeln!(f, "{}", form.fragment(*source).source_context())
            },
            FormError::CallTargetNotConstant { source, form } => {
                writeln!(f, "target of function call is not constant:")?;
                writeln!(f, "{}", form.fragment(*source).source_context())
            }
        }
    }
}