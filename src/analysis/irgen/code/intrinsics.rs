use std::mem;

use crate::{analysis::irgen::scope::FunctionScope, ir::{FunctionsBuilder, PlaceAddress}};

/// Generates some builtin function that serve as a basis for other runtime
/// functions implemented in LISP.
///
/// While they can be used in user code, doing so is usually not safe or
/// compatible with any standard.
///
/// Most of the intrinsic functions are wrapped in runtime functions implemented
/// in LISP to offer functionality that more closely resembles Common LISP,
/// e.g. `intrinsic:add2` is used internally by the `+` function, and user code
/// should only use that function.
pub fn generate_intrinsic_functions<'i, 's>(
    functions: &'i mut FunctionsBuilder,
    function_scope: &'i mut FunctionScope<'s>,
    nil_place: PlaceAddress,
) {
    Intrinsics::new(functions, function_scope, nil_place)
        .generate_builtin_functions();
}

struct Intrinsics<'i, 's> {
    functions: &'i mut FunctionsBuilder,
    function_scope: &'i mut FunctionScope<'s>,
    nil_place: PlaceAddress,
}

impl<'i, 's> Intrinsics<'i, 's> {
    fn new(
        functions: &'i mut FunctionsBuilder,
        function_scope: &'i mut FunctionScope<'s>,
        nil_place: PlaceAddress,
    ) -> Self {
        Self {
            function_scope,
            functions,
            nil_place,
        }
    }

    fn generate_builtin_functions(&mut self) {
        self.generate_princ();
        self.generate_builtin_type_tag_of();
        self.generate_builtin_concat_string_like_2();
        self.generate_cons();
        self.generate_car();
        self.generate_cdr();
        self.generate_add2();
        self.generate_sub2();
        self.generate_mul2();
        self.generate_div2();
        self.generate_eq2();
        self.generate_ne2();
        self.generate_gt2();
        self.generate_lt2();
        self.generate_gte2();
        self.generate_lte2();
        self.generate_nil_if_0();
        self.generate_panic();
    }

    /// Function that prints a single argument that is a string or identifier (no typechecking)
    ///
    /// No builtin: prefix because this function does not have a wrapper in rt.
    fn generate_princ(&mut self) {
        let name = "intrinsic:princ";
        let format_addr = self.functions.add_private_function(name);
        self.function_scope.add_binding(name, format_addr);
        let working_place = PlaceAddress::new_local(0);
        self.functions
            .implement_function(format_addr)
            .consume_param(working_place)
            .call_print(working_place)
            .add_return(self.nil_place);
    }

    fn generate_builtin_type_tag_of(&mut self) {
        let name = "intrinsic:type-tag-of";
        let addr = self.functions.add_private_function(name);
        self.function_scope.add_binding(name, addr);
        let working_place = PlaceAddress::new_local(0);
        self.functions
            .implement_function(addr)
            .consume_param(working_place)
            .load_type_tag(working_place, working_place)
            .add_return(working_place);
    }

    fn generate_builtin_concat_string_like_2(&mut self) {
        let name = "intrinsic:concat-string-like-2";
        let addr = self.functions.add_private_function(name);
        self.function_scope.add_binding(name, addr);
        let left_place = PlaceAddress::new_local(0);
        let right_place =
            PlaceAddress::new_local(mem::size_of::<i32>() as i32);
        self.functions
            .implement_function(addr)
            .consume_param(left_place)
            .consume_param(right_place)
            .concat_string_like(left_place, right_place, left_place)
            .add_return(left_place);
    }

    fn generate_cons(&mut self) {
        let name = "intrinsic:cons";
        let addr = self.functions.add_private_function(name);
        self.function_scope.add_binding(name, addr);
        let left_place = PlaceAddress::new_local(0);
        let right_place =
            PlaceAddress::new_local(mem::size_of::<i32>() as i32);
        self.functions
            .implement_function(addr)
            .consume_param(left_place)
            .consume_param(right_place)
            .cons(left_place, right_place, left_place)
            .add_return(left_place);
    }

    fn generate_car(&mut self) {
        let name = "intrinsic:car";
        let addr = self.functions.add_private_function(name);
        self.function_scope.add_binding(name, addr);
        let place = PlaceAddress::new_local(0);
        self.functions
            .implement_function(addr)
            .consume_param(place)
            .load_car(place, place)
            .add_return(place);
    }

    fn generate_cdr(&mut self) {
        let name = "intrinsic:cdr";
        let addr = self.functions.add_private_function(name);
        self.function_scope.add_binding(name, addr);
        let place = PlaceAddress::new_local(0);
        self.functions
            .implement_function(addr)
            .consume_param(place)
            .load_cdr(place, place)
            .add_return(place);
    }

    fn generate_add2(&mut self) {
        let name = "intrinsic:add-2";
        let addr = self.functions.add_private_function(name);
        self.function_scope.add_binding(name, addr);
        let left = PlaceAddress::new_local(0);
        let right = PlaceAddress::new_local(mem::size_of::<i32>() as i32);
        self.functions
            .implement_function(addr)
            .consume_param(left)
            .consume_param(right)
            .add(left, right, left)
            .add_return(left);
    }

