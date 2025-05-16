use std::fmt;

use crate::{
    parse::{AstNode, Atom, List, TokenKind},
    source::Source,
};

pub enum Form<'s, 't> {
    /// A variable (not function) name.
    Name(Name<'s, 't>),
    FunctionName(FunctionName<'s, 't>),
    Constant(Constant<'s, 't>),
    LetForm(LetForm<'s, 't>),
    IfForm(IfForm<'s, 't>),
    AndForm(AndForm<'s, 't>),
    OrForm(OrForm<'s, 't>),
    Call(Call<'s, 't>),
    Apply(Apply<'s, 't>),
}

pub struct Name<'s, 't> {
    source: Source<'s>,
    ident: &'t Atom<'s>,
}

pub struct FunctionName<'s, 't> {
    source: Source<'s>,
    name: &'t Atom<'s>,
}

pub struct Constant<'s, 't> {
    source: Source<'s>,
    node: &'t AstNode<'s>,
}

pub struct IfForm<'s, 't> {
    source: Source<'s>,
    test: Box<Form<'s, 't>>,
    then_form: Box<Form<'s, 't>>,
    else_form: Option<Box<Form<'s, 't>>>,
}

pub struct AndForm<'s, 't> {
    source: Source<'s>,
    forms: Vec<Form<'s, 't>>,
}

pub struct OrForm<'s, 't> {
    source: Source<'s>,
    forms: Vec<Form<'s, 't>>,
}

pub struct Call<'s, 't> {
    source: Source<'s>,
    function: &'t Atom<'s>,
    args: Vec<Form<'s, 't>>,
}

pub struct Binding<'s, 't> {
    name: &'t Atom<'s>,
    value: Form<'s, 't>,
}

pub struct LetForm<'s, 't> {
    source: Source<'s>,
    bindings: Vec<Binding<'s, 't>>,
    body: Vec<Form<'s, 't>>,
}

pub struct Apply<'s, 't> {
    source: Source<'s>,
    /// Something that can be resolved to a function.
    function: Box<Form<'s, 't>>,
    /// Arg list can be calculated at runtime.
    args: Box<Form<'s, 't>>,
}

impl<'s, 't> Form<'s, 't> {
    pub fn extract(
        source: Source<'s>,
        form: &'t AstNode<'s>,
    ) -> Result<Form<'s, 't>, FormError<'s, 't>> {
        Ok(match form {
            AstNode::Quoted(quoted) => Self::Constant(Constant {
                source,
                node: quoted.quoted(),
            }),
            AstNode::Atom(atom)
                if matches!(atom.token().kind(), TokenKind::Ident) =>
            {
                Self::Name(Name {
                    source,
                    ident: atom,
                })
            }
            AstNode::Atom(atom)
                if matches!(atom.token().kind(), TokenKind::FuncIdent) =>
            {
                Self::FunctionName(FunctionName { source, name: atom })
            }
            AstNode::Atom(_) => {
                Self::Constant(Constant { source, node: form })
            }
            AstNode::List(list) if list.elements().is_empty() => {
                Self::Constant(Constant { source, node: form })
            }
            AstNode::List(non_empty) => {
                if let Some(if_form) =
                    IfForm::extract_assume_nonempty(source, non_empty)?
                {
                    return Ok(Form::IfForm(if_form));
                }
                if let Some(and_form) =
                    AndForm::extract_assume_nonempty(source, non_empty)?
                {
                    return Ok(Form::AndForm(and_form));
                }
                if let Some(or_form) =
                    OrForm::extract_assume_nonempty(source, non_empty)?
                {
                    return Ok(Form::OrForm(or_form));
                }
                if let Some(let_form) =
                    LetForm::extract_assume_nonempty(source, non_empty)?
                {
                    return Ok(Form::LetForm(let_form));
                }
                if let Some(apply_static) =
                    Apply::extract_assume_nonempty(source, non_empty)?
                {
                    return Ok(Form::Apply(apply_static));
                }
                Form::Call(Call::extract_assume_nonempty(source, non_empty)?)
            }
        })
    }

    pub fn name(&self) -> Option<&Name<'s, 't>> {
        match self {
            Self::Name(name) => Some(name),
            _ => None,
        }
    }

