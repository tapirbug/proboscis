enum Instruction {
    Call,
    /// Write a constant value to a place.
    Const,
    /// Create a new list from car and cdr and write it to a place.
    MakeList,
}
