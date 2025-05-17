use crate::analysis::{FunctionDefinition, form::Form};

/// Checks if the function contains any lambdas.
///
/// This is useful to know up front to know if persistent storage is required
/// or local storage is enough.
pub fn contains_function_lambdas<'s, 't>(
    definition: &'t FunctionDefinition<'s, 't>,
) -> bool {
    // if we supported default arguments, we would have to check them here too
    definition.body().iter().any(contains_form_lambdas)
}

pub fn contains_form_lambdas<'s, 't>(form: &'t Form<'s, 't>) -> bool {
    match form {
        Form::Lambda(_) => true,
        Form::Name(..) | Form::FunctionName(..) | Form::Constant(..) => false,
        Form::LetForm(form) => {
            form.bindings()
                .iter()
                .any(|b| contains_form_lambdas(b.value()))
                || form.body().iter().any(contains_form_lambdas)
        }
        Form::IfForm(form) => {
            contains_form_lambdas(form.test_form())
                || contains_form_lambdas(form.then_form())
                || form.else_form().map(contains_form_lambdas).unwrap_or(false)
        }
        Form::AndForm(form) => form.forms().iter().any(contains_form_lambdas),
        Form::OrForm(form) => form.forms().iter().any(contains_form_lambdas),
        Form::Call(form) => form.args().iter().any(contains_form_lambdas),
        Form::Apply(form) => {
            contains_form_lambdas(form.function())
                || contains_form_lambdas(form.args())
        }
        Form::Funcall(form) => {
            contains_form_lambdas(form.function())
                || form.args().iter().any(contains_form_lambdas)
        }
    }
}
