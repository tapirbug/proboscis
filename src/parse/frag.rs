use std::{fmt, marker::PhantomData, ptr};

use super::source::Source;

/// A non-empty portion of source code.
#[derive(Copy, Clone)]
pub struct Fragment<'s> {
    /// Full source code into which from and to index to locate this fragment.
    source: &'s Source,
    /// Location and length of the fragment in the source code.
    range: SourceRange<'s>,
}

/// A non-empty range of characters in a string, as two byte offsets.
#[derive(Copy, Clone)]
pub struct SourceRange<'s> {
    /// First character (inclusive)
    from: usize,
    /// Last character (exclusive)
    to: usize,
    /// Marker to tell the typesystem that the type system that the validity
    /// of indexes depends on a reference to immutable string data that must
    /// not be changed as long as this source range exists.
    _lifetime: PhantomData<&'s Source>,
}

/// A nice way of displaying context for a fragment.
pub struct SourceContext<'s>(Fragment<'s>);

/// A position in source code of a character, optimized for reading by humans.
///
/// Line and column are 1-based and count characters, not byte offsets.
#[derive(Copy, Clone)]
pub struct SourceLocation {
    /// 1-based line number
    line: usize,
    /// 1-based column number
    col: usize,
}

impl<'s> Fragment<'s> {
    pub fn new(source: &'s Source, from: usize, to: usize) -> Self {
        SourceRange::new(from, to).of(source)
    }

    pub fn union<I: IntoIterator<Item = Self>>(fragments: I) -> Option<Self> {
        fragments.into_iter().reduce(|left, right| {
            assert!(ptr::eq(left.source, right.source)); // OPTIMIZE this should be compared by pointer
            Self {
                source: left.source,
                range: SourceRange::new(
                    left.range.from.min(right.range.from),
                    left.range.to.max(right.range.to),
                ),
            }
        })
    }

    /// The amount of characters in the fragment.
    ///
    /// This may be less than the number of bytes.
    pub fn char_count(self) -> usize {
        self.source().chars().count()
    }

    /// The characters that make up the source code of this fragment.
    pub fn source(self) -> &'s str {
        &self.source.as_ref()[self.range.from..self.range.to]
    }

    /// All characters before the fragment.
    pub fn source_before(self) -> &'s str {
        &self.source.as_ref()[..self.range.from]
    }

    /// All characters after the fragment.
    pub fn source_after(self) -> &'s str {
        &self.source.as_ref()[self.range.to..]
    }

    /// A nice way of displaying context about the fragment.
    pub fn source_context(self) -> SourceContext<'s> {
        SourceContext(self)
    }

    pub fn first_line(self) -> &'s str {
        let src = self.source.as_ref();
        let line_from_idx = line_start_before(src, self.range.from);
        let line_to_idx = line_end_after_or_eq(src, self.range.from);
        &src[line_from_idx..line_to_idx]
    }

    /// The full line or lines that make up this fragments, including chracters
    /// before the start and after the end up to the next newline, if any.
    ///
    /// The final newlines is always excluded, while interior newlines are
    /// preserved.
    pub fn all_lines(self) -> &'s str {
        let src = self.source.as_ref();
        let line_from_idx = line_start_before(src, self.range.from);
        let line_to_idx = line_end_after_or_eq(src, self.range.to);
        &src[line_from_idx..line_to_idx]
    }

    pub fn from_position(self) -> SourceLocation {
        SourceLocation::START.advance(self.source_before())
    }

    pub fn to_position(self) -> SourceLocation {
        // same as SourcePosition::START.advance(self.source_before()).advance(self.source())
        // or self.from_position().advance(self.source())
        SourceLocation::START.advance(&self.source.as_ref()[..self.range.to])
    }
    pub fn from_to_positions(self) -> (SourceLocation, SourceLocation) {
        let from = self.from_position();
        let to = from.advance(self.source());
        (from, to)
    }
}

/// Find the start (inclusive) and end (exclusive) indexes of the line
/// containing the specified mid character.
///
/// The last trailing newline is excluded.
fn line_of_char_at_idx(str: &str, mid: usize) -> (usize, usize) {
    (line_start_before(str, mid), line_end_after_or_eq(str, mid))
}

fn line_start_before(str: &str, before: usize) -> usize {
    str.as_bytes()
        .iter()
        .enumerate()
        .take(before)
        .rev()
        .find(|&(_, &c)| c == b'\n')
        .map(|(i, _)| i + 1)
        .unwrap_or(0)
}

