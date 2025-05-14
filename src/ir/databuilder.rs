use std::mem;

use crate::ir::{data::DataAddress, inmem::append_string};

use super::{
    inmem::{append_function, append_identifier, append_list_node, append_nil, append_place, append_sint32}, place::PlaceAddress, FunctionTableIndex, StaticFunctionAddress
};

pub struct StaticData {
    static_data: Vec<u8>,
    table_entries: Vec<StaticFunctionAddress>,
}

pub struct StaticDataBuilder {
    static_data: Vec<u8>,
    table_entries: Vec<StaticFunctionAddress>,
}

impl StaticData {
    pub fn data(&self) -> &[u8] {
        &self.static_data
    }

    pub fn table_entries(&self) -> &[StaticFunctionAddress] {
        &self.table_entries
    }
}

impl StaticDataBuilder {
    pub fn new() -> Self {
        StaticDataBuilder {
            static_data: vec![],
            table_entries: vec![]
        }
    }

    fn top_static_data_address(&self) -> DataAddress {
        DataAddress::new_unsafe(self.static_data.len().try_into().unwrap())
    }

    fn top_static_place_address(&self) -> PlaceAddress {
        PlaceAddress::new_global(self.static_data.len().try_into().unwrap())
    }

    fn top_table_idx(&self) -> FunctionTableIndex {
        FunctionTableIndex::new_unsafe(self.table_entries.len() as u32)
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

    pub fn static_function(&mut self, static_addr: StaticFunctionAddress) -> DataAddress {
        let address = self.top_static_data_address();
        let idx = self.top_table_idx();
        self.table_entries.push(static_addr);
        append_function(&mut self.static_data, idx.to_u32()).unwrap();
        address
    }

    pub fn function_table_entry(&mut self, static_addr: StaticFunctionAddress) -> FunctionTableIndex {
        let idx = self.top_table_idx();
        self.table_entries.push(static_addr);
        idx
    }

    /// Build the static data, consuming the contents of the builder and
    /// leaving it empty.
    pub fn build(&mut self) -> StaticData {
        StaticData {
            static_data: mem::take(&mut self.static_data),
            table_entries: mem::take(&mut self.table_entries)
        }
    }
}
