use std::{
    fmt, fs,
    io::{self, Write},
    mem,
};

use crate::ir::{
    AddressingMode, FunctionAttribute, Instruction, IrDataType, PlaceAddress,
    Program, StaticData,
};

use super::locals::{LocalPlacesInfo, LocalStrategy};

const RUNTIME_PATH: &str = "rt/rt.wat";

pub fn write_wat<W: Write>(w: &mut W, program: &Program) -> io::Result<()> {
    write!(w, "(module\n")?;
    write!(w, "\t(import \"js\" \"mem\" (memory 10))\n")?; // reserves 640KiB
    // we assume this is present to log at a specific memory offset with a specific len, assuming UTF-8
    write!(
        w,
        "\t(import \"console\" \"log\" (func $log (param i32 i32)))\n"
    )?;
    write_static_data(w, program.static_data())?;
    write_tables(w, program.static_data())?;
    write_runtime_variables(w, program.static_data())?;
    write_runtime_functions(w)?;
    for (idx, _) in program.functions().iter().enumerate() {
        write_function(w, program, idx)?;
    }
    write!(w, ")\n")?; // closing module
    Ok(())
}

fn write_tables<W: Write>(
    w: &mut W,
    static_data: &StaticData,
) -> io::Result<()> {
    write!(
        w,
        "\t(table {} funcref)\n",
        static_data.table_entries().len()
    )?;
    write!(w, "\t(elem (i32.const 0)")?;
    for entry in static_data.table_entries() {
        let entry = entry.to_i32();
        write!(w, " $fun{}", entry)?;
    }
    write!(w, ")\n")?;
    Ok(())
}

fn write_runtime_variables<W: Write>(
    w: &mut W,
    static_data: &StaticData,
) -> io::Result<()> {
    let stack_start = static_data.data().len();
    // reserve 10KiB of stack space right after the constant data, disregarding alignment
    // the stack will grow there from the bottom by increasing stack_bottom
    let stack_end = stack_start + 10 * 1024_usize;
    let heap_start = stack_end;
    write!(
        w,
        "\t(global $stack_bottom (mut i32) (i32.const {}))\n",
        stack_start
    )?;
    write!(w, "\t(global $stack_top i32 (i32.const {}))\n", stack_end)?;
    write!(
        w,
        "\t(global $heap_start (mut i32) (i32.const {}))\n",
        heap_start
    )?;
    Ok(())
}

fn write_runtime_functions<W: Write>(w: &mut W) -> io::Result<()> {
    let runtime = fs::read_to_string(RUNTIME_PATH)?;
    write!(w, "\n{}\n", runtime)?;
    Ok(())
}

fn write_static_data<W: Write>(
    w: &mut W,
    static_data: &StaticData,
) -> io::Result<()> {
    let offset_data = 0;
    write!(
        w,
        "\t(data (i32.const {}) {})\n",
        offset_data,
        WebassemblyString(static_data.data())
    )?;
    Ok(())
}