    fn generate_sub2(&mut self) {
        let name = "intrinsic:sub-2";
        let addr = self.functions.add_private_function(name);
        self.function_scope.add_binding(name, addr);
        let left = PlaceAddress::new_local(0);
        let right = PlaceAddress::new_local(mem::size_of::<i32>() as i32);
        self.functions
            .implement_function(addr)
            .consume_param(left)
            .consume_param(right)
            .sub(left, right, left)
            .add_return(left);
    }

    fn generate_mul2(&mut self) {
        let name = "intrinsic:mul-2";
        let addr = self.functions.add_private_function(name);
        self.function_scope.add_binding(name, addr);
        let left = PlaceAddress::new_local(0);
        let right = PlaceAddress::new_local(mem::size_of::<i32>() as i32);
        self.functions
            .implement_function(addr)
            .consume_param(left)
            .consume_param(right)
            .mul(left, right, left)
            .add_return(left);
    }

    fn generate_div2(&mut self) {
        let name = "intrinsic:div-2";
        let addr = self.functions.add_private_function(name);
        self.function_scope.add_binding(name, addr);
        let left = PlaceAddress::new_local(0);
        let right = PlaceAddress::new_local(mem::size_of::<i32>() as i32);
        self.functions
            .implement_function(addr)
            .consume_param(left)
            .consume_param(right)
            .div(left, right, left)
            .add_return(left);
    }

    fn generate_eq2(&mut self) {
        let name = "intrinsic:=-2";
        let addr = self.functions.add_private_function(name);
        self.function_scope.add_binding(name, addr);
        let left = PlaceAddress::new_local(0);
        let right = PlaceAddress::new_local(mem::size_of::<i32>() as i32);
        self.functions
            .implement_function(addr)
            .consume_param(left)
            .consume_param(right)
            .eq(left, right, left)
            .add_return(left);
    }

    fn generate_ne2(&mut self) {
        let name = "intrinsic:/=-2";
        let addr = self.functions.add_private_function(name);
        self.function_scope.add_binding(name, addr);
        let left = PlaceAddress::new_local(0);
        let right = PlaceAddress::new_local(mem::size_of::<i32>() as i32);
        self.functions
            .implement_function(addr)
            .consume_param(left)
            .consume_param(right)
            .ne(left, right, left)
            .add_return(left);
    }

    fn generate_lt2(&mut self) {
        let name = "intrinsic:<-2";
        let addr = self.functions.add_private_function(name);
        self.function_scope.add_binding(name, addr);
        let left = PlaceAddress::new_local(0);
        let right = PlaceAddress::new_local(mem::size_of::<i32>() as i32);
        self.functions
            .implement_function(addr)
            .consume_param(left)
            .consume_param(right)
            .lt(left, right, left)
            .add_return(left);
    }

    fn generate_gt2(&mut self) {
        let name = "intrinsic:>-2";
        let addr = self.functions.add_private_function(name);
        self.function_scope.add_binding(name, addr);
        let left = PlaceAddress::new_local(0);
        let right = PlaceAddress::new_local(mem::size_of::<i32>() as i32);
        self.functions
            .implement_function(addr)
            .consume_param(left)
            .consume_param(right)
            .gt(left, right, left)
            .add_return(left);
    }

    fn generate_lte2(&mut self) {
        let name = "intrinsic:<=-2";
        let addr = self.functions.add_private_function(name);
        self.function_scope.add_binding(name, addr);
        let left = PlaceAddress::new_local(0);
        let right = PlaceAddress::new_local(mem::size_of::<i32>() as i32);
        self.functions
            .implement_function(addr)
            .consume_param(left)
            .consume_param(right)
            .lte(left, right, left)
            .add_return(left);
    }

    fn generate_gte2(&mut self) {
        let name = "intrinsic:>=-2";
        let addr = self.functions.add_private_function(name);
        self.function_scope.add_binding(name, addr);
        let left = PlaceAddress::new_local(0);
        let right = PlaceAddress::new_local(mem::size_of::<i32>() as i32);
        self.functions
            .implement_function(addr)
            .consume_param(left)
            .consume_param(right)
            .gte(left, right, left)
            .add_return(left);
    }

    fn generate_nil_if_0(&mut self) {
        let name = "intrinsic:nil-if-0";
        let addr = self.functions.add_private_function(name);
        self.function_scope.add_binding(name, addr);
        let place = PlaceAddress::new_local(0);
        self.functions
            .implement_function(addr)
            .consume_param(place)
            .nil_if_zero(place, place)
            .add_return(place);
    }

    fn generate_panic(&mut self) {
        let name = "intrinsic:panic";
        let addr = self.functions.add_private_function(name);
        self.function_scope.add_binding(name, addr);
        self.functions
            .implement_function(addr)
            .panic()
            .add_return(self.nil_place);
    }
}