fn line_end_after_or_eq(str: &str, after: usize) -> usize {
    str.as_bytes()
        .iter()
        .enumerate()
        // assume the specified from character is not itself the newline by adding (this allows skipping lines by giving the newline offsite)
        .skip(after + 1)
        .find(|&(_, &c)| c == b'\n')
        .map(|(i, _)| i)
        .unwrap_or_else(|| str.len())
}

impl<'s> fmt::Debug for Fragment<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let source = self.source();
        let from_pos = self.from_position();
        let line = from_pos.line_no();
        let col = from_pos.col_no();
        f.debug_struct("Fragment")
            .field("source", &source)
            .field("line", &line)
            .field("col", &col)
            .finish()
    }
}

impl<'s> fmt::Display for Fragment<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let source = self.source();
        let from_pos = self.from_position();
        write!(f, "`{source}` at {from_pos}")
    }
}

impl<'s> fmt::Display for SourceContext<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (from, to) = self.0.from_to_positions();
        let first_line = self.0.first_line();
        writeln!(f, "At line {}:", from.line_no())?;
        writeln!(f, "{}", first_line)?;
        let highlight_from = from.col_no() - 1;
        let highlight_to = if from.line_no() == to.line_no() {
            to.col_no() - 1
        } else {
            highlight_from + 1
        };
        for _ in 0..highlight_from {
            write!(f, " ")?;
        }
        for _ in highlight_from..highlight_to {
            write!(f, "^")?;
        }
        Ok(())
    }
}

impl<'s> SourceRange<'s> {
    pub fn new(from: usize, to: usize) -> Self {
        assert!(from <= to, "Source range with from > to");
        assert!(from != to, "Source range with from == to would be empty");
        Self {
            from,
            to,
            _lifetime: PhantomData,
        }
    }

    pub fn union_two(
        left: SourceRange<'s>,
        right: SourceRange<'s>,
    ) -> SourceRange<'s> {
        Self {
            from: left.from.min(right.from),
            to: left.to.max(right.to),
            _lifetime: PhantomData,
        }
    }

    pub fn union<I: IntoIterator<Item = Self>>(ranges: I) -> Option<Self> {
        ranges.into_iter().reduce(Self::union_two)
    }

    pub fn of(self, source: &'s Source) -> Fragment<'s> {
        let Self { from, to, .. } = self;
        let len = source.len();
        assert!(
            from <= len,
            "Source range from={from} would be out of bounds for len={len}"
        );
        assert!(
            to <= len,
            "Source range to={to} would be out of bounds for len={len}"
        );
        Fragment {
            source,
            range: self,
        }
    }

    /// Length of the source range in bytes, not characters
    pub fn len(self) -> usize {
        self.to - self.from
    }
}

impl<'s> fmt::Debug for SourceRange<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceRange")
            .field("from", &self.from)
            .field("to", &self.to)
            .finish()
    }
}

impl<'s> fmt::Display for SourceRange<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { from, to, .. } = self;
        write!(f, "`from byte {from} through {to}")
    }
}

impl SourceLocation {
    const START: SourceLocation = SourceLocation { line: 1, col: 1 };

    /// The 1-based line number.
    pub fn line_no(self) -> usize {
        self.line
    }

    // The 1-based column number.
    pub fn col_no(self) -> usize {
        self.col
    }

    fn advance(mut self, source: &str) -> Self {
        for char in source.chars() {
            match char {
                '\n' => {
                    self.line += 1;
                    self.col = SourceLocation::START.col;
                }
                _ => {
                    self.col += 1;
                }
            }
        }
        self
    }
}

