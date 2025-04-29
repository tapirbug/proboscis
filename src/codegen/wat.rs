use std::io::{self, Write};

use crate::{analysis::{MultiStringTable, StringTable}, ir::Program};

pub fn write_wat<W: Write>(w: &mut W, program: &Program) -> io::Result<()> {
    write!(w, "(module\n")?;
    write!(w, "\t(import \"js\" \"mem\" (memory 1))\n")?;
    write_strings(w, program.strings())?;
    write!(w, ")\n")?; // closing module
    Ok(())
}

fn write_strings<W: Write>(w: &mut W, strings: &MultiStringTable) -> io::Result<()> {
    for string in strings.entries() {
        let offset = string.offset().absolute_offset_from(0);
        let data = string.data().as_ref();
        write!(w, "\t(data (i32.const {}) {:?})\n", offset, data)?;
    }
    Ok(())
}
