use std::mem;

use crate::ir::{data::DataAddress, inmem::append_string};

use super::{
    inmem::{append_identifier, append_list_node, append_nil, append_place, append_sint32},
    place::PlaceAddress,
};

pub struct StaticData {
    static_data: Vec<u8>,
}

pub struct StaticDataBuilder {
    static_data: Vec<u8>,
}

impl StaticData {
    pub fn data(&self) -> &[u8] {
        &self.static_data
    }
}

impl StaticDataBuilder {
    pub fn new() -> Self {
        StaticDataBuilder {
            static_data: vec![],
        }
    }

    fn top_static_data_address(&self) -> DataAddress {
        DataAddress::new_unsafe(self.static_data.len().try_into().unwrap())
    }

    fn top_static_place_address(&self) -> PlaceAddress {
        PlaceAddress::new_global(self.static_data.len().try_into().unwrap())
    }

    pub fn static_nil(&mut self) -> DataAddress {
        let address = self.top_static_data_address();
        // car and cdr of nil are nil
        append_nil(&mut self.static_data, address.offset()).unwrap();
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

    /// Append a new static identifier as data without checking for duplicates.
    pub fn static_identifier(&mut self, data: &str) -> DataAddress {
        let address = self.top_static_data_address();
        append_identifier(&mut self.static_data, data).unwrap();
        address
    }

    pub fn static_list_node(&mut self, car: DataAddress, cdr: DataAddress) -> DataAddress {
        let address = self.top_static_data_address();
        append_list_node(&mut self.static_data, car, cdr).unwrap();
        address
    }

    pub fn static_place(&mut self, data: DataAddress) -> PlaceAddress {
        let address = self.top_static_place_address();
        append_place(&mut self.static_data, data).unwrap();
        address
    }

    /// Build the static data, consuming the contents of the builder and
    /// leaving it empty.
    pub fn build(&mut self) -> StaticData {
        StaticData {
            static_data: mem::take(&mut self.static_data),
        }
    }
}
