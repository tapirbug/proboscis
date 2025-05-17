use std::io::{self, Write};

use crate::ir::{Function, Instruction, Program};

/// Serializes the intermediate representation of the program in a text format
/// that is intended for reading only (there is no parser).
pub fn write_pirt<W: Write>(w: &mut W, program: &Program) -> io::Result<()> {
    write_function_table(w, program)?;
    for fun in program.functions() {
        write_function(w, fun)?;
    }
    Ok(())
}

fn write_function_table<W: Write>(
    w: &mut W,
    program: &Program,
) -> io::Result<()> {
    if !program.static_data().table_entries().is_empty() {
        let mut entries = program.static_data().table_entries().iter();
        let first = program.resolve_function_addr(*entries.next().unwrap());
        write!(w, "function_table = [ &{}", first.name())?;
        for &entry in entries {
            let fun = program.resolve_function_addr(entry);
            write!(w, " &{}", fun.name())?;
        }
        write!(w, " ]\n\n")?;
    }
    Ok(())
}

fn write_function<W: Write>(w: &mut W, function: &Function) -> io::Result<()> {
    if !function.attributes().is_empty() {
        let attributes = function
            .attributes()
            .iter()
            .map(|attr| format!("{:?}", attr))
            .collect::<Vec<_>>()
            .join("] [");
        write!(w, "[{}] ", attributes)?;
    }
    write!(w, "{} {{\n", function.name())?;
    let mut indents = 1;
    for inst in function.instructions() {
        if let Instruction::ExitBlock = inst {
            indents -= 1;
        }
        for _ in 0..indents {
            write!(w, "\t")?;
        }
        match inst {
            Instruction::EnterBlock => write!(w, "{{\n"),
            Instruction::ExitBlock => write!(w, "}}\n"),
            inst => write!(w, "{:?}\n", inst),
        }?;
        if let Instruction::EnterBlock = inst {
            indents += 1
        }
    }
    write!(w, "}}\n")?;
    Ok(())
}
