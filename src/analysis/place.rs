use crate::source::{Source, SourceRange};

struct Place<'s> {
    source: Source<'s>,
    source_range: SourceRange<'s>,
}

impl<'s> Place<'s> {}
