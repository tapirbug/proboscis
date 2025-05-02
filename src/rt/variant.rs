use super::datatype::DataType;

#[repr(C)]
struct Address(i32);

#[repr(C)]
struct Length(i32);

#[repr(C)]
struct ListNodeData {
    car: Address,
    cdr: Address,
}

#[repr(C)]
struct SInt32Data(i32);

#[repr(C)]
struct CharacterDataHeader {
    len: Length,
}
