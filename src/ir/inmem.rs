//! Creates in-memory representations of data

use super::{data::DataAddress, datatype::IrDataType};
use std::io::{self, IoSlice, Write};

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
    type_to_tag(data_type).to_le_bytes()
}

pub fn type_to_tag(data_type: IrDataType) -> u32 {
    match data_type {
        IrDataType::ListNode => 0,
        IrDataType::CharacterData => 1,
        IrDataType::SInt32 => 2,
        IrDataType::Identifier => 3,
    }
}

fn tag_to_type(encoded: u32) -> IrDataType {
    match encoded {
        0 => IrDataType::ListNode,
        1 => IrDataType::CharacterData,
        2 => IrDataType::SInt32,
        3 => IrDataType::Identifier,
        _ => panic!(),
    }
}
