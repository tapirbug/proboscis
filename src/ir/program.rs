use crate::analysis::MultiStringTable;

pub struct Program<'s> {
    strings: MultiStringTable<'s>,
}

impl<'s> Program<'s> {
    pub fn new(strings: MultiStringTable<'s>) -> Self {
        Self { strings }
    }

    pub fn strings(&self) -> &MultiStringTable<'s> {
        &self.strings
    }
}
