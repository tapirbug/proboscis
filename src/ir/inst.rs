use super::{
    data::DataAddress, func::StaticFunctionAddress, place::PlaceAddress,
};
use std::mem;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Call {
        function: StaticFunctionAddress,
        params: PlaceAddress,
        /// Where to write the return value
        to: PlaceAddress, // could extend here to switch the stack or make it an extra instruction
    },
    /// Builtin to print a string, with no typechecking.
    CallPrint {
        string: PlaceAddress
    },
    // copies the given place address to the return value and returns from
    // the function
    Return {
        value: PlaceAddress,
    },
    // mark the beginning of a block in code
    EnterBlock,
    // start the block `block_up` relative to the current point in code
    // again from the beginning.
    Continue {
        block_up: u32,
    },
    // exit the block `block_up` relative to the current point in code
    // early.
    Break {
        block_up: u32,
    },
    /// Execute the next statement only if a place points to nil.
    IfNil {
        check: PlaceAddress,
    },
    /// Execute the next statement only if a place points to something else
    /// than nil.
    IfNotNil {
        check: PlaceAddress,
    },
    // mark the end of a block in code
    ExitBlock,
    /// read a param from the current function and write a reference to it to
    /// a place.
    /// Modify the params to include only the rest after the consumed
    /// parameter.
    ConsumeParam {
        to: PlaceAddress,
    },
    /// Write a reference to a constant value to a place.
    ///
    /// This is also used to write nil.
    LoadData {
        data: DataAddress,
        to: PlaceAddress,
    },
    /// Copies a place to another place.
    WritePlace {
        from: PlaceAddress,
        to: PlaceAddress,
    },
    /// Create a new list from car and cdr and write a reference to it to a
    /// place.
    ///
    /// Cdr is not checked for type but should be a list.
    Cons {
        car: PlaceAddress,
        cdr: PlaceAddress,
        to: PlaceAddress,
    },
    /// Gets the car (head) part of a list, without any typechecking,
    /// and writes it to a place.
    LoadCar {
        list: PlaceAddress,
        to: PlaceAddress,
    },
    /// Gets the cdr (tail list) part of a list, without any typechecking,
    /// and writes it to a place.
    LoadCdr {
        list: PlaceAddress,
        to: PlaceAddress,
    },
}

pub struct InstructionBuilder {
    instructions: Vec<Instruction>,
}

impl InstructionBuilder {
    pub fn new() -> Self {
        Self {
            instructions: vec![],
        }
    }

    pub fn call(
        &mut self,
        function: StaticFunctionAddress,
        params: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::Call {
            function,
            params,
            to,
        });
        self
    }

    pub fn call_print(&mut self, value: PlaceAddress) -> &mut Self {
        self.instructions.push(Instruction::CallPrint { string: value });
        self
    }

    pub fn add_return(&mut self, value: PlaceAddress) -> &mut Self {
        self.instructions.push(Instruction::Return { value });
        self
    }

    pub fn enter_block(&mut self) -> &mut Self {
        self.instructions.push(Instruction::EnterBlock);
        self
    }

    pub fn add_continue(&mut self, block_up: u32) -> &mut Self {
        self.instructions.push(Instruction::Continue { block_up });
        self
    }

    pub fn add_break(&mut self, block_up: u32) -> &mut Self {
        self.instructions.push(Instruction::Break { block_up });
        self
    }

    pub fn if_nil(&mut self, check: PlaceAddress) -> &mut Self {
        self.instructions.push(Instruction::IfNil { check });
        self
    }

    pub fn if_not_nil(&mut self, check: PlaceAddress) -> &mut Self {
        self.instructions.push(Instruction::IfNotNil { check });
        self
    }

    pub fn exit_block(&mut self) -> &mut Self {
        self.instructions.push(Instruction::ExitBlock);
        self
    }

    pub fn consume_param(&mut self, to: PlaceAddress) -> &mut Self {
        self.instructions.push(Instruction::ConsumeParam { to });
        self
    }

    pub fn load_data(
        &mut self,
        data: DataAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::LoadData { data, to });
        self
    }

    pub fn write_place(
        &mut self,
        from: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::WritePlace { from, to });
        self
    }
    pub fn cons(
        &mut self,
        car: PlaceAddress,
        cdr: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::Cons { car, cdr, to });
        self
    }

    pub fn load_car(
        &mut self,
        list: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::LoadCar { list, to });
        self
    }

    pub fn load_cdr(
        &mut self,
        list: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::LoadCdr { list, to });
        self
    }

    /// Build the function, clearing the builder for the next function.
    pub fn build(&mut self) -> Vec<Instruction> {
        mem::take(&mut self.instructions)
    }
}

/*
max-2:
    alloc-places 5
    consume-param place#0
    consume-param place#1
    load-data place#4 data#0
    cons car place#1 cdr place#4 to place#3 ;; place#4 = nil
    cons car place#0 cdr place#3 to place#3
    call function ">" params place#3 to place#3
    enter-block
    enter-block
    if-not-nil place#3
        break 0
    write-place from place#0 to place#4
    break 1
    exit-block
    write-place from place#1 to place#4
    exit-block
    return place#4


*/
