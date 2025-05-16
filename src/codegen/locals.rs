use std::mem;

use crate::ir::{AddressingMode, Instruction, PlaceAddress};

struct LocalSpaceCalculator {
    max_offset: i32,
}

impl LocalSpaceCalculator {
    fn new() -> Self {
        Self {
            // start with minus one as the max offset, so if it's empty
            // the end result will be 0
            max_offset: -(mem::size_of::<i32>() as i32),
        }
    }

    fn must_contain(&mut self, addr: PlaceAddress) {
        match addr.mode() {
            AddressingMode::Global => {} // don't care about globals for counting locals
            AddressingMode::Local => {
                self.max_offset = self.max_offset.max(addr.offset());
            }
        }
    }

    fn local_space_bytes(&self) -> i32 {
        // include space for the last place
        let bytes = self.max_offset + (mem::size_of::<i32>() as i32);
        bytes
    }
}

pub fn local_places_byte_len(instructions: &[Instruction]) -> i32 {
    let mut locals = LocalSpaceCalculator::new();
    for &inst in instructions {
        match inst {
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
            Instruction::CreateClosure { to } => {
                locals.must_contain(to);
            }
            Instruction::CreateFunction { closure, to, .. } => {
                locals.must_contain(closure);
                locals.must_contain(to);
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
        }
    }
    locals.local_space_bytes()
}
