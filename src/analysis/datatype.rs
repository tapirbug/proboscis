use std::ops::{BitAnd, BitOr};

/// Types that a variant can hold and be tagged with.
#[derive(Debug, Copy, Clone)]
pub enum DataType {
    /// A 64-bit cons list node, laid out as follows:
    /// All zero is the empty list.
    /// If non-zero, the first 32 bits must point to an element, the last point
    /// to another list node, or are zero if this is the last node.
    ListNode,
    /// Signed 64-bit integer
    SInt64,
    /// A 32-bit number, followed by
    /// that exact number of bytes forming a valid UTF-8 string.
    ///
    /// Lengths do not include the length header, so the empty strings is one
    /// 32-bit zero.
    ///
    /// Local variables cannot hold this type, but they might refer to it.
    CharacterData,
}

const HIGHEST_T_BIT: u8 = 0b100;
const LOWEST_T_BIT: u8 = 0b1;
const ALL_T_BITS: u8 = 0b111;

/// Data type to internal single-bit representation.
const fn t_to_i(data_type: DataType) -> u8 {
    match data_type {
        DataType::ListNode => 0b1,
        DataType::SInt64 => 0b10,
        DataType::CharacterData => 0b100,
    }
}

/// Single bit back to enum.
///
/// # Panics
/// If more than one bit given or not a value returned from t_to_i.
const fn i_to_t_unsafe(as_i: u8) -> DataType {
    match as_i {
        0b1 => DataType::ListNode,
        0b10 => DataType::SInt64,
        0b100 => DataType::CharacterData,
        _ => panic!(),
    }
}

#[derive(Copy, Clone)]
pub struct DataTypeSet {
    types: u8,
}

impl DataTypeSet {
    const EMPTY: DataTypeSet = DataTypeSet { types: 0 };
    const ALL: DataTypeSet = DataTypeSet { types: ALL_T_BITS };

    /// A set containing no types.
    pub fn empty() -> Self {
        Self::EMPTY
    }

    /// A set containing all types.
    pub fn all() -> Self {
        DataTypeSet { types: 0b111 }
    }

    /// Creates a new set without the given type and returns it.
    pub fn remove(self, data_type: DataType) -> Self {
        Self {
            types: (self.types & !t_to_i(data_type)) & ALL_T_BITS,
        }
    }

    pub fn contains(self, data_type: DataType) -> bool {
        (self.types & t_to_i(data_type)) != 0
    }

    pub fn contains_all(self, data_types: DataTypeSet) -> bool {
        (self.types & data_types.types) == data_types.types
    }

    pub fn contains_any(self, data_types: DataTypeSet) -> bool {
        (self.types & data_types.types) != 0
    }

    /// Modifies the set in place to exclude the specified type.
    ///
    /// Returns self for chaining.
    pub fn remove_in_place(&mut self, data_type: DataType) -> &mut Self {
        *self = self.remove(data_type);
        self
    }

    pub fn remove_all(self, types: DataTypeSet) -> Self {
        Self {
            types: (self.types & !types.types) & ALL_T_BITS,
        }
    }

    pub fn remove_all_in_place(&mut self, types: DataTypeSet) -> &mut Self {
        *self = self.remove_all(types);
        self
    }

    /// Creates a new set without the given type and returns it.
    pub fn add(self, data_type: DataType) -> Self {
        Self {
            types: self.types | t_to_i(data_type),
        }
    }

    /// Modifies the set in place to exclude the specified type.
    ///
    /// Returns self for chaining.
    pub fn add_in_place(&mut self, data_type: DataType) -> &mut Self {
        *self = self.add(data_type);
        self
    }

    pub fn add_all(self, types: DataTypeSet) -> Self {
        Self {
            types: self.types | types.types,
        }
    }

    pub fn add_all_in_place(&mut self, types: DataTypeSet) -> &mut Self {
        *self = self.add_all(types);
        self
    }

    pub fn union<T: IntoIterator<Item = DataTypeSet>>(iter: T) -> Self {
        iter.into_iter()
            .reduce(|l, r| DataTypeSet {
                types: l.types | r.types,
            })
            .unwrap_or(Self::EMPTY)
    }

    pub fn iter(self) -> DataTypeSetIter {
        DataTypeSetIter::new_unsafe(self.types)
    }
}

impl BitOr for DataType {
    type Output = DataTypeSet;

    fn bitor(self, rhs: Self) -> Self::Output {
        DataTypeSet {
            types: t_to_i(self) | t_to_i(rhs),
        }
    }
}

// I think this has no useful use cases
/*impl BitAnd for DataType {
    type Output = DataTypeSet;

    fn bitand(self, rhs: Self) -> Self::Output {
        DataTypeSet { types: t_to_i(self) & t_to_i(rhs) }
    }
}*/

/// Shorthand for add_all
impl BitOr<Self> for DataTypeSet {
    type Output = DataTypeSet;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.add_all(rhs)
    }
}

/// Gets the intersection of two type sets.
///
/// Different from remove!
impl BitAnd<Self> for DataTypeSet {
    type Output = DataTypeSet;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            types: self.types & rhs.types,
        }
    }
}

/// Add a single data type to the set
impl BitOr<DataType> for DataTypeSet {
    type Output = DataTypeSet;

    fn bitor(self, rhs: DataType) -> Self::Output {
        self.add(rhs)
    }
}

// not sure yet if this will be needed
/*impl BitAnd<DataType> for DataTypeSet {
    type Output = DataTypeSet;

    fn bitand(self, rhs: DataType) -> Self::Output {
        Self {
            types: self.types & t_to_i(rhs)
        }
    }
}*/

impl FromIterator<DataType> for DataTypeSet {
    fn from_iter<T: IntoIterator<Item = DataType>>(iter: T) -> Self {
        DataTypeSet {
            types: iter.into_iter().fold(0, |a, b| a | t_to_i(b)),
        }
    }
}

#[derive(Copy, Clone)]
struct DataTypeSetIter {
    types: u8,
    next_check: u8,
}

impl DataTypeSetIter {
    const FIRST_NEXT_CHECK: u8 = LOWEST_T_BIT;
    const END_NEXT_CHECK: u8 = HIGHEST_T_BIT << 1;

    pub fn new_unsafe(for_types: u8) -> Self {
        DataTypeSetIter {
            types: for_types,
            next_check: Self::FIRST_NEXT_CHECK,
        }
    }
}

impl Iterator for DataTypeSetIter {
    type Item = DataType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_check == Self::END_NEXT_CHECK {
            None
        } else {
            let if_present = self.next_check;
            let is_present = self.types & if_present == if_present;
            self.next_check <<= 1; // we assume that no bytes are unused
            if is_present {
                return Some(i_to_t_unsafe(if_present));
            } else {
                self.next() // maybe next type is present
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_list_bit() {
        let types: DataTypeSet = [DataType::CharacterData, DataType::ListNode]
            .into_iter()
            .collect();
        assert!(types.contains(DataType::CharacterData));
        assert!(types.contains(DataType::ListNode));
        assert!(types.contains_all(types));
        assert!(types.contains_any(types));
    }
}