    pub fn function_name(&self) -> Option<&FunctionName<'s, 't>> {
        match self {
            Self::FunctionName(name) => Some(name),
            _ => None,
        }
    }

    pub fn constant(&self) -> Option<&Constant<'s, 't>> {
        match self {
            Self::Constant(c) => Some(c),
            _ => None,
        }
    }

    pub fn if_form(&self) -> Option<&IfForm<'s, 't>> {
        match self {
            Self::IfForm(i) => Some(i),
            _ => None,
        }
    }

    pub fn call(&self) -> Option<&Call<'s, 't>> {
        match self {
            Self::Call(c) => Some(c),
            _ => None,
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

impl<'s, 't> FunctionName<'s, 't> {
    /// Full identifier token inlcuding #'.
    pub fn ident(&self) -> &'t Atom<'s> {
        &self.name
    }

    /// The name excluding #' and whitespace.
    pub fn as_str(&self) -> &'s str {
        self.name.fragment(self.source).source()["#'".len()..].trim()
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
    fn extract_assume_nonempty(
        source: Source<'s>,
        form: &'t List<'s>,
    ) -> Result<Option<IfForm<'s, 't>>, FormError<'s, 't>> {
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

        let test_form = elements
            .next()
            .ok_or_else(|| FormError::IfMissingTest { source, form })?;
        let test_form = Form::extract(source, test_form)?;

        let then_form = elements
            .next()
            .ok_or_else(|| FormError::IfMissingTest { source, form })?;
        let then_form = Form::extract(source, then_form)?;

        let else_form = match elements.next() {
            None => None,
            Some(else_form) => {
                Some(Box::new(Form::extract(source, else_form)?))
            }
        };

        match elements.next() {
            None => {}
            Some(extraneous) => {
                return Err(FormError::IfExtraneousForm {
                    source,
                    form: extraneous,
                });
            }
        };

        Ok(Some(Self {
            source,
            test: Box::new(test_form),
            then_form: Box::new(then_form),
            else_form,
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

impl<'s, 't> AndForm<'s, 't> {
    fn extract_assume_nonempty(
        source: Source<'s>,
        form: &'t List<'s>,
    ) -> Result<Option<AndForm<'s, 't>>, FormError<'s, 't>> {
        let mut elements = form.elements().iter();

        let head = elements.next().unwrap();
        let is_and = match head {
            AstNode::Atom(first)
                if first.source_range().of(source).source() == "and" =>
            {
                true
            }
            _ => false,
        };
        if !is_and {
            return Ok(None);
        }

        let forms = elements
            .map(|a| Form::extract(source, a))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Some(AndForm { source, forms }))
    }

    pub fn source(&self) -> Source<'s> {
        self.source
    }

    pub fn forms(&self) -> &[Form<'s, 't>] {
        &self.forms
    }
}

impl<'s, 't> OrForm<'s, 't> {
    fn extract_assume_nonempty(
        source: Source<'s>,
        form: &'t List<'s>,
    ) -> Result<Option<OrForm<'s, 't>>, FormError<'s, 't>> {
        let mut elements = form.elements().iter();

        let head = elements.next().unwrap();
        let is_or = match head {
            AstNode::Atom(first)
                if first.source_range().of(source).source() == "or" =>
            {
                true
            }
            _ => false,
        };
        if !is_or {
            return Ok(None);
        }

        let forms = elements
            .map(|a| Form::extract(source, a))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Some(OrForm { source, forms }))
    }

    pub fn source(&self) -> Source<'s> {
        self.source
    }

    pub fn forms(&self) -> &[Form<'s, 't>] {
        &self.forms
    }
}

impl<'s, 't> Apply<'s, 't> {
    fn extract_assume_nonempty(
        source: Source<'s>,
        form: &'t List<'s>,
    ) -> Result<Option<Apply<'s, 't>>, FormError<'s, 't>> {
        let mut elements = form.elements().iter();

        let head = elements.next().unwrap();
        let is_apply = match head {
            AstNode::Atom(first)
                if first.source_range().of(source).source() == "apply" =>
            {
                true
            }
            _ => false,
        };
        if !is_apply {
            // ignore root-level directive that is not defun
            return Ok(None);
        }
        let head = head.atom().unwrap();

        let func =
            Form::extract(
                source,
                elements.next().ok_or_else(|| {
                    FormError::ApplyStaticTooShort { source, atom: head }
                })?,
            )?;
        let args =
            elements
                .next()
                .ok_or_else(|| FormError::ApplyStaticTooShort {
                    source,
                    atom: head,
                })?;
        let args = Form::extract(source, args)?;

        if elements.next().is_some() {
            return Err(FormError::ApplyStaticTooLong { source, atom: head });
        }

        Ok(Some(Apply {
            source,
            function: Box::new(func),
            args: Box::new(args),
        }))
    }

    pub fn source(&self) -> Source<'s> {
        self.source
    }

    pub fn function<'a>(&'a self) -> &'a Form<'s, 't> {
        self.function.as_ref()
    }

    pub fn args<'a>(&'a self) -> &'a Form<'s, 't> {
        self.args.as_ref()
    }
}