impl fmt::Debug for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let line = self.line_no();
        let col = self.col_no();
        f.debug_struct("SourcePosition")
            .field("line", &line)
            .field("col", &col)
            .finish()
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let line = self.line_no();
        let col = self.col_no();
        write!(f, "{line}:{col}")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::iter::{empty, once};
    use std::ptr;

    #[should_panic]
    #[test]
    fn out_of_bounds_new() {
        Fragment::new(&Source::new("a"), 1, 2);
    }

    #[should_panic]
    #[test]
    fn to_lt_from_new() {
        Fragment::new(&Source::new("a"), 1, 0);
    }

    #[should_panic]
    #[test]
    fn empty_new() {
        Fragment::new(&Source::new("a"), 1, 1);
    }

    #[test]
    fn union_zero() {
        let positions = empty::<Fragment<'static>>();
        let result = Fragment::union(positions);
        assert!(result.is_none());
    }

    #[test]
    fn union_one() {
        let source = Source::new("asdf");
        let positions = once(Fragment::new(&source, 0, 4));
        let result = Fragment::union(positions)
            .expect("expected union of one to have a result");
        assert!(ptr::eq(result.source, &source));
        assert_eq!(result.range.from, 0);
        assert_eq!(result.range.to, 4);
    }

    #[test]
    fn union_two_array() {
        let source = Source::new("asdf");
        let positions: [Fragment<'_>; 2] =
            [Fragment::new(&source, 0, 1), Fragment::new(&source, 1, 3)];
        let result = Fragment::union(positions)
            .expect("expected union of two to have a result");
        assert!(ptr::eq(result.source, &source));
        assert_eq!(result.range.from, 0);
        assert_eq!(result.range.to, 3);
    }

    #[test]
    fn union_three_vec() {
        let source = Source::new("asdf");
        let positions = vec![
            Fragment::new(&source, 2, 4),
            Fragment::new(&source, 1, 2),
            Fragment::new(&source, 2, 3),
        ];
        let result = Fragment::union(positions)
            .expect("expected union of two to have a result");
        assert!(ptr::eq(result.source, &source));
        assert_eq!(result.range.from, 1);
        assert_eq!(result.range.to, 4);
    }

    #[test]
    fn source_accessors() {
        let source = Source::new("a = 1");
        let fragment = Fragment::new(&source, 2, 3);
        assert_eq!(fragment.source_before(), "a ");
        assert_eq!(fragment.source(), "=");
        assert_eq!(fragment.source_after(), " 1");
    }

    #[test]
    fn char_count() {
        let source = Source::new("你好");
        let fragment = Fragment::new(&source, 0, "你好".len());
        let char_count = fragment.char_count();
        assert_eq!(char_count, 2);
    }

    #[test]
    fn fragment_positions_line_1() {
        let source = Source::new("a = 1");
        let fragment = Fragment::new(&source, 2, 3);
        let from = fragment.from_position();
        assert_eq!(from.line_no(), 1);
        assert_eq!(
            from.col_no(),
            3,
            "one-based column number for from position"
        );
        let to = fragment.to_position();
        assert_eq!(to.line_no(), 1);
        assert_eq!(to.col_no(), 4, "one-based column number for to position");
        let (from, to) = fragment.from_to_positions();
        assert_eq!(from.line_no(), 1);
        assert_eq!(
            from.col_no(),
            3,
            "one-based column number for from position, combined version"
        );
        assert_eq!(to.line_no(), 1);
        assert_eq!(
            to.col_no(),
            4,
            "one-based column number for to position, combined version"
        );
    }

    #[test]
    fn fragment_positions_lines_2_3() {
        let source = Source::new("\nb = 3\n \nc=4");
        let equals_line_2 = Fragment::new(&source, 3, 4);
        assert_eq!(equals_line_2.source(), "=");
        let literal_line_4 = Fragment::new(&source, 11, 12);
        assert_eq!(literal_line_4.source(), "4");

        let from = equals_line_2.from_position();
        assert_eq!(from.line_no(), 2);
        assert_eq!(
            from.col_no(),
            3,
            "one-based column number for from position"
        );
        let to = equals_line_2.to_position();
        assert_eq!(to.line_no(), 2);
        assert_eq!(to.col_no(), 4, "one-based column number for to position");
        let (from, to) = equals_line_2.from_to_positions();
        assert_eq!(from.line_no(), 2);
        assert_eq!(
            from.col_no(),
            3,
            "one-based column number for from position, combined version"
        );
        assert_eq!(to.line_no(), 2);
        assert_eq!(
            to.col_no(),
            4,
            "one-based column number for to position, combined version"
        );

        let from = literal_line_4.from_position();
        assert_eq!(from.line_no(), 4);
        assert_eq!(
            from.col_no(),
            3,
            "one-based column number for from position"
        );
        let to = literal_line_4.to_position();
        assert_eq!(to.line_no(), 4);
        assert_eq!(to.col_no(), 4, "one-based column number for to position");
        let (from, to) = literal_line_4.from_to_positions();
        assert_eq!(from.line_no(), 4);
        assert_eq!(
            from.col_no(),
            3,
            "one-based column number for from position, combined version"
        );
        assert_eq!(to.line_no(), 4);
        assert_eq!(
            to.col_no(),
            4,
            "one-based column number for to position, combined version"
        );
    }

    #[test]
    fn format_position() {
        let source = Source::new("\nb = 3\n \nc=4");
        let literal_line_4 =
            &format!("{}", Fragment::new(&source, 11, 12).from_position());
        assert_eq!(literal_line_4, "4:3");
    }

    #[test]
    fn format_fragment() {
        let source = Source::new("\nb = 3\n \nc=4");
        let b_line_2 = &format!("{}", Fragment::new(&source, 1, 2));
        assert_eq!(b_line_2, "`b` at 2:1");
    }
}
