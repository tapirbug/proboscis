use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Immutable source code for compilation.
///
/// Usually borrowed and shared.
#[derive(Debug)]
pub struct Source {
    path: PathBuf,
    text: String, // might include a filename or other useful debug info here in additional
                  // fields if needed
}

impl Source {
    #[cfg(test)]
    pub fn new(source: &str) -> Self {
        Self {
            text: source.into(),
            path: PathBuf::new(),
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Source> {
        let path = PathBuf::from(path.as_ref());
        let text = fs::read_to_string(&path)?;
        Ok(Source { path, text })
    }

    pub fn load_many<I: IntoIterator<Item = P>, P: AsRef<Path>>(
        paths: I,
    ) -> io::Result<Vec<Source>> {
        paths.into_iter().map(Self::load).collect()
    }

    pub fn path(&self) -> &Path {
        &self.path
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
