use std::{fs, io, path::Path};

/// Immutable source code for compilation.
///
/// Usually borrowed and shared.
pub struct Source {
    text: String, // might include a filename or other useful debug info here in additional
                  // fields if needed
}

impl Source {
    #[cfg(test)]
    pub fn new(source: &str) -> Self {
        Self {
            text: source.into(),
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Source> {
        let text = fs::read_to_string(path)?;
        Ok(Source { text })
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }
}

impl AsRef<str> for Source {
    fn as_ref(&self) -> &str {
        &self.text
    }
}
