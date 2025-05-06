#[derive(Debug, Clone, Copy)]
pub struct PlaceAddress {
    mode: AddressingMode,
    /// Offset in bytes
    offset: i32
}

#[derive(Debug, Copy, Clone)]
pub enum AddressingMode {
    // accessing a local place
    Local,
    // accessing a place relative to the beginning of memory, used for global
    // variables
    Global
}

impl PlaceAddress {
    pub fn new_global(absolute_address: i32) -> Self {
        Self {
            mode: AddressingMode::Global,
            offset: absolute_address
        }
    }

    pub fn new_local(local_place_byte_offset: i32) -> Self {
        Self {
            mode: AddressingMode::Local,
            offset: local_place_byte_offset
        }
    }

    pub fn mode(self) -> AddressingMode {
        self.mode
    }

    pub fn offset(self) -> i32 {
        self.offset
    }
}
