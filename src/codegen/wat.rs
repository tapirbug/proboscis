use std::{fmt, io::{self, Write}};

use crate::{
    analysis::{MultiStringTable, StringTable},
    ir::{Function, Program, StaticData},
};

pub fn write_wat<W: Write>(w: &mut W, program: &Program) -> io::Result<()> {
    write!(w, "(module\n")?;
    write!(w, "\t(import \"js\" \"mem\" (memory 1))\n")?;
    write_static_data(w, program.static_data())?;
    for function in program.functions() {
        write_function(w, function)?;
    }
    write!(w, ")\n")?; // closing module
    Ok(())
}

fn write_static_data<W: Write>(w: &mut W, static_data: &StaticData) -> io::Result<()> {
    let offset_data = 0;
    write!(w, "\t(data (i32.const {}) {})\n", offset_data, WebassemblyString(static_data.data()))?;
    Ok(())
}

fn write_function<W: Write>(w: &mut W, function: &Function) -> io::Result<()> {
    write!(w, "\t(func ")?;
    if let Some(name) = function.export_name() {
        write!(w, "(export \"{}\") ", name)?;
    }
    write!(w, "(result i32)\n")?;
    // TODO write instruction
    write!(w, "\t)\n")?;
    Ok(())
}

struct WebassemblyString<'s>(&'s [u8]);

impl<'s> fmt::Display for WebassemblyString<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"")?;
        for &byte in self.0 {
            if byte.is_ascii_graphic() || (byte.is_ascii_punctuation() && byte != b'\"') || byte == b' ' {
                write!(f, "{}", char::from(byte))?;
            } else {
                write!(f, "\\")?;
                write!(f, "{:02X?}", byte)?;
            }
        }
        write!(f, "\"")?;
        Ok(())
    }
}
