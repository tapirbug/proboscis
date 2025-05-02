use std::{borrow::Cow, mem};

use crate::parse::{AstNode, Source, Token, TokenKind};

/// Analysis result of static strings for a single source file.
///
/// Records the static strings in their decoded form and their lengths.
pub struct StringTable<'s> {
    source: &'s Source,
    /// Static strings, only owned if they contain escapes.
    ///
    /// This could be leveraged to make borrowed-to-borrowed conversions very
    /// fast, although we don't do that right now.
    ///
    /// Could also be an opportunity to try vectored writes when serializing
    /// this.
    strings: Vec<Cow<'s, str>>,
}

#[derive(Clone)]
pub struct StringTableEntry<'s, 't> {
    offset: StringTableOffset,
    value: &'t Cow<'s, str>
}

pub struct StringTableEntryIter<'s, 't> {
    table: &'t StringTable<'s>,
    next_idx: usize,
    next_offset: usize
}

#[derive(Copy, Clone)]
pub struct StringTableOffset(usize);

impl<'s> StringTable<'s> {
    pub fn analyze(source: &'s Source, ast: &[AstNode<'s>]) -> Self {
        let mut table = StringTable {
            source,
            strings: vec![],
        };
        for node in ast {
            table.add_strings(node);
        }
        table
    }

    /// Combined byte length of the string table, including space for the
    /// lengths.
    pub fn byte_len(&self) -> usize {
        self.strings.iter().map(|s| mem::size_of::<u32>() + s.len()).sum()
    }

    pub fn string_count(&self) -> usize {
        self.strings.len()
    }

    pub fn entries<'t>(&'t self) -> StringTableEntryIter<'s, 't> {
        StringTableEntryIter { table: self, next_idx: 0, next_offset: 0 }
    }

    fn add_strings<'b, 'c>(&'b mut self, node: &'c AstNode<'s>) {
        match node {
            AstNode::Atom(atom)
                if atom.token().kind() == TokenKind::StringLit =>
            {
                let decoded =
                    decode_string(atom.token().fragment(self.source).source());
                if let None = self.get_offset_of_decoded(&decoded) {
                    // if not found yet, push it
                    self.strings.push(decoded)
                }
            }
            AstNode::Atom(_) => {} // ignore numbers
            AstNode::List(list) => {
                for el in list.elements() {
                    self.add_strings(el);
                }
            }
            AstNode::QuotedList(list) => {
                for el in list.elements() {
                    self.add_strings(el);
                }
            }
        }
    }

    /// Gets the byte offset in string table memory for a given string token.
    ///
    /// Returns None if not found, although this should never happen when
    /// the analysis ran on the same source file than the one the token
    /// is from.
    pub fn get_offset<'b, 'c>(
        &'b self,
        of_string_token: &'c Token<'s>,
    ) -> Option<StringTableOffset> {
        assert!(matches!(of_string_token.kind(), TokenKind::StringLit));
        self.get_offset_of_encoded(
            of_string_token.fragment(self.source).source(),
        )
    }

    fn get_offset_of_encoded<'b>(
        &'b self,
        encoded: &str,
    ) -> Option<StringTableOffset> {
        self.get_offset_of_decoded(&decode_string(encoded))
    }

    fn get_offset_of_decoded<'b, 'c>(
        &'b self,
        decoded: &'c Cow<'s, str>,
    ) -> Option<StringTableOffset> {
        let decoded = decoded.as_ref();
        let mut offset = 0_usize;
        for intered_str in &self.strings {
            let interned_str = intered_str.as_ref();
            if decoded == interned_str {
                return Some(StringTableOffset(offset));
            } else {
                offset += mem::size_of::<u32>(); // room for the length
                offset += interned_str.len()
            }
        }
        None
    }
}