impl<'s, 't> LetForm<'s, 't> {
    fn extract_assume_nonempty(
        source: Source<'s>,
        form: &'t List<'s>,
    ) -> Result<Option<LetForm<'s, 't>>, FormError<'s, 't>> {
        let mut elements = form.elements().iter();

        let head = elements.next().unwrap();
        let is_let = match head {
            AstNode::Atom(first)
                if first.source_range().of(source).source() == "let" =>
            {
                true
            }
            _ => false,
        };
        if !is_let {
            // ignore root-level directive that is not defun
            return Ok(None);
        }
        let head = head.atom().unwrap();

        let bindings =
            elements
                .next()
                .ok_or_else(|| FormError::LetMissingBindings {
                    source,
                    atom: head,
                })?;
        let bindings =
            bindings
                .list()
                .ok_or_else(|| FormError::LetBindingsNotList {
                    source,
                    atom: bindings,
                })?;
        let mut bindings_parsed = vec![];
        for binding in bindings.elements() {
            let binding = binding.list().ok_or_else(|| {
                FormError::LetBindingEntryNotList {
                    source,
                    atom: binding,
                }
            })?;
            if binding.elements().len() < 2 {
                return Err(FormError::LetBindingEntryTooShort {
                    source,
                    atom: binding,
                });
            }
            if binding.elements().len() > 2 {
                return Err(FormError::LetBindingEntryTooLong {
                    source,
                    atom: binding,
                });
            }
            let name = binding.elements()[0]
                .atom()
                .filter(|a| a.token().kind() == TokenKind::Ident)
                .ok_or_else(|| FormError::LetBindingVarNotIdent {
                    source,
                    atom: &binding.elements()[0],
                })?;
            let value = Form::extract(source, &binding.elements()[1])?;
            bindings_parsed.push(Binding { name, value });
        }

        let body = &form.elements()[2..];
        if body.is_empty() {
            return Err(FormError::LetMissingBody { source, atom: head });
        }
        let body = body
            .iter()
            .map(|f| Form::extract(source, f))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Some(LetForm {
            source,
            bindings: bindings_parsed,
            body,
        }))
    }

    pub fn bindings(&self) -> &[Binding<'s, 't>] {
        &self.bindings
    }

    pub fn body(&self) -> &[Form<'s, 't>] {
        &self.body
    }
}

impl<'s, 't> Binding<'s, 't> {
    pub fn name(&self) -> &'t Atom<'s> {
        self.name
    }

    pub fn value(&self) -> &Form<'s, 't> {
        &self.value
    }
}

