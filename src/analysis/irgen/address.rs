use std::mem;

use crate::ir::{AddressingMode, PlaceAddress};

/// Counts up addresses to generate local and persistent.
pub struct PlaceGenerator {
    mode: AddressingMode,
    next_address: i32,
}

impl PlaceGenerator {
    pub fn local() -> Self {
        Self::new(AddressingMode::Local)
    }

    pub fn persistent() -> Self {
        Self::new(AddressingMode::Persistent)
    }

    fn new(mode: AddressingMode) -> Self {
        PlaceGenerator {
            mode,
            next_address: 0,
        }
    }

    /// Gets the next place of the specified kind.
    pub fn next(&mut self) -> PlaceAddress {
        let address = self.next_address;
        self.next_address += (mem::size_of::<i32>() as i32);
        PlaceAddress::new(self.mode, address)
    }
}
