use std::mem;

use crate::ir::PlaceAddress;

/// Counts up addresses to generate local and persistent.
pub struct LocalPlaceGenerator {
    next_address: i32,
}

impl LocalPlaceGenerator {
    pub fn new() -> Self {
        LocalPlaceGenerator { next_address: 0 }
    }

    /// Gets the next place of the specified kind.
    pub fn next(&mut self) -> PlaceAddress {
        let address = self.next_address;
        self.next_address += mem::size_of::<i32>() as i32;
        PlaceAddress::new_local(address)
    }
}
