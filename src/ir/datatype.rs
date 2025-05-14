use std::fmt;

/// Types that a variant can hold and be tagged with.
#[derive(Debug, Copy, Clone)]
pub enum IrDataType {
    /// Nil, laid out like a list node but car and cdr pointing to itself.
    Nil,
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
    CharacterData,
    /// A 32-bit number, followed by
    /// that exact number of bytes forming a valid UTF-8 string.
    ///
    /// Lengths do not include the length header, so the empty strings is one
    /// 32-bit zero.
    Identifier,
    /// A function that can be called with apply or funcall.
    /// 
    /// The first entry is a table index for the function.
    /// 
    /// The second entry is a value for the second parameter that is used to
    /// pass an alternate stack offset for accessing closure parameters. It's
    /// zero (nil) if no closure parameters are to be used.
    Function,
}

#[derive(Copy, Clone)]
pub struct IrDataTypeTag {
    value: u32
}

const TYPE_COUNT: u32 = 5;
const HIGHEST_T_BIT: u32 = 1 << TYPE_COUNT;
const LOWEST_T_BIT: u32 = 0b1;
const ALL_T_BITS: u32 = HIGHEST_T_BIT + (HIGHEST_T_BIT - 1);

impl IrDataType {
    pub fn to_tag(self) -> IrDataTypeTag {
        IrDataTypeTag {
            value: match self {
                IrDataType::Nil => 0b1,
                IrDataType::ListNode => 0b10,
                IrDataType::SInt32 => 0b100,
                IrDataType::CharacterData => 0b1000,
                IrDataType::Identifier => 0b1_0000,
                IrDataType::Function => 0b10_0000
            }
        }
    }

    pub fn to_u32(self) -> u32 {
        self.to_tag().to_u32()
    }
}

impl IrDataTypeTag {
    pub fn to_type(self) -> IrDataType {
        match self.value {
            0b1 => IrDataType::Nil,
            0b10 => IrDataType::ListNode,
            0b100 => IrDataType::SInt32,
            0b1000 => IrDataType::CharacterData,
            0b1_0000 => IrDataType::Identifier,
            0b10_0000 => IrDataType::Function,
            _ => unreachable!() // valid tags don't end up here, and IrDataTypeTag contains a valid tag
        }
    }

    pub fn to_u32(self) -> u32 {
        self.value
    }
}

impl TryFrom<u32> for IrDataTypeTag {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if (value >> (TYPE_COUNT + 1)) != 0 || value == 0 {
            // out of bounds or all zero is not allowed
            return Err(())
        }
        Ok(IrDataTypeTag { value })
    }    
}

impl From<IrDataType> for IrDataTypeTag {
    fn from(value: IrDataType) -> Self {
        value.to_tag()
    }
}

impl From<IrDataTypeTag> for IrDataType {
    fn from(value: IrDataTypeTag) -> Self {
        value.to_type()
    }
}

impl From<IrDataTypeTag> for u32 {
    fn from(value: IrDataTypeTag) -> Self {
        value.value
    }
}

impl fmt::Debug for IrDataTypeTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IrDataTypeTag")
            .field(&IrDataTypeTag::from(*self))
            .finish()
    }
}
