use std::{fmt, io::{self, Write}, mem};

use crate::ir::{type_to_tag, IrDataType, AddressingMode, Function, Instruction, PlaceAddress, Program, StaticData};

use super::locals::local_places_byte_len;

pub fn write_wat<W: Write>(w: &mut W, program: &Program) -> io::Result<()> {
    write!(w, "(module\n")?;
    write!(w, "\t(import \"js\" \"mem\" (memory 1))\n")?; // reserves 64KiB
    // we assume this is present to log at a specific memory offset with a specific len, assuming UTF-8
    write!(w, "\t(import \"console\" \"log\" (func $log (param i32 i32)))\n")?;
    write_static_data(w, program.static_data())?;
    let stack_start = program.static_data().data().len();
    // reserve 10KiB of stack space, disregarding alignment
    let stack_end = stack_start + 10 * 1024_usize;
    let heap_start = stack_end;
    write!(w, "\t(global $stack_bottom i32 (i32.const {}))\n", stack_start)?;
    write!(w, "\t(global $stack_top (mut i32) (i32.const {}))\n", stack_end)?;
    write!(w, "\t(global $local_offset (mut i32) (i32.const {}))\n", stack_start)?;
    write!(w, "\t(global $heap_start (mut i32) (i32.const {}))\n", heap_start)?;
    for (idx, function) in program.functions().iter().enumerate() {
        write_function(w, function, idx)?;
    }
    write!(w, ")\n")?; // closing module
    Ok(())
}

fn write_static_data<W: Write>(w: &mut W, static_data: &StaticData) -> io::Result<()> {
    let offset_data = 0;
    write!(w, "\t(data (i32.const {}) {})\n", offset_data, WebassemblyString(static_data.data()))?;
    Ok(())
}