impl<'s, 't> Call<'s, 't> {
    /// The call form matches everything else, so we have to try it last.
    ///
    /// # Panics
    /// Panics if given an empty list.
    fn extract_assume_nonempty(
        source: Source<'s>,
        form: &'t List<'s>,
    ) -> Result<Call<'s, 't>, FormError<'s, 't>> {
        let mut elements = form.elements().iter();

        let head = elements.next().unwrap().atom().ok_or_else(|| {
            FormError::CallTargetNotConstant {
                source,
                form: &form.elements()[0],
            }
        })?;

        let arg_asts = &form.elements()[1..];
        let arg_forms = arg_asts
            .into_iter()
            .map(|a| Form::extract(source, a))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Call {
            source,
            function: head,
            args: arg_forms,
        })
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
        form: &'t List<'s>,
    },
    IfMissingThen {
        source: Source<'s>,
        form: &'t List<'s>,
    },
    IfExtraneousForm {
        source: Source<'s>,
        form: &'t AstNode<'s>,
    },
    LetMissingBindings {
        source: Source<'s>,
        atom: &'t Atom<'s>,
    },
    LetBindingsNotList {
        source: Source<'s>,
        atom: &'t AstNode<'s>,
    },
    LetBindingEntryNotList {
        source: Source<'s>,
        atom: &'t AstNode<'s>,
    },
    LetBindingEntryTooShort {
        source: Source<'s>,
        atom: &'t List<'s>,
    },
    LetBindingEntryTooLong {
        source: Source<'s>,
        atom: &'t List<'s>,
    },
    LetBindingVarNotIdent {
        source: Source<'s>,
        atom: &'t AstNode<'s>,
    },
    LetMissingBody {
        source: Source<'s>,
        atom: &'t Atom<'s>,
    },
    CallTargetNotConstant {
        source: Source<'s>,
        form: &'t AstNode<'s>,
    },
    ApplyStaticTooShort {
        source: Source<'s>,
        atom: &'t Atom<'s>,
    },
    ApplyStaticTooLong {
        source: Source<'s>,
        atom: &'t Atom<'s>,
    },
}

impl<'s, 't> fmt::Display for FormError<'s, 't> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormError::IfMissingTest { source, form } => {
                writeln!(f, "if form is missing the test part:")?;
                writeln!(f, "{}", form.fragment(*source).source_context())
            }
            FormError::IfMissingThen { source, form } => {
                writeln!(f, "if form is missing the then part:")?;
                writeln!(f, "{}", form.fragment(*source).source_context())
            }
            FormError::IfExtraneousForm { source, form } => {
                writeln!(f, "if form has extra element after then part:")?;
                writeln!(f, "{}", form.fragment(*source).source_context())
            }
            FormError::LetMissingBindings { source, atom } => {
                writeln!(f, "let form does not define a bindings list:")?;
                writeln!(f, "{}", atom.fragment(*source).source_context())
            }
            FormError::LetMissingBody { source, atom } => {
                writeln!(f, "let form does not define a body:")?;
                writeln!(f, "{}", atom.fragment(*source).source_context())
            }
            FormError::CallTargetNotConstant { source, form } => {
                writeln!(f, "target of function call is not constant:")?;
                writeln!(f, "{}", form.fragment(*source).source_context())
            }
            FormError::LetBindingsNotList { source, atom } => {
                writeln!(f, "let form must define bindings as a list:")?;
                writeln!(f, "{}", atom.fragment(*source).source_context())
            }
            FormError::LetBindingEntryNotList { source, atom } => {
                writeln!(f, "let form defines a binding that is not a list:")?;
                writeln!(f, "{}", atom.fragment(*source).source_context())
            }
            FormError::LetBindingEntryTooShort { source, atom } => {
                writeln!(f, "let form binding is missing a value:")?;
                writeln!(f, "{}", atom.fragment(*source).source_context())
            }
            FormError::LetBindingEntryTooLong { source, atom } => {
                writeln!(f, "let form binding has superfluous values:")?;
                writeln!(f, "{}", atom.fragment(*source).source_context())
            }
            FormError::LetBindingVarNotIdent { source, atom } => {
                writeln!(
                    f,
                    "let form binding must define an identifier as the name for the binding:"
                )?;
                writeln!(f, "{}", atom.fragment(*source).source_context())
            }
            FormError::ApplyStaticTooShort { source, atom } => {
                writeln!(f, "apply has too few arguments")?;
                writeln!(f, "{}", atom.fragment(*source).source_context())
            }
            FormError::ApplyStaticTooLong { source, atom } => {
                writeln!(f, "apply has too many arguments")?;
                writeln!(f, "{}", atom.fragment(*source).source_context())
            }
        }
    }
}
