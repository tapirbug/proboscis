/// Types that a variant can hold and be tagged with.
#[derive(Debug, Copy, Clone)]
pub enum IrDataType {
    /// A 64-bit cons list node, laid out as follows:
    /// All zero is the empty list.
    /// If non-zero, the first 32 bits must point to an element, the last point
    /// to another list node, or are zero if this is the last node.
    ListNode,
    /// Signed 32-bit integer
    SInt32,
    /// A 32-bit number, followed by
    /// that exact number of bytes forming a valid UTF-8 string.
    ///
    /// Lengths do not include the length header, so the empty strings is one
    /// 32-bit zero.
    ///
    /// Local variables cannot hold this type, but they might refer to it.
    CharacterData,
}