fn write_function<W: Write>(
    w: &mut W,
    program: &Program,
    idx: usize,
) -> io::Result<()> {
    let static_data = program.static_data();
    let function = &program.functions()[idx];
    let locals = LocalPlacesInfo::extract(function, program);
    let mut next_block_num = 1;
    let mut block_stack: Vec<i32> = vec![];

    write!(w, ";; {}\n", function.name())?;
    write!(w, "\t(func $fun{} ", idx)?;
    if let Some(name) = function.export_name() {
        write!(w, "(export \"{}\") ", name)?;
    }
    write!(
        w,
        "(param $param_head i32) (param $persistent_bottom i32) (result i32) (local $tmp i32) (local $retval i32)\n"
    )?;

    // function prologue
    if let Some(ref locals) = locals {
        write!(w, "\t\t;; start of function prologue\n")?;
        match locals.strategy() {
            LocalStrategy::Stack => {
                write!(w, "\t\ti32.const {}\n", locals.len())?;
                write!(w, "\t\tcall $inc_stack_bottom\n")?;
            }
            LocalStrategy::Heap => {
                if function
                    .attributes()
                    .contains(&FunctionAttribute::CreatesPersistentPlaces)
                {
                    // if persistent, reserve space for all the lambdas on the top-level function
                    write!(w, "\t\ti32.const {}\n", locals.len())?;
                    write!(w, "\t\tcall $alloc_heap\n")?;
                    write!(w, "\t\tlocal.set $persistent_bottom\n")?;
                } // lambdas themselves don't need to reserve anything in the prologue
            }
        }
        write!(w, "\t\t;; end of function prologue\n")?;
    }

    write!(w, "\t\t(block $body\n")?;

    for &instruction in function.instructions() {
        write!(w, "\t\t;; {:?}\n", instruction)?;
        match instruction {
            Instruction::LoadData { data, to } => {
                write_load_place_self_address(w, &locals, to)?;
                write!(w, "\t\t\ti32.const {}\n", data.offset())?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::LoadTypeTag { of, to } => {
                write_load_place_self_address(w, &locals, to)?;
                write_load_place_referee(w, &locals, of)?;
                write!(w, "\t\t\ti32.load\n")?;
                write!(w, "\t\t\tcall $make_num\n")?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::LoadCar { list, to } => {
                write_load_place_self_address(w, &locals, to)?;
                write_load_place_referee(w, &locals, list)?;
                // skip one to go to car after type and load
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::LoadCdr { list, to } => {
                write_load_place_self_address(w, &locals, to)?;
                write_load_place_referee(w, &locals, list)?;
                // skip two to go to cdr after type and car and load
                write!(w, "\t\t\ti32.const {}\n", 2 * mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::Cons { car, cdr, to } => {
                // type tag, car address, cdr address
                write_heap_alloc(w, mem::size_of::<i32>() * 3)?;
                write!(w, "\t\t\tlocal.set $tmp\n")?; // remember the allocation

                // write tag at offset 0
                write!(w, "\t\t\tlocal.get $tmp\n")?;
                let tag = IrDataType::ListNode.to_u32();
                write!(w, "\t\t\ti32.const {}\n", tag)?;
                write!(w, "\t\t\ti32.store\n")?;

                // load car address and write at offset 1
                write!(w, "\t\t\tlocal.get $tmp\n")?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write_load_place_referee(w, &locals, car)?;
                write!(w, "\t\t\ti32.store\n")?;

                // load cdr address and write at offset 2
                write!(w, "\t\t\tlocal.get $tmp\n")?;
                write!(w, "\t\t\ti32.const {}\n", 2 * mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write_load_place_referee(w, &locals, cdr)?;
                write!(w, "\t\t\ti32.store\n")?;

                // finally, remember the list in the target place
                write_load_place_self_address(w, &locals, to)?;
                write!(w, "\t\t\tlocal.get $tmp\n")?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::WritePlace { from, to } => {
                write_load_place_self_address(w, &locals, to)?;
                write_load_place_referee(w, &locals, from)?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::Call {
                function,
                params,
                to,
            } => {
                write!(
                    w,
                    "\t\t\t;; calling {}\n",
                    program.resolve_function_addr(function).name()
                )?;
                write_load_place_self_address(w, &locals, to)?;
                write_load_place_referee(w, &locals, params)?;
                write!(w, "\t\t\ti32.const 0\n")?; // target of direct call never uses persistent storage, so 0
                write!(w, "\t\t\tcall $fun{}\n", function.to_i32())?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::CallIndirect {
                function,
                params,
                to,
            } => {
                write_load_place_self_address(w, &locals, to)?;
                write_load_place_referee(w, &locals, function)?;
                write_load_place_referee(w, &locals, params)?;
                // persistent parameter comes from the storage of the function
                write!(w, "\t\t\tcall $call_function\n")?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::Return { value } => {
                // keep the return value on the stack when branching out of body
                write_load_place_referee(w, &locals, value)?;
                write!(w, "\t\t\tlocal.set $retval\n")?;
                // branch out of body and deallocate stack frame, leaving return value in place
                write!(w, "\t\t\tbr $body\n")?;
            }
            Instruction::CreateFunction { to, function } => {
                write!(
                    w,
                    "\t\t\t;; creating function with code at {}\n",
                    program.resolve_function_idx(function).name()
                )?;
                write_load_place_self_address(w, &locals, to)?;
                write!(w, "\t\t\ti32.const {}\n", function.to_u32())?;
                write!(w, "\t\t\tlocal.get $persistent_bottom\n")?;
                write!(w, "\t\t\tcall $make_function\n")?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::CallPrint { string } => {
                write_load_place_referee(w, &locals, string)?;
                write!(w, "\t\t\tlocal.set $tmp\n")?; // remember the string start address
                write!(w, "\t\t\tlocal.get $tmp\n")?;
                write!(w, "\t\t\ti32.const {}\n", 2 * mem::size_of::<i32>())?; // skip the tag and length and go to the character data
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\tlocal.get $tmp\n")?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?; // then load the length in bytes
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                write!(w, "\t\t\tcall $log\n")?;
            }
            Instruction::ConsumeParam { to } => {
                // load address of target place
                write_load_place_self_address(w, &locals, to)?;
                // load the passed location of argument list
                write!(w, "\t\t\tlocal.get $param_head\n")?;
                // skip type tag and go to car, load it, and store it in target place
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                write!(w, "\t\t\ti32.store\n")?;
                // load the passed location of argument list again
                write!(w, "\t\t\tlocal.get $param_head\n")?;
                // skip type tag and go to cdr, load it, and store it as new argument list
                write!(w, "\t\t\ti32.const {}\n", 2 * mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                write!(w, "\t\t\tlocal.set $param_head\n")?;
            }
            Instruction::ConsumeRest { to } => {
                // load address of target place
                write_load_place_self_address(w, &locals, to)?;
                // load the passed location of argument list
                write!(w, "\t\t\tlocal.get $param_head\n")?;
                // and directly store a reference to the list
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::ConcatStringLike { left, right, to } => {
                write_load_place_self_address(w, &locals, to)?;
                write_load_place_referee(w, &locals, left)?;
                write_load_place_referee(w, &locals, right)?;
                write!(w, "\t\t\tcall $concat_strings\n")?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::Add { left, right, to } => {
                write_load_place_self_address(w, &locals, to)?;
                // number on the left after type tag, number on the right after type tag
                write_load_place_referee(w, &locals, left)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                write_load_place_referee(w, &locals, right)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                // add together
                write!(w, "\t\t\ti32.add")?;
                // create new number and save address into to
                write!(w, "\t\t\tcall $make_num")?;
                write!(w, "\t\t\ti32.store")?;
            }
            Instruction::Sub { left, right, to } => {
                write_load_place_self_address(w, &locals, to)?;
                // number on the left after type tag, number on the right after type tag
                write_load_place_referee(w, &locals, left)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                write_load_place_referee(w, &locals, right)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                // subtract
                write!(w, "\t\t\ti32.sub\n")?;
                // create new number and save address into to
                write!(w, "\t\t\tcall $make_num\n")?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::Mul { left, right, to } => {
                write_load_place_self_address(w, &locals, to)?;
                // number on the left after type tag, number on the right after type tag
                write_load_place_referee(w, &locals, left)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                write_load_place_referee(w, &locals, right)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                // multiply
                write!(w, "\t\t\ti32.mul\n")?;
                // create new number and save address into to
                write!(w, "\t\t\tcall $make_num\n")?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::Div { left, right, to } => {
                write_load_place_self_address(w, &locals, to)?;
                // number on the left after type tag, number on the right after type tag
                write_load_place_referee(w, &locals, left)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                write_load_place_referee(w, &locals, right)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                // signed division
                write!(w, "\t\t\ti32.div_s\n")?;
                // create new number and save address into to
                write!(w, "\t\t\tcall $make_num\n")?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::Eq { left, right, to } => {
                write_load_place_self_address(w, &locals, to)?;
                // true value is address of T, false value is address of nil
                write!(
                    w,
                    "\t\t\ti32.const {}\n",
                    static_data.t_data().offset()
                )?;
                write!(
                    w,
                    "\t\t\ti32.const {}\n",
                    static_data.nil_data().offset()
                )?;
                // number on the left after type tag, number on the right after type tag
                write_load_place_referee(w, &locals, left)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                write_load_place_referee(w, &locals, right)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                // perform check and leave address of T or nil on stack after target address, then store
                write!(w, "\t\t\ti32.eq\n")?;
                write!(w, "\t\t\tselect\n")?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::Ne { left, right, to } => {
                write_load_place_self_address(w, &locals, to)?;
                // true value is address of T, false value is address of nil
                write!(
                    w,
                    "\t\t\ti32.const {}\n",
                    static_data.t_data().offset()
                )?;
                write!(
                    w,
                    "\t\t\ti32.const {}\n",
                    static_data.nil_data().offset()
                )?;
                // number on the left after type tag, number on the right after type tag
                write_load_place_referee(w, &locals, left)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                write_load_place_referee(w, &locals, right)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                // perform check and leave address of T or nil on stack after target address, then store
                write!(w, "\t\t\ti32.ne\n")?;
                write!(w, "\t\t\tselect\n")?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::Lt { left, right, to } => {
                write_load_place_self_address(w, &locals, to)?;
                // true value is address of T, false value is address of nil
                write!(
                    w,
                    "\t\t\ti32.const {}\n",
                    static_data.t_data().offset()
                )?;
                write!(
                    w,
                    "\t\t\ti32.const {}\n",
                    static_data.nil_data().offset()
                )?;
                // number on the left after type tag, number on the right after type tag
                write_load_place_referee(w, &locals, left)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                write_load_place_referee(w, &locals, right)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                // perform check and leave address of T or nil on stack after target address, then store
                write!(w, "\t\t\ti32.lt_s\n")?;
                write!(w, "\t\t\tselect\n")?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::Gt { left, right, to } => {
                write_load_place_self_address(w, &locals, to)?;
                // true value is address of T, false value is address of nil
                write!(
                    w,
                    "\t\t\ti32.const {}\n",
                    static_data.t_data().offset()
                )?;
                write!(
                    w,
                    "\t\t\ti32.const {}\n",
                    static_data.nil_data().offset()
                )?;
                // number on the left after type tag, number on the right after type tag
                write_load_place_referee(w, &locals, left)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                write_load_place_referee(w, &locals, right)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                // perform check and leave address of T or nil on stack after target address, then store
                write!(w, "\t\t\ti32.gt_s\n")?;
                write!(w, "\t\t\tselect\n")?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::Lte { left, right, to } => {
                write_load_place_self_address(w, &locals, to)?;
                // true value is address of T, false value is address of nil
                write!(
                    w,
                    "\t\t\ti32.const {}\n",
                    static_data.t_data().offset()
                )?;
                write!(
                    w,
                    "\t\t\ti32.const {}\n",
                    static_data.nil_data().offset()
                )?;
                // number on the left after type tag, number on the right after type tag
                write_load_place_referee(w, &locals, left)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                write_load_place_referee(w, &locals, right)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                // perform check and leave address of T or nil on stack after target address, then store
                write!(w, "\t\t\ti32.le_s\n")?;
                write!(w, "\t\t\tselect\n")?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::Gte { left, right, to } => {
                write_load_place_self_address(w, &locals, to)?;
                // true value is address of T, false value is address of nil
                write!(
                    w,
                    "\t\t\ti32.const {}\n",
                    static_data.t_data().offset()
                )?;
                write!(
                    w,
                    "\t\t\ti32.const {}\n",
                    static_data.nil_data().offset()
                )?;
                // number on the left after type tag, number on the right after type tag
                write_load_place_referee(w, &locals, left)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                write_load_place_referee(w, &locals, right)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;
                // perform check and leave address of T or nil on stack after target address, then store
                write!(w, "\t\t\ti32.ge_s\n")?;
                write!(w, "\t\t\tselect\n")?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::Break { block_up } => {
                let target_block =
                    block_stack[block_stack.len() - (block_up as usize)];
                write!(w, "\t\t\tbr $block_{}_end\n", target_block)?;
            }
            Instruction::Continue { block_up } => {
                let target_block =
                    block_stack[block_stack.len() - (block_up as usize)];
                write!(w, "\t\t\tbr $block_{}_start\n", target_block)?;
            }
            Instruction::BreakIfNotNil {
                block_up,
                if_not_nil,
            } => {
                write_load_place_referee(w, &locals, if_not_nil)?;
                let target_block =
                    block_stack[block_stack.len() - (block_up as usize)];
                write!(w, "\t\t\tbr_if $block_{}_end\n", target_block)?;
            }
            Instruction::BreakIfNil { if_nil, block_up } => {
                write_load_place_referee(w, &locals, if_nil)?;
                write!(
                    w,
                    "\t\t\ti32.const {}\n",
                    static_data.nil_data().offset()
                )?;
                write!(w, "\t\t\ti32.eq\n")?;
                let target_block =
                    block_stack[block_stack.len() - (block_up as usize)];
                write!(w, "\t\t\tbr_if $block_{}_end\n", target_block)?;
            }
            Instruction::ContinueIfNotNil {
                block_up,
                if_not_nil,
            } => {
                write_load_place_referee(w, &locals, if_not_nil)?;
                let target_block =
                    block_stack[block_stack.len() - (block_up as usize)];
                write!(w, "\t\t\tbr_if $block_{}_start\n", target_block)?;
            }
            Instruction::EnterBlock => {
                write!(
                    w,
                    "\t\t\t(loop $block_{}_start (block $block_{}_end\n",
                    next_block_num, next_block_num
                )?;
                block_stack.push(next_block_num);
                next_block_num += 1;
            }
            Instruction::ExitBlock => {
                write!(w, "\t\t\t));; end block\n")?;
                block_stack.pop();
            }
            Instruction::NilIfZero { check, to } => {
                write_load_place_self_address(w, &locals, to)?;

                // load number address as first alternative
                write_load_place_referee(w, &locals, check)?;

                // and as the alternative load address of nil
                write!(
                    w,
                    "\t\t\ti32.const {}\n",
                    static_data.nil_data().offset()
                )?;

                // load the number value as the test
                write_load_place_referee(w, &locals, check)?;
                write!(w, "\t\t\ti32.const {}\n", mem::size_of::<i32>())?;
                write!(w, "\t\t\ti32.add\n")?;
                write!(w, "\t\t\ti32.load\n")?;

                // and select nil address or the original number address based on the value being zero or not
                write!(w, "\t\t\tselect\n")?;
                write!(w, "\t\t\ti32.store\n")?;
            }
            Instruction::Panic => {
                write!(w, "\t\t\tunreachable\n")?;
            }
        }
    }

    // end of body block
    write!(w, "\t\t\t);; end body\n")?;

    // function epilogue
    write!(w, "\t\t;; start of function epilogue\n")?;
    if let Some(ref locals) = locals {
        if let LocalStrategy::Stack = locals.strategy() {
            write!(w, "\t\ti32.const {}\n", -locals.len())?;
            write!(w, "\t\tcall $inc_stack_bottom\n")?;
        }
        write!(w, "\t\tlocal.get $retval\n")?;
    }
    write!(w, "\t\t;; end of function epilogue\n")?;

    write!(w, "\t)\n")?;
    Ok(())
}

/// Loads the address of a place itself, so that it can be overwritten.
fn write_load_place_self_address<W: Write>(
    w: &mut W,
    local_info: &Option<LocalPlacesInfo>,
    from: PlaceAddress,
) -> io::Result<()> {
    let offset = from.offset() as usize;

    match (from.mode(), local_info.as_ref().unwrap().strategy()) {
        // local variables are below the stack bottom that gets bumped on entry
        (AddressingMode::Local, LocalStrategy::Stack) => {
            write!(w, "\t\t\tglobal.get $stack_bottom\n")?;
            write!(
                w,
                "\t\t\ti32.const {}\n",
                (offset + mem::size_of::<i32>())
            )?;
            write!(w, "\t\t\ti32.sub\n")
        }
        // persistent are addressed from the beginning, so no extra needed
        (AddressingMode::Local, LocalStrategy::Heap) => {
            write!(w, "\t\t\tlocal.get $persistent_bottom\n")?;
            write!(w, "\t\t\ti32.const {}\n", offset)?;
            write!(w, "\t\t\ti32.add\n")
        }
        (AddressingMode::Global, _) => {
            write!(w, "\t\t\ti32.const {}\n", offset)
        }
    }
}

/// Loads the address that a place points to
fn write_load_place_referee<W: Write>(
    w: &mut W,
    local_info: &Option<LocalPlacesInfo>,
    from: PlaceAddress,
) -> io::Result<()> {
    write_load_place_self_address(w, local_info, from)?;
    write!(w, "\t\t\ti32.load\n")
}

/// Writes a heap allocation, the result being the start address of the allocation
fn write_heap_alloc<W: Write>(w: &mut W, size: usize) -> io::Result<()> {
    // just append to the back for now
    write!(w, "\t\t\tglobal.get $heap_start\n")?;
    write!(w, "\t\t\tglobal.get $heap_start\n")?;
    write!(w, "\t\t\ti32.const {}\n", size)?;
    write!(w, "\t\t\ti32.add\n")?;
    write!(w, "\t\t\tglobal.set $heap_start\n")?;
    Ok(())
}

struct WebassemblyString<'s>(&'s [u8]);

impl<'s> fmt::Display for WebassemblyString<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"")?;
        for &byte in self.0 {
            if ((byte.is_ascii_graphic() || byte.is_ascii_punctuation())
                && byte != b'\"')
                || byte == b' '
            {
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
