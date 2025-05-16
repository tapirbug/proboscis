use std::{
    fmt,
    fs::File,
    io::{self, Read},
    ops::Range,
    path::{Path, PathBuf},
    ptr,
};

/// A set of source files that should be compiled as a unit.
pub struct SourceSet {
    /// Append-only combined source code without duplicate source files.
    /// There is no padding between source files, so they should be accessed
    /// via the source iterator to tell the files apart.
    combined_source: String,
    entries: Vec<SourceInfoEntry>,
}

pub struct SourceSetIter<'set> {
    source: &'set SourceSet,
    consumed_entries: usize,
}

/// Internal representation without lifetimes
struct SourceInfoEntry {
    /// File, if this source was loaded from a file
    file: Option<PathBuf>,
    /// The byte range in the combined source
    range: Range<usize>,
}

#[derive(Clone, Copy)]
pub struct Source<'set> {
    set: &'set SourceSet,
    idx: usize,
}

impl SourceSet {
    /// Creates a new empty source set.
    pub fn new() -> Self {
        Self {
            combined_source: String::new(),
            entries: Vec::new(),
        }
    }

    #[cfg(test)]
    pub fn new_debug(source_string: &str) -> Self {
        let mut set = SourceSet::new();
        set.load_without_path(source_string);
        set
    }

    /// Checks if no source files have successfully been added
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Checks the amount of sources, which may be more than the amount of
    /// files because not all sources are files.
    pub fn source_count(&self) -> usize {
        self.entries.len()
    }

    /// Loads a file into the source set. Will refuse to load files it has
    /// already loaded (but can easily be tricked by paths that would be
    /// the same canonically).
    pub fn load<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<Source, SourceError> {
        let path = path.as_ref();

        self.check_duplicate_file(path)?;
        let mut file = File::open(path).map_err(|io| SourceError::IO {
            path: path.into(),
            error: io,
        })?;

        let appended_bytes_after_padding = file
            .read_to_string(&mut self.combined_source)
            .map_err(|io| SourceError::IO {
                path: path.into(),
                error: io,
            })?;

        self.entries.push(SourceInfoEntry {
            file: Some(path.into()),
            range: (self.combined_source.len() - appended_bytes_after_padding)
                ..self.combined_source.len(),
        });

        Ok(Source {
            set: self,
            idx: self.entries.len() - 1,
        })
    }

    /// Loads a source that is not associated with a particular file.
    ///
    /// No attempt is made to check for duplicates for sources added this way.
    pub fn load_without_path(&mut self, source: &str) -> Source {
        self.combined_source.push_str(source);
        self.entries.push(SourceInfoEntry {
            file: None,
            range: (self.combined_source.len() - source.len())
                ..self.combined_source.len(),
        });
        Source {
            set: self,
            idx: self.entries.len() - 1,
        }
    }

    fn check_duplicate_file(&self, path: &Path) -> Result<(), SourceError> {
        for entry in &self.entries {
            if let Some(ref loaded_path) = entry.file {
                if path == loaded_path {
                    return Err(SourceError::DuplicateSource(path.into()));
                }
            }
        }
        Ok(())
    }

    pub fn iter(&self) -> SourceSetIter {
        SourceSetIter {
            source: self,
            consumed_entries: 0,
        }
    }

    #[cfg(test)]
    pub fn one(&self) -> Source {
        assert_eq!(self.entries.len(), 1);
        self.iter().next().unwrap()
    }
}

impl<'set> Iterator for SourceSetIter<'set> {
    type Item = Source<'set>;

    fn next(&mut self) -> Option<Self::Item> {
        let this_idx = self.consumed_entries;
        if this_idx >= self.source.entries.len() {
            return None;
        }
        self.consumed_entries += 1;
        return Some(Source {
            set: self.source,
            idx: this_idx,
        });
    }
}

impl<'set> Source<'set> {
    pub fn path(self) -> Option<&'set Path> {
        (&self.set.entries[self.idx])
            .file
            .as_ref()
            .map(|f| f.as_ref())
    }

    pub fn as_str(self) -> &'set str {
        &self.set.combined_source[self.set.entries[self.idx].range.clone()]
    }

    pub fn len(self) -> usize {
        self.as_str().len()
    }
}

impl<'set> PartialEq for Source<'set> {
    fn eq(&self, other: &Self) -> bool {
        // sets have to be identical at the same address to be equal
        ptr::eq(self.set, other.set) && self.idx == other.idx
    }
}

impl<'set> Eq for Source<'set> {}

impl<'set> fmt::Debug for Source<'set> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = self.path() {
            f.debug_tuple("Source").field(&path).finish()
        } else {
            f.debug_tuple("Source").field(&"No source file").finish()
        }
    }
}

#[derive(Debug)]
pub enum SourceError {
    IO { path: PathBuf, error: io::Error },
    DuplicateSource(PathBuf),
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateSource(source) => {
                writeln!(f, "duplicate source file `{}`", source.display())
            }
            Self::IO { path, error } => {
                writeln!(
                    f,
                    "I/O error trying to read source file `{}`: {}",
                    path.display(),
                    error
                )
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn load_debug() {
        let debug = SourceSet::new_debug("asdf");
        let one = debug.one();
        assert_eq!(one.as_str(), "asdf");
    }
}