fn write_function<W: Write>(w: &mut W, function: &Function, idx: usize) -> io::Result<()> {
    let locals_byte_len = local_places_byte_len(function.instructions());

    write!(w, "\t(func $fun{} ", idx)?;
    if let Some(name) = function.export_name() {
        write!(w, "(export \"{}\") ", name)?;
    }
    write!(w, "(param i32) (result i32) (local i32)\n")?;

    if locals_byte_len > 0 {
        write!(w, "\t\tglobal.get $stack_top\n")?;
        write!(w, "\t\ti32.const {}\n", locals_byte_len)?;
        write!(w, "\t\ti32.add\n")?;
        write!(w, "\t\tglobal.set $stack_top\n")?;
    }

    for &instruction in function.instructions() {
        write!(w, "\t\t;; {:?}\n", instruction)?;
        match instruction {
            Instruction::LoadData { data, to } => {
                write_load_place_self_address(w, to)?;
                write!(w, "\t\ti32.const {}\n", data.offset())?;
                write!(w, "\t\ti32.store\n")?;
            },
            Instruction::WritePlace { from, to } => {
                write_load_place_self_address(w, to)?;
                write_load_place_referee(w, from)?;
                write!(w, "\t\ti32.store\n")?;
            },
            Instruction::Call { function, params, to } => {
                // remember the previous local offset before the call
                write!(w, "\t\tglobal.get $local_offset\n")?;
                write_load_place_self_address(w, to)?;
                write_load_place_referee(w, params)?;
                // then set top of the stack as new local offset for the function we call
                write!(w, "\t\tglobal.get $stack_top\n")?;
                write!(w, "\t\tglobal.set $local_offset\n")?;
                write!(w, "\t\tcall $fun{}\n", function.to_i32())?;
                // store result in place we remembered before params
                write!(w, "\t\ti32.store\n")?;
                // restore local offset which we also remembered previously
                write!(w, "\t\tglobal.set $local_offset\n")?;
            },
            Instruction::Return { value } => {
                write_load_place_referee(w, value)?;
                write!(w, "\t\treturn\n")?;
            },
            Instruction::CallPrint { string } => {
                write_load_place_referee(w, string)?;
                write!(w, "\t\tlocal.set 1\n")?; // remember the string start address
                write!(w, "\t\tlocal.get 1\n")?;
                write!(w, "\t\ti32.const {}\n", 2 * mem::size_of::<i32>())?; // skip the tag and length and go to the character data
                write!(w, "\t\ti32.add\n")?;
                write!(w, "\t\tlocal.get 1\n")?;
                write!(w, "\t\ti32.const {}\n", mem::size_of::<i32>())?; // then load the length in bytes
                write!(w, "\t\ti32.add\n")?;
                write!(w, "\t\ti32.load\n")?;
                write!(w, "\t\tcall $log\n")?;
            },
            Instruction::Cons { car, cdr, to } => {
                // type tag, car address, cdr address
                write_heap_alloc(w, mem::size_of::<i32>() * 3)?;
                write!(w, "\t\tlocal.set 1\n")?; // remember the allocation

                // write tag at offset 0
                write!(w, "\t\tlocal.get 1\n")?;
                let tag = type_to_tag(IrDataType::ListNode);
                write!(w, "\t\ti32.const {}\n", tag)?;
                write!(w, "\t\ti32.store\n")?;

                // load car address and write at offset 1
                write!(w, "\t\tlocal.get 1\n")?;
                write!(w, "\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\ti32.add\n")?;
                write_load_place_referee(w, car)?;
                write!(w, "\t\ti32.store\n")?;

                // load cdr address and write at offset 2
                write!(w, "\t\tlocal.get 1\n")?;
                write!(w, "\t\ti32.const {}\n", 2 * mem::size_of::<i32>())?;
                write!(w, "\t\ti32.add\n")?;
                write_load_place_referee(w, cdr)?;
                write!(w, "\t\ti32.store\n")?;

                // finally, remember the list in the target place
                write_load_place_self_address(w, to)?;
                write!(w, "\t\tlocal.get 1\n")?;
                write!(w, "\t\ti32.store\n")?;
            },
            Instruction::ConsumeParam { to } => {
                // load address of target place
                write_load_place_self_address(w, to)?;
                // load the passed location of argument list
                write!(w, "\t\tlocal.get 0\n")?;
                // skip type tag and go to car, load it, and store it in target place
                write!(w, "\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\ti32.add\n")?;
                write!(w, "\t\ti32.load\n")?;
                write!(w, "\t\ti32.store\n")?;
                // load the passed location of argument list again
                write!(w, "\t\tlocal.get 0\n")?;
                // skip type tag and go to cdr, load it, and store it as new argument list
                write!(w, "\t\ti32.const {}\n", 2 * mem::size_of::<i32>())?;
                write!(w, "\t\ti32.add\n")?;
                write!(w, "\t\ti32.load\n")?;
                write!(w, "\t\tlocal.set 0\n")?;


                /*// first load the target address
                write_load_place_address(w, to)?;
                // load the passed location of argument list
                write!(w, "\t\tlocal.get 0\n")?;
                // skip type tag and go to car
                write!(w, "\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\ti32.add\n")?;
                // load car address and remember it
                write!(w, "\t\ti32.load\n")?;
                write!(w, "\t\tlocal.set 1\n")?;
                // load car and store it in the target place we loaded in the beginning
                write!(w, "\t\tlocal.get 1\n")?;
                write!(w, "\t\ti32.store\n")?;

                // now on to modifying params
                write!(w, "\t\tlocal.get 0\n")?; // push the location of the params

                // load the location of the beginning of the params list as dst
                write!(w, "\t\tlocal.get 0\n")?;
                // skip type tag and car and go to cdr
                write!(w, "\t\ti32.const {}\n", 2 * mem::size_of::<i32>())?;
                write!(w, "\t\ti32.add\n")?;
                // load the cdr address
                write!(w, "\t\ti32.load\n")?;
                // copy 3 ints for type tag, car and cdr from the next node to the params list
                write!(w, "\t\ti32.const {}\n", 3 * mem::size_of::<i32>())?;
                write!(w, "\t\tmemory.copy\n")?;*/
            },

            inst => unimplemented!("instruction unimplemented: {:?}", inst)
        }
    }

    if locals_byte_len > 0 {
        write!(w, "\t\tglobal.get $stack_top\n")?;
        write!(w, "\t\ti32.const {}\n", locals_byte_len)?;
        write!(w, "\t\ti32.sub\n")?;
        write!(w, "\t\tglobal.set $stack_top\n")?;
    }

    // TODO write instruction
    write!(w, "\t)\n")?;
    Ok(())
}

/// Loads the address of a place itself, so that it can be overwritten.
fn write_load_place_self_address<W: Write>(w: &mut W, from: PlaceAddress) -> io::Result<()> {
    let offset = from.offset() as usize;
    match from.mode() {
        AddressingMode::Local => {
            write!(w, "\t\tglobal.get $local_offset\n")?;
            write!(w, "\t\ti32.const {}\n", offset)?;
            write!(w, "\t\ti32.add\n")
        },
        AddressingMode::Global => {
            write!(w, "\t\ti32.const {}\n", offset)
        }
    }
}

/// Loads the address that a place points to
fn write_load_place_referee<W: Write>(w: &mut W, from: PlaceAddress) -> io::Result<()> {
    write_load_place_self_address(w, from)?;
    write!(w, "\t\ti32.load\n")
}

/// Writes a heap allocation, the result being the start address of the allocation
fn write_heap_alloc<W: Write>(w: &mut W, size: usize) -> io::Result<()> {
    // just append to the back for now
    write!(w, "\t\tglobal.get $heap_start\n")?;
    write!(w, "\t\tglobal.get $heap_start\n")?;
    write!(w, "\t\ti32.const {}\n", size)?;
    write!(w, "\t\ti32.add\n")?;
    write!(w, "\t\tglobal.set $heap_start\n")?;
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
