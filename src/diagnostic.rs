use std::{
    fmt::{self},
    io::{Write as _, stderr},
};

/// A place to send diagnostics to.
///
/// Diagnostics will be reported to stderr and their count remembered. At
/// critical points in the code, we can abort in case of errors by calling
/// `ensure_no_errors`.
pub struct Diagnostics {
    error_count: usize,
    warning_count: usize,
}

impl Diagnostics {
    pub fn new() -> Self {
        Diagnostics {
            error_count: 0,
            warning_count: 0,
        }
    }

    pub fn report<'s, D: Diagnostic>(&mut self, diagnostic: &D) {
        let mut stderr = stderr().lock();
        match diagnostic.kind() {
            DiagnosticKind::Error => {
                self.error_count += 1;
                write!(stderr, "error: ").ok();
            }
            DiagnosticKind::Warning => {
                self.warning_count += 1;
                write!(stderr, "warning: ").ok();
            }
        };
        write!(stderr, "{}", diagnostic).ok();
    }

    /// Consumes the result and if it was an error, reports it.
    pub fn report_if_err<T, D: Diagnostic>(&mut self, result: Result<T, D>) {
        self.ok(result);
    }

    /// Returns the result as Some if Ok, and on errors reports the diagnostic
    /// and remembers the count for later calls to `ensure_no_errors`.
    pub fn ok<T, D: Diagnostic>(&mut self, result: Result<T, D>) -> Option<T> {
        match result {
            Ok(t) => Some(t),
            Err(ref diagnostic) => {
                self.report(diagnostic);
                None
            }
        }
    }

    pub fn ensure_no_errors(&self) -> Result<(), DiagnosticError> {
        if self.error_count == 0 {
            Ok(())
        } else {
            Err(DiagnosticError {
                error_count: self.error_count,
                warning_count: self.warning_count,
            })
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum DiagnosticKind {
    /// Something fatal that stops us from producing a final output.
    Error,
    /// Something that could be a problem but which does not stop us from
    /// producing an output program.
    Warning,
}

pub trait Diagnostic: fmt::Display {
    fn kind(&self) -> DiagnosticKind;
}

#[derive(Debug)]
pub struct DiagnosticError {
    error_count: usize,
    warning_count: usize,
}

impl fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "stopping after")?;
        if self.error_count > 0 {
            write!(f, " {} errors", self.error_count)?;
        }
        if self.error_count > 0 && self.warning_count > 0 {
            write!(f, ",")?;
        }
        if self.warning_count > 0 {
            write!(f, " {} warnings", self.warning_count)?;
        }
        Ok(())
    }
}
