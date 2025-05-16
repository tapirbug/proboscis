//! Creates in-memory representations of data

use super::{data::DataAddress, datatype::IrDataType};
use std::io::{self, IoSlice, Write};

pub fn append_function<W: Write>(
    buf: &mut W,
    table_idx: u32,
) -> io::Result<()> {
    buf.write_all(&type_to_tag_bytes(IrDataType::Function))?;
    buf.write_all(&table_idx.to_le_bytes())?;
    // no stack switch for in-mem functions
    buf.write_all(&0_u32.to_le_bytes())?;
    Ok(())
}

pub fn append_nil<W: Write>(buf: &mut W, offset: i32) -> io::Result<()> {
    buf.write_vectored(&[
        IoSlice::new(&type_to_tag_bytes(IrDataType::Nil)),
        // hack: nil has a layout like a list node and both data and cdr point to itself
        //       this makes it easier to implement that e.g. cdr of nil is nil
        IoSlice::new(&offset.to_le_bytes()),
        IoSlice::new(&offset.to_le_bytes()),
    ])?;
    Ok(())
}

pub fn append_string<W: Write>(buf: &mut W, data: &str) -> io::Result<()> {
    buf.write_vectored(&[
        IoSlice::new(&type_to_tag_bytes(IrDataType::CharacterData)),
        IoSlice::new(&(u32::try_from(data.len()).unwrap()).to_le_bytes()),
        IoSlice::new(data.as_bytes()),
    ])?;
    Ok(())
}

pub fn append_identifier<W: Write>(buf: &mut W, data: &str) -> io::Result<()> {
    buf.write_vectored(&[
        IoSlice::new(&type_to_tag_bytes(IrDataType::Identifier)),
        IoSlice::new(&(u32::try_from(data.len()).unwrap()).to_le_bytes()),
        IoSlice::new(data.as_bytes()),
    ])?;
    Ok(())
}

pub fn append_list_node<W: Write>(
    buf: &mut W,
    car: DataAddress,
    cdr: DataAddress,
) -> io::Result<()> {
    buf.write_vectored(&[
        IoSlice::new(&type_to_tag_bytes(IrDataType::ListNode)),
        IoSlice::new(&car.to_le_bytes()),
        IoSlice::new(&cdr.to_le_bytes()),
    ])?;
    Ok(())
}

pub fn append_sint32<W: Write>(buf: &mut W, number: i32) -> io::Result<()> {
    buf.write_vectored(&[
        IoSlice::new(&type_to_tag_bytes(IrDataType::SInt32)),
        IoSlice::new(&number.to_le_bytes()),
    ])?;
    Ok(())
}

pub fn append_place<W: Write>(
    buf: &mut W,
    data_address: DataAddress,
) -> io::Result<()> {
    buf.write(&data_address.to_le_bytes())?;
    Ok(())
}

pub fn type_to_tag_bytes(data_type: IrDataType) -> [u8; 4] {
    u32::from(data_type.to_tag()).to_le_bytes()
}
