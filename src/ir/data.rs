#[derive(Debug, Clone, Copy)]
pub struct DataAddress(i32);

impl DataAddress {
    pub fn new_unsafe(from: i32) -> Self {
        Self(from)
    }

    pub fn to_le_bytes(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}
