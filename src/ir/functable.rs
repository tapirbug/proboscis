
#[derive(Debug, Copy, Clone)]
pub struct FunctionTableIndex {
    idx: u32
}

impl FunctionTableIndex {
    pub fn new_unsafe(idx: u32) -> Self {
        Self { idx }
    }

    pub fn to_u32(&self) -> u32 {
        self.idx
    }
}
