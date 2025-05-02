use std::mem;

use crate::ir::{data::DataAddress, inmem::append_string};

use super::{
    inmem::{append_list_node, append_place, append_sint32},
    place::PlaceAddress,
};

pub struct StaticData {
    static_data: Vec<u8>,
    static_places: Vec<u8>,
}

pub struct StaticDataBuilder {
    static_data: Vec<u8>,
    static_places: Vec<u8>,
}

impl StaticData {
    pub fn data(&self) -> &[u8] {
        &self.static_data
    }

    pub fn places(&self) -> &[u8] {
        &self.static_places
    }
}

impl StaticDataBuilder {
    pub fn new() -> Self {
        StaticDataBuilder {
            static_data: vec![],
            static_places: vec![],
        }
    }

    fn top_static_data_address(&self) -> DataAddress {
        DataAddress::new_unsafe(self.static_data.len().try_into().unwrap())
    }

    fn top_static_place_address(&self) -> PlaceAddress {
        PlaceAddress::new_unsafe(
            self.static_places
                .iter()
                .map(|_| mem::size_of::<u32>() as i32)
                .sum(),
        )
    }

    pub fn static_nil(&mut self) -> DataAddress {
        let address = self.top_static_data_address();
        // car and cdr of nil are nil
        append_list_node(&mut self.static_data, address, address).unwrap();
        address
    }

    pub fn static_number(&mut self, number: i32) -> DataAddress {
        let address = self.top_static_data_address();
        append_sint32(&mut self.static_data, number).unwrap();
        address
    }

    /// Append a new static string as data without checking for duplicates.
    pub fn static_string(&mut self, data: &str) -> DataAddress {
        let address = self.top_static_data_address();
        append_string(&mut self.static_data, data).unwrap();
        address
    }

    pub fn static_list_node(
        &mut self,
        car: DataAddress,
        cdr: DataAddress,
    ) -> DataAddress {
        let address = self.top_static_data_address();
        append_list_node(&mut self.static_data, car, cdr).unwrap();
        address
    }

    pub fn static_place(&mut self, data: DataAddress) -> PlaceAddress {
        let address = self.top_static_place_address();
        append_place(&mut self.static_places, data).unwrap();
        address
    }

    /// Build the static data, consuming the contents of the builder and
    /// leaving it empty.
    pub fn build(&mut self) -> StaticData {
        StaticData {
            static_data: mem::take(&mut self.static_data),
            static_places: mem::take(&mut self.static_places),
        }
    }
}