fn decode_string(raw: &str) -> Cow<str> {
    assert!(raw.len() >= 2);
    assert!(
        raw.as_bytes()[0] == b'"' && raw.as_bytes()[raw.len() - 1] == b'"'
    );
    // drop the surrounding quotes
    let raw = &raw[1..(raw.len() - 1)];
    let escape_count = raw.as_bytes().iter().filter(|&&b| b == b'\\').count();
    if escape_count > 0 {
        let mut decoded = String::with_capacity(raw.len() - escape_count);
        let mut last_was_esc = false;
        for raw in raw.chars() {
            if last_was_esc || raw != '\\' {
                decoded.push(raw);
                last_was_esc = false;
            } else {
                last_was_esc = true;
            }
        }
        Cow::Owned(decoded)
    } else {
        Cow::Borrowed(raw)
    }
}

impl<'s, 't> StringTableEntryIter<'s, 't> {
    /// Make a new iterator that continues from the next string table but continuing
    /// with the offset from previous table iterations.
    pub fn switch_table<'c, 'ns, 'nt>(&'c self, next: &'nt StringTable<'ns>) -> StringTableEntryIter<'ns, 'nt> {
        StringTableEntryIter {
            table: next,
            next_offset: self.next_offset,
            next_idx: 0
        }        
    }
}

impl<'s, 't> Iterator for StringTableEntryIter<'s, 't> {
    type Item = StringTableEntry<'s, 't>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_idx >= self.table.strings.len() {
            return None
        }
        let data = &self.table.strings[self.next_idx];
        let offset = self.next_offset;
        self.next_idx += 1;
        self.next_offset += mem::size_of::<u32>() + data.len();
        Some(StringTableEntry { offset: StringTableOffset(offset), value: data })
    }
}

impl<'s, 't> StringTableEntry<'s, 't> {
    pub fn data(&self) -> &'t Cow<'s, str> {
        self.value
    }

    pub fn offset(&self) -> StringTableOffset {
        self.offset
    }
}

impl StringTableOffset {
    pub fn absolute_offset_from(&self, relative_to: usize) -> usize {
        relative_to + self.0
    }
}

#[cfg(test)]
mod test {
    use crate::parse::Parser;

    use super::*;

    #[test]
    fn test_borrowed() {
        let source = &Source::new(
            "(deffun asdf () \"heya!\" (\"how's it going\" \"how's it going\"  \"how's it going\") '(\"pal\"))",
        );
        let ast = Parser::new(source).parse().unwrap();
        let table = StringTable::analyze(source, &ast);
        assert_eq!(table.string_count(), 3);

        let offset = table.get_offset_of_encoded("\"heya!\"").unwrap();
        assert_eq!(offset.0, 0);
        let offset =
            table.get_offset_of_encoded("\"how's it going\"").unwrap();
        assert_eq!(offset.0, 4 + "heya!".len());
        let offset = table.get_offset_of_encoded("\"pal\"").unwrap();
        assert_eq!(offset.0, 4 + "heya!".len() + 4 + "how's it going".len());

        let offset = table
            .get_offset_of_decoded(&Cow::Borrowed("heya!"))
            .unwrap();
        assert_eq!(offset.0, 0);
        let offset = table
            .get_offset_of_decoded(&Cow::Borrowed("how's it going"))
            .unwrap();
        assert_eq!(offset.0, 4 + "heya!".len());
        let offset =
            table.get_offset_of_decoded(&Cow::Borrowed("pal")).unwrap();
        assert_eq!(offset.0, 4 + "heya!".len() + 4 + "how's it going".len());
    }

    #[test]
    fn test_escaped() {
        let source = &Source::new(
            "(\"9\\\\11\" \"\\\"What?\\\" she said\" \"9\\\\11\")",
        );
        let ast = Parser::new(source).parse().unwrap();
        let table = StringTable::analyze(source, &ast);
        assert_eq!(table.string_count(), 2);

        let offset = table.get_offset_of_encoded("\"9\\\\11\"").unwrap();
        assert_eq!(offset.0, 0);
        let offset = table
            .get_offset_of_encoded("\"\\\"What?\\\" she said\"")
            .unwrap();
        assert_eq!(offset.0, 4 + "9\\11".len());
    }
}
