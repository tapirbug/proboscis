use std::mem;

use crate::ir::{
    Function, FunctionAttribute, Instruction, PlaceAddress, Program,
};

#[derive(Copy, Clone)]
pub enum LocalStrategy {
    /// Store locals on the heap and preserve them after returning.
    Heap,
    /// Store locals on the stack and don't preserve them.
    Stack,
}

impl LocalStrategy {
    fn choose(function: &Function) -> LocalStrategy {
        let attributes = function.attributes();
        if attributes.contains(&FunctionAttribute::AcceptsPersistentPlaces)
            || attributes.contains(&FunctionAttribute::CreatesPersistentPlaces)
        {
            LocalStrategy::Heap
        } else {
            LocalStrategy::Stack
        }
    }
}

pub struct LocalPlacesInfo {
    strategy: LocalStrategy,
    len: i32,
}

impl LocalPlacesInfo {
    /// The length of local space storage in bytes.
    ///
    /// The length is never 0.
    pub fn len(&self) -> i32 {
        self.len
    }

    pub fn strategy(&self) -> LocalStrategy {
        self.strategy
    }
}

impl LocalPlacesInfo {
    pub fn extract(function: &Function, program: &Program) -> Option<Self> {
        let mut acc = LocalSpaceAccumulator::new();
        for &inst in function.instructions() {
            consider_instruction(&mut acc, inst, program);
        }
        acc.finish().map(|len| LocalPlacesInfo {
            strategy: LocalStrategy::choose(function),
            len,
        })
    }
}

struct LocalSpaceAccumulator {
    max_offset: i32,
}

impl LocalSpaceAccumulator {
    fn new() -> Self {
        Self {
            // start with minus one as the max offset, so if it's empty
            // the end result will be 0
            max_offset: -(mem::size_of::<i32>() as i32),
        }
    }

    fn must_contain(&mut self, addr: PlaceAddress) {
        self.max_offset = self.max_offset.max(addr.offset());
    }

    fn finish(self) -> Option<i32> {
        (self.max_offset >= 0)
            .then(|| self.max_offset + (mem::size_of::<i32>() as i32))
    }
}

fn consider_instruction(
    locals: &mut LocalSpaceAccumulator,
    instruction: Instruction,
    program: &Program,
) {
    match instruction {
        Instruction::CreateFunction { function, to } => {
            // if the function creates a lambda, it must reserve for the places in the lambda
            locals.must_contain(to);
            let function = program.resolve_function_idx(function);
            for &inst in function.instructions() {
                consider_instruction(locals, inst, program);
            }
        }
        Instruction::Call { params, to, .. } => {
            locals.must_contain(params);
            locals.must_contain(to);
        }
        Instruction::CallIndirect {
            function,
            params,
            to,
        } => {
            locals.must_contain(function);
            locals.must_contain(params);
            locals.must_contain(to);
        }
        Instruction::CallPrint { string } => {
            locals.must_contain(string);
        }
        Instruction::Return { value } => {
            locals.must_contain(value);
        }
        Instruction::EnterBlock => {}
        Instruction::Continue { .. } => {}
        Instruction::Break { .. } => {}
        Instruction::BreakIfNotNil { if_not_nil, .. } => {
            locals.must_contain(if_not_nil);
        }
        Instruction::BreakIfNil { if_nil, .. } => {
            locals.must_contain(if_nil);
        }
        Instruction::ContinueIfNotNil { if_not_nil, .. } => {
            locals.must_contain(if_not_nil);
        }
        Instruction::ExitBlock => {}
        Instruction::ConsumeParam { to } => {
            locals.must_contain(to);
        }
        Instruction::ConsumeRest { to } => {
            locals.must_contain(to);
        }
        Instruction::LoadData { to, .. } => {
            locals.must_contain(to);
        }
        Instruction::WritePlace { from, to } => {
            locals.must_contain(from);
            locals.must_contain(to);
        }
        Instruction::Cons { car, cdr, to } => {
            locals.must_contain(car);
            locals.must_contain(cdr);
            locals.must_contain(to);
        }
        Instruction::LoadCar { list, to } => {
            locals.must_contain(list);
            locals.must_contain(to);
        }
        Instruction::LoadCdr { list, to } => {
            locals.must_contain(list);
            locals.must_contain(to);
        }
        Instruction::ConcatStringLike { left, right, to } => {
            locals.must_contain(left);
            locals.must_contain(right);
            locals.must_contain(to);
        }
        Instruction::Add { left, right, to } => {
            locals.must_contain(left);
            locals.must_contain(right);
            locals.must_contain(to);
        }
        Instruction::Sub { left, right, to } => {
            locals.must_contain(left);
            locals.must_contain(right);
            locals.must_contain(to);
        }
        Instruction::Mul { left, right, to } => {
            locals.must_contain(left);
            locals.must_contain(right);
            locals.must_contain(to);
        }
        Instruction::Div { left, right, to } => {
            locals.must_contain(left);
            locals.must_contain(right);
            locals.must_contain(to);
        }
        Instruction::Eq { left, right, to } => {
            locals.must_contain(left);
            locals.must_contain(right);
            locals.must_contain(to);
        }
        Instruction::Ne { left, right, to } => {
            locals.must_contain(left);
            locals.must_contain(right);
            locals.must_contain(to);
        }
        Instruction::Gt { left, right, to } => {
            locals.must_contain(left);
            locals.must_contain(right);
            locals.must_contain(to);
        }
        Instruction::Lt { left, right, to } => {
            locals.must_contain(left);
            locals.must_contain(right);
            locals.must_contain(to);
        }
        Instruction::Gte { left, right, to } => {
            locals.must_contain(left);
            locals.must_contain(right);
            locals.must_contain(to);
        }
        Instruction::Lte { left, right, to } => {
            locals.must_contain(left);
            locals.must_contain(right);
            locals.must_contain(to);
        }
        Instruction::NilIfZero { check, to } => {
            locals.must_contain(check);
            locals.must_contain(to);
        }
        Instruction::LoadTypeTag { of, to } => {
            locals.must_contain(of);
            locals.must_contain(to);
        }
        Instruction::Panic => {}
    }
}
