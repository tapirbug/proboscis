use std::{collections::HashMap, mem};

use crate::ir::{DataAddress, FunctionsBuilder, PlaceAddress, StaticDataBuilder, StaticFunctionAddress};

pub fn generate_builtin_functions<'i>(static_data: &'i mut StaticDataBuilder,
    functions: &'i mut FunctionsBuilder,
    function_addresses: &'i mut HashMap<String, StaticFunctionAddress>,
    nil_address: DataAddress,
    nil_place: PlaceAddress) {

    BuiltinGen::new(static_data, functions, function_addresses, nil_address, nil_place).generate_builtin_functions();
}

struct BuiltinGen<'i> {
    static_data: &'i mut StaticDataBuilder,
    functions: &'i mut FunctionsBuilder,
    function_addresses: &'i mut HashMap<String, StaticFunctionAddress>,
    nil_address: DataAddress,
    nil_place: PlaceAddress,
}

impl<'i> BuiltinGen<'i> {
    fn new(static_data: &'i mut StaticDataBuilder,
    functions: &'i mut FunctionsBuilder,
    function_addresses: &'i mut HashMap<String, StaticFunctionAddress>,
    nil_address: DataAddress,
    nil_place: PlaceAddress) -> Self {
        Self {
            static_data,
            function_addresses,
            functions,
            nil_address,
            nil_place
        }
    }

    fn generate_builtin_functions(&mut self) {
        self.generate_builtin_format();
        self.generate_builtin_type_tag_of();
        self.generate_builtin_concat_string_like_2();
        self.generate_cons();
        self.generate_car();
        self.generate_cdr();
        self.generate_add2();
        self.generate_sub2();
        self.generate_nil_if_0();
    }

    fn generate_builtin_format(&mut self) {
        let format_addr = self.functions.add_private_function();
        self.function_addresses.insert("format".to_string(), format_addr);
        let working_place = PlaceAddress::new_local(0);
        self.functions.implement_function(format_addr)
            .consume_param(working_place) // ignore the t parameter
            .consume_param(working_place) // this is the format string, which we just print verbatim for now
            .call_print(working_place)
            .add_return(self.nil_place);
    }

    fn generate_builtin_type_tag_of(&mut self) {
        let addr = self.functions.add_private_function();
        self.function_addresses.insert("type-tag-of".to_string(), addr);
        let working_place = PlaceAddress::new_local(0);
        self.functions.implement_function(addr)
            .consume_param(working_place)
            .load_type_tag(working_place, working_place)
            .add_return(working_place);
    }

    fn generate_builtin_concat_string_like_2(&mut self) {
        let addr = self.functions.add_private_function();
        self.function_addresses.insert("concat-string-like-2".to_string(), addr);
        let left_place = PlaceAddress::new_local(0);
        let right_place = PlaceAddress::new_local(mem::size_of::<i32>() as i32);
        self.functions.implement_function(addr)
            .consume_param(left_place)
            .consume_param(right_place)
            .concat_string_like(left_place, right_place, left_place)
            .add_return(left_place);
    }

    fn generate_cons(&mut self) {
        let addr = self.functions.add_private_function();
        self.function_addresses.insert("cons".to_string(), addr);
        let left_place = PlaceAddress::new_local(0);
        let right_place = PlaceAddress::new_local(mem::size_of::<i32>() as i32);
        self.functions.implement_function(addr)
            .consume_param(left_place)
            .consume_param(right_place)
            .cons(left_place, right_place, left_place)
            .add_return(left_place);
    }

    fn generate_car(&mut self) {
        let addr = self.functions.add_private_function();
        self.function_addresses.insert("car".to_string(), addr);
        let place = PlaceAddress::new_local(0);
        self.functions.implement_function(addr)
            .consume_param(place)
            .load_car(place, place)
            .add_return(place);
    }

    fn generate_cdr(&mut self) {
        let addr = self.functions.add_private_function();
        self.function_addresses.insert("cdr".to_string(), addr);
        let place = PlaceAddress::new_local(0);
        self.functions.implement_function(addr)
            .consume_param(place)
            .load_cdr(place, place)
            .add_return(place);
    }

    fn generate_add2(&mut self) {
        let addr = self.functions.add_private_function();
        self.function_addresses.insert("add-2".to_string(), addr);
        let left = PlaceAddress::new_local(0);
        let right = PlaceAddress::new_local(mem::size_of::<i32>() as i32);
        self.functions.implement_function(addr)
            .consume_param(left)
            .consume_param(right)
            .add(left, right, left)
            .add_return(left);
    }

    fn generate_sub2(&mut self) {
        let addr = self.functions.add_private_function();
        self.function_addresses.insert("sub-2".to_string(), addr);
        let left = PlaceAddress::new_local(0);
        let right = PlaceAddress::new_local(mem::size_of::<i32>() as i32);
        self.functions.implement_function(addr)
            .consume_param(left)
            .consume_param(right)
            .sub(left, right, left)
            .add_return(left);
    }

    fn generate_nil_if_0(&mut self) {
        let addr = self.functions.add_private_function();
        self.function_addresses.insert("nil-if-0".to_string(), addr);
        let place = PlaceAddress::new_local(0);
        self.functions.implement_function(addr)
            .consume_param(place)
            .nil_if_zero(place, place)
            .add_return(place);
    }
}
