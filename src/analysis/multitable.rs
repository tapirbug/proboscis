use super::{
    StringTable,
    strings::{StringTableEntry, StringTableEntryIter},
};

pub struct MultiStringTable<'s> {
    tables: Vec<StringTable<'s>>,
}

pub struct MultiStringTableEntryIter<'s, 't, 'is, 'it> {
    remaining_tables: &'t [StringTable<'s>],
    iter: Option<StringTableEntryIter<'is, 'it>>,
}

impl<'s> FromIterator<StringTable<'s>> for MultiStringTable<'s> {
    fn from_iter<T: IntoIterator<Item = StringTable<'s>>>(iter: T) -> Self {
        MultiStringTable {
            tables: iter.into_iter().collect(),
        }
    }
}

impl<'s> MultiStringTable<'s> {
    pub fn entries(&self) -> MultiStringTableEntryIter {
        MultiStringTableEntryIter::new(self)
    }
}

impl<'s: 'is, 't: 'it, 'is, 'it> MultiStringTableEntryIter<'s, 't, 'is, 'it> {
    fn new(table: &'t MultiStringTable<'s>) -> Self {
        if table.tables.is_empty() {
            Self {
                remaining_tables: &[],
                iter: None,
            }
        } else {
            Self {
                remaining_tables: &table.tables[1..],
                iter: Some(table.tables[0].entries()),
            }
        }
    }
}

impl<'s: 'is, 't: 'it, 'is, 'it> Iterator
    for MultiStringTableEntryIter<'s, 't, 'is, 'it>
{
    type Item = StringTableEntry<'is, 'it>;

    fn next(&mut self) -> Option<Self::Item> {
        // stop early if no iterator set
        let iter = self.iter.as_mut()?;
        let next_entry = iter.next();
        if next_entry.is_some() {
            // current iterator not exhausted
            return next_entry;
        }

        if self.remaining_tables.is_empty() {
            // no more tables, stop and take iterator
            self.iter.take();
            return None;
        }

        let next_table = &self.remaining_tables[0];
        self.remaining_tables = &self.remaining_tables[1..];
        self.iter = Some(iter.switch_table(next_table));
        self.next()
    }
}
