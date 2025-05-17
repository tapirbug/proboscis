use super::{
    FunctionTableIndex, data::DataAddress, func::StaticFunctionAddress,
    place::PlaceAddress,
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
    CallIndirect {
        function: PlaceAddress,
        params: PlaceAddress,
        to: PlaceAddress,
    },
    /// Builtin to print a string, with no typechecking.
    CallPrint {
        string: PlaceAddress,
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
    ContinueIfNotNil {
        if_not_nil: PlaceAddress,
        block_up: u32,
    },
    BreakIfNotNil {
        if_not_nil: PlaceAddress,
        block_up: u32,
    },
    BreakIfNil {
        if_nil: PlaceAddress,
        block_up: u32,
    },
    NilIfZero {
        check: PlaceAddress,
        to: PlaceAddress,
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
    ConsumeRest {
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
    /// Allocates a new function that calls the given function and does not
    /// use any persistent storage.
    ///
    /// A reference to the function is saved in to.
    CreateFunction {
        function: FunctionTableIndex,
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
    /// Creates a new number from adding two numbers.
    Add {
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    },
    /// Creates a new number from subtracting two numbers.
    Sub {
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    },
    /// Creates a new number from multiplying two numbers.
    Mul {
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    },
    /// Creates a new number from doing a truncating division of left by right.
    Div {
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    },
    /// If left == right, write T to the target place, otherwise NIL.
    Eq {
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    },
    /// If left != right, write T to the target place, otherwise NIL.
    Ne {
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    },
    /// If left < right, write T to the target place, otherwise NIL.
    Lt {
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    },
    /// If left > right, write T to the target place, otherwise NIL.
    Gt {
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    },
    /// If left <= right, write T to the target place, otherwise NIL.
    Lte {
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    },
    /// If left >= right, write T to the target place, otherwise NIL.
    Gte {
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    },
    /// Concatenate two strings or identifiers (or a mix) to form a new string.
    /// No typechecking.
    ConcatStringLike {
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    },
    /// Gets the type tag of the thing referred to by of, and writes an integer
    /// with the type tag to `to`.
    LoadTypeTag {
        of: PlaceAddress,
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

    pub fn call_indirect(
        &mut self,
        function: PlaceAddress,
        params: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::CallIndirect {
            function,
            params,
            to,
        });
        self
    }

    pub fn create_function(
        &mut self,
        function: FunctionTableIndex,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions
            .push(Instruction::CreateFunction { function, to });
        self
    }

    pub fn call_print(&mut self, value: PlaceAddress) -> &mut Self {
        self.instructions
            .push(Instruction::CallPrint { string: value });
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

    pub fn continue_if_not_nil(
        &mut self,
        block_up: u32,
        if_not_nil: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::ContinueIfNotNil {
            block_up,
            if_not_nil,
        });
        self
    }

    pub fn break_if_not_nil(
        &mut self,
        block_up: u32,
        if_not_nil: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::BreakIfNotNil {
            block_up,
            if_not_nil,
        });
        self
    }

    pub fn break_if_nil(
        &mut self,
        block_up: u32,
        if_nil: PlaceAddress,
    ) -> &mut Self {
        self.instructions
            .push(Instruction::BreakIfNil { if_nil, block_up });
        self
    }

    pub fn nil_if_zero(
        &mut self,
        check: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::NilIfZero { check, to });
        self
    }

    pub fn exit_block(&mut self) -> &mut Self {
        self.instructions.push(Instruction::ExitBlock);
        self
    }

    pub fn add(
        &mut self,
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::Add { left, right, to });
        self
    }

    pub fn sub(
        &mut self,
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::Sub { left, right, to });
        self
    }

    pub fn mul(
        &mut self,
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::Mul { left, right, to });
        self
    }

    pub fn div(
        &mut self,
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::Div { left, right, to });
        self
    }

    pub fn eq(
        &mut self,
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::Eq { left, right, to });
        self
    }

    pub fn ne(
        &mut self,
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::Ne { left, right, to });
        self
    }

    pub fn lt(
        &mut self,
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::Lt { left, right, to });
        self
    }

    pub fn gt(
        &mut self,
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::Gt { left, right, to });
        self
    }

    pub fn lte(
        &mut self,
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::Lte { left, right, to });
        self
    }

    pub fn gte(
        &mut self,
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::Gte { left, right, to });
        self
    }

    pub fn consume_param(&mut self, to: PlaceAddress) -> &mut Self {
        self.instructions.push(Instruction::ConsumeParam { to });
        self
    }

    pub fn consume_rest(&mut self, to: PlaceAddress) -> &mut Self {
        self.instructions.push(Instruction::ConsumeRest { to });
        self
    }

    pub fn concat_string_like(
        &mut self,
        left: PlaceAddress,
        right: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::ConcatStringLike {
            left,
            right,
            to,
        });
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

    pub fn load_type_tag(
        &mut self,
        of: PlaceAddress,
        to: PlaceAddress,
    ) -> &mut Self {
        self.instructions.push(Instruction::LoadTypeTag { of, to });
        self
    }

    /// Build the function, clearing the builder for the next function.
    pub fn build(&mut self) -> Vec<Instruction> {
        mem::take(&mut self.instructions)
    }
}
