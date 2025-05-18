use std::{collections::HashMap, fmt};

use address::LocalPlaceGenerator;
use lambdas::{contains_form_lambdas, contains_function_lambdas};
use scope::VariableScope;
use statics::{StaticDataError, StaticsGen};

use crate::{
    analysis::FunctionDefinition,
    ir::{
        FunctionAttribute, FunctionsBuilder, PlaceAddress,
        Program, StaticFunctionAddress,
    },
    parse::{Atom, TokenKind},
    source::Source,
};

use super::{
    SemanticAnalysis,
    builtin::generate_intrinsic_functions,
    form::{
        AndForm, Apply, Call, Form, Funcall, IfForm, Lambda, LetForm, OrForm,
    },
};

mod address;
mod lambdas;
mod scope;
mod statics;

pub struct IrGen<'a, 's, 't> {
    analysis: &'a SemanticAnalysis<'s, 't>,
    static_data: StaticsGen<'s>,
    functions: FunctionsBuilder,
    function_addresses: HashMap<String, StaticFunctionAddress>,
    variable_scope: VariableScope<'s>,
}

impl<'a: 't, 's, 't> IrGen<'a, 's, 't> {
    pub fn new(analysis: &'a SemanticAnalysis<'s, 't>) -> Self {
        let static_data = StaticsGen::new();
        let functions = FunctionsBuilder::new();
        let function_addresses: HashMap<String, StaticFunctionAddress> =
            HashMap::<String, StaticFunctionAddress>::new();
        let mut global_variables = VariableScope::new();
        global_variables.add_binding("nil", static_data.nil_place());
        global_variables.add_binding("t", static_data.t_place());
        Self {
            static_data,
            functions,
            function_addresses,
            variable_scope: global_variables,
            analysis,
        }
    }

    pub fn generate(
        analysis: &'a SemanticAnalysis<'s, 't>,
    ) -> Result<Program, IrGenError<'s, 't>> {
        let mut generator = Self::new(analysis);
        generate_intrinsic_functions(
            &mut generator.functions,
            &mut generator.function_addresses,
            generator.static_data.nil_place(),
        );
        generator.generate_source_global_variables()?;
        generator.generate_source_functions()?;
        Ok(Program::new(
            generator.static_data.build(),
            generator.functions.build(),
        ))
    }

    pub fn generate_source_global_variables(
        &mut self,
    ) -> Result<(), IrGenError<'s, 't>> {
        for global in self.analysis.global_definitions() {
            let name = global.name().fragment(global.source()).source();
            let value = global.value().constant().ok_or_else(|| {
                IrGenError::GlobalMustHaveConstantInitializer {
                    source: global.source(),
                    ident: global.name(),
                }
            })?;
            let data_address =
                self.static_data.for_node(global.source(), value.node())?;
            let data_place = self.static_data.static_place(data_address);
            self.variable_scope.add_binding(name, data_place);
        }
        Ok(())
    }

    fn generate_source_functions(&mut self) -> Result<(), IrGenError<'s, 't>> {
        // first give them all an index
        for function in self.analysis.function_definitions() {
            let name = function.name().fragment(function.source()).source();
            let address = self.functions.add_exported_function(name);
            self.function_addresses.insert(name.into(), address);
        }
        // then generate the actual code for named functions
        for function in self.analysis.function_definitions() {
            self.generate_function(function)?;
        }
        // and top-level code
        self.generate_root_code()?;
        Ok(())
    }

    fn generate_function(
        &mut self,
        definition: &'t FunctionDefinition<'s, 't>,
    ) -> Result<(), IrGenError<'s, 't>> {
        self.variable_scope.enter_scope();
        let func_address = self.function_addresses
            [definition.name().fragment(definition.source()).source()];
        let mut locals = LocalPlaceGenerator::new();
        if contains_function_lambdas(definition) {
            self.functions.add_attribute(
                func_address,
                FunctionAttribute::CreatesPersistentPlaces,
            );
        }
        self.functions.implement_function(func_address);
        for arg in definition.positional_args().iter() {
            let ident = arg.fragment(definition.source()).source();
            let address = locals.next();
            self.functions
                .implement_function(func_address)
                .consume_param(address);
            self.variable_scope.add_binding(ident, address);
        }
        if let Some(rest_arg) = definition.rest_arg() {
            let rest_ident = rest_arg.fragment(definition.source()).source();
            let rest_address = locals.next();
            self.functions
                .implement_function(func_address)
                .consume_rest(rest_address);
            self.variable_scope.add_binding(rest_ident, rest_address);
        }
        let mut last_place = None;
        for code in definition.body() {
            last_place = Some(self.generate_code(
                definition.source(),
                code,
                func_address,
                &mut locals,
            )?);
        }
        self.functions
            .implement_function(func_address)
            .add_return(last_place.unwrap_or_else(|| self.static_data.nil_place()));
        self.variable_scope.exit_scope();
        Ok(())
    }

    fn generate_root_code(&mut self) -> Result<(), IrGenError<'s, 't>> {
        let conflicting_definition = self
            .analysis
            .function_definitions()
            .iter()
            .find(|d| d.name().fragment(d.source()).source() == "main");
        if let Some(conflicting_definition) = conflicting_definition {
            return Err(IrGenError::ReservedName {
                ident: conflicting_definition.name(),
                source: conflicting_definition.source(),
            });
        }
        let main_addr = self.functions.add_exported_function("main");
        let contains_toplevel_lambdas = self
            .analysis
            .root_code()
            .iter()
            .flat_map(|r| r.code())
            .any(contains_form_lambdas);
        let mut locals = LocalPlaceGenerator::new();
        if contains_toplevel_lambdas {
            self.functions.add_attribute(
                main_addr,
                FunctionAttribute::CreatesPersistentPlaces,
            );
        }

        let mut last_place = None;
        for code in self.analysis.root_code() {
            for each_code in code.code() {
                last_place = Some(self.generate_code(
                    code.source(),
                    each_code,
                    main_addr,
                    &mut locals,
                )?);
            }
        }
        self.functions
            .implement_function(main_addr)
            .add_return(last_place.unwrap_or(self.static_data.nil_place()));
        Ok(())
    }

    fn generate_code(
        &mut self,
        source: Source<'s>,
        code: &Form<'s, 't>,
        addr: StaticFunctionAddress,
        locals: &mut LocalPlaceGenerator,
    ) -> Result<PlaceAddress, IrGenError<'s, 't>> {
        Ok(match code {
            // names refer directly to the place they are bound to, so they can be written
            Form::Name(name) => self
                .variable_scope
                .resolve(name.as_str())
                .map_err(|_| IrGenError::NotInScope {
                    source,
                    atom: name.ident(),
                })?,
            Form::FunctionName(name) => {
                // functions don't really have a scope here, but should probably consider stuff like flet in the future
                let value = name.as_str();
                let static_address = *self
                    .function_addresses
                    .get(value)
                    .ok_or_else(|| IrGenError::FunctionNotFound {
                        source,
                        ident: name.ident(),
                    })?;
                let static_func_address = self.static_data.static_function(static_address);
                let place_address = locals.next();
                self.functions
                    .implement_function(addr)
                    .load_data(static_func_address, place_address);
                place_address
            }
            // numbers, strings and quoted stuff evaluate to a reference to static data stored in a local place
            Form::Constant(constant) => {
                let data_address =
                    self.static_data.for_node(source, constant.node())?;
                let place_address = locals.next();
                self.functions
                    .implement_function(addr)
                    .load_data(data_address, place_address);
                place_address
            }
            Form::IfForm(form) => {
                self.generate_code_for_if_form(source, form, addr, locals)?
            }
            Form::AndForm(form) => {
                self.generate_code_for_and_form(source, form, addr, locals)?
            }
            Form::OrForm(form) => {
                self.generate_code_for_or_form(source, form, addr, locals)?
            }
            Form::LetForm(let_form) => self
                .generate_code_for_let_form(source, let_form, addr, locals)?,
            Form::Call(call) => self.generate_code_for_function_application(
                source, call, addr, locals,
            )?,
            Form::Apply(form) => {
                self.generate_code_for_apply(source, form, addr, locals)?
            }
            Form::Funcall(form) => {
                self.generate_code_for_funcall(source, form, addr, locals)?
            }
            Form::Lambda(lambda) => {
                self.generate_code_for_lambda(source, lambda, addr, locals)?
            }
        })
    }

    fn generate_code_for_if_form(
        &mut self,
        source: Source<'s>,
        form: &IfForm<'s, 't>,
        addr: StaticFunctionAddress,
        locals: &mut LocalPlaceGenerator,
    ) -> Result<PlaceAddress, IrGenError<'s, 't>> {
        // generate something like: result = test(); a:{ b:{ if result != nil { break b; } … else code …  break a } … then code … }

        // evaluate the test
        let test_result_place =
            self.generate_code(source, form.test_form(), addr, locals)?;
        let result_place = locals.next();

        // then do some checking and the else branch
        self.functions
            .implement_function(addr)
            .enter_block()
            .enter_block()
            .break_if_not_nil(1, test_result_place);
        let else_result_place = match form.else_form() {
            Some(else_form) => {
                self.generate_code(source, else_form, addr, locals)?
            }
            None => self.static_data.nil_place(),
        };
        self.functions
            .implement_function(addr)
            .write_place(else_result_place, result_place)
            .add_break(2)
            .exit_block();

        // and the then branch after the inner block
        let then_result_place =
            self.generate_code(source, form.then_form(), addr, locals)?;

        self.functions
            .implement_function(addr)
            .write_place(then_result_place, result_place)
            .exit_block();
        Ok(result_place)
    }

    fn generate_code_for_and_form(
        &mut self,
        source: Source<'s>,
        form: &AndForm<'s, 't>,
        addr: StaticFunctionAddress,
        locals: &mut LocalPlaceGenerator,
    ) -> Result<PlaceAddress, IrGenError<'s, 't>> {
        match form.forms().len() {
            0 => Ok(self.static_data.t_place()),
            1 => self.generate_code(source, &form.forms()[0], addr, locals),
            // short-circuiting makes sense for two or more forms
            _ => {
                let result_place = locals.next();

                self.functions
                    .implement_function(addr)
                    .load_data(self.static_data.t_data(), result_place)
                    .enter_block();

                for form in form.forms() {
                    let form_result =
                        self.generate_code(source, form, addr, locals)?;
                    self.functions
                        .implement_function(addr)
                        .write_place(form_result, result_place)
                        .break_if_nil(1, result_place);
                }

                self.functions.implement_function(addr).exit_block();

                Ok(result_place)
            }
        }
    }

    fn generate_code_for_or_form(
        &mut self,
        source: Source<'s>,
        form: &OrForm<'s, 't>,
        addr: StaticFunctionAddress,
        locals: &mut LocalPlaceGenerator,
    ) -> Result<PlaceAddress, IrGenError<'s, 't>> {
        match form.forms().len() {
            0 => Ok(self.static_data.nil_place()),
            1 => self.generate_code(source, &form.forms()[0], addr, locals),
            // short-circuiting makes sense for two or more forms
            _ => {
                let result_place = locals.next();

                self.functions
                    .implement_function(addr)
                    .load_data(self.static_data.nil_data(), result_place)
                    .enter_block();

                for form in form.forms() {
                    let form_result =
                        self.generate_code(source, form, addr, locals)?;
                    self.functions
                        .implement_function(addr)
                        .write_place(form_result, result_place)
                        .break_if_not_nil(1, result_place);
                }

                self.functions.implement_function(addr).exit_block();

                Ok(result_place)
            }
        }
    }

    fn generate_code_for_let_form(
        &mut self,
        source: Source<'s>,
        form: &LetForm<'s, 't>,
        addr: StaticFunctionAddress,
        locals: &mut LocalPlaceGenerator,
    ) -> Result<PlaceAddress, IrGenError<'s, 't>> {
        let mut places_to_add_simultaneously =
            Vec::with_capacity(form.bindings().len());
        for binding in form.bindings() {
            let place =
                self.generate_code(source, binding.value(), addr, locals)?;
            places_to_add_simultaneously
                .push((binding.name().fragment(source).source(), place));
        }
        self.variable_scope.enter_scope();
        for (name, address) in places_to_add_simultaneously {
            self.variable_scope.add_binding(name, address);
        }

        let mut last_result = None;
        for body in form.body() {
            last_result =
                Some(self.generate_code(source, body, addr, locals)?);
        }
        let last_result = last_result.unwrap_or(self.static_data.nil_place());
        self.variable_scope.exit_scope();
        Ok(last_result)
    }

    fn generate_code_for_function_application(
        &mut self,
        source: Source<'s>,
        call: &Call<'s, 't>,
        addr: StaticFunctionAddress,
        locals: &mut LocalPlaceGenerator,
    ) -> Result<PlaceAddress, IrGenError<'s, 't>> {
        let args = call.args();
        let mut evaluated_arg_places: Vec<PlaceAddress> =
            Vec::with_capacity(args.len());
        for arg in args {
            let place = self.generate_code(source, arg, addr, locals)?;
            evaluated_arg_places.push(place);
        }

        let arguments_place = locals.next();
        let result_place = locals.next();
        // start with empty argument list
        let instructions = self.functions.implement_function(addr);
        instructions.load_data(self.static_data.nil_data(), arguments_place);
        // and then start pushing elements from the back to the front
        for &arg in evaluated_arg_places.iter().rev() {
            instructions.cons(arg, arguments_place, arguments_place);
        }

        let func_ident = call.function();
        let func_address = self
            .function_addresses
            .get(func_ident.fragment(source).source())
            .cloned()
            .ok_or_else(|| IrGenError::FunctionNotFound {
                ident: func_ident,
                source,
            })?;
        instructions.call(func_address, arguments_place, result_place);

        Ok(result_place)
    }

    fn generate_code_for_apply(
        &mut self,
        source: Source<'s>,
        apply: &Apply<'s, 't>,
        addr: StaticFunctionAddress,
        locals: &mut LocalPlaceGenerator,
    ) -> Result<PlaceAddress, IrGenError<'s, 't>> {
        let arg_list =
            self.generate_code(source, apply.args(), addr, locals)?;
        let result_place = locals.next();
        let function = apply.function();
        if let Some(func_name) = function.function_name().map(|c| c.ident()) {
            // when the target is a static function identify, we can do a fast direct call
            assert!(matches!(func_name.token().kind(), TokenKind::FuncIdent));
            let func_name_str =
                func_name.fragment(source).source()[2..].trim();
            let func_address = self
                .function_addresses
                .get(func_name_str)
                .cloned()
                .ok_or_else(|| IrGenError::FunctionNotFound {
                    ident: func_name,
                    source,
                })?;
            self.functions.implement_function(addr).call(
                func_address,
                arg_list,
                result_place,
            );
        } else {
            // function calculated at runtime, need an indirect call
            let function_place =
                self.generate_code(source, function, addr, locals)?;
            self.functions.implement_function(addr).call_indirect(
                function_place,
                arg_list,
                result_place,
            );
        }

        Ok(result_place)
    }

    fn generate_code_for_funcall(
        &mut self,
        source: Source<'s>,
        funcall: &Funcall<'s, 't>,
        addr: StaticFunctionAddress,
        locals: &mut LocalPlaceGenerator,
    ) -> Result<PlaceAddress, IrGenError<'s, 't>> {
        let args = funcall.args();
        let mut evaluated_arg_places: Vec<PlaceAddress> =
            Vec::with_capacity(args.len());
        for arg in args {
            let place = self.generate_code(source, arg, addr, locals)?;
            evaluated_arg_places.push(place);
        }

        let arguments_place = locals.next();
        let result_place = locals.next();
        // start with empty argument list
        let instructions = self.functions.implement_function(addr);
        instructions.load_data(self.static_data.nil_data(), arguments_place);
        // and then start pushing elements from the back to the front
        for &arg in evaluated_arg_places.iter().rev() {
            instructions.cons(arg, arguments_place, arguments_place);
        }

        let function = funcall.function();
        if let Some(func_name) = function.function_name().map(|c| c.ident()) {
            // when the target is a static function identifier, we can do a fast direct call
            assert!(matches!(func_name.token().kind(), TokenKind::FuncIdent));
            let func_name_str =
                func_name.fragment(source).source()[2..].trim();
            let func_address = self
                .function_addresses
                .get(func_name_str)
                .cloned()
                .ok_or_else(|| IrGenError::FunctionNotFound {
                    ident: func_name,
                    source,
                })?;
            self.functions.implement_function(addr).call(
                func_address,
                arguments_place,
                result_place,
            );
        } else {
            // function calculated at runtime, need an indirect call
            // bug: I think the function should be evaluated first, same for apply
            let function_place =
                self.generate_code(source, function, addr, locals)?;
            self.functions.implement_function(addr).call_indirect(
                function_place,
                arguments_place,
                result_place,
            );
        }

        Ok(result_place)
    }

    fn generate_code_for_lambda(
        &mut self,
        source: Source<'s>,
        lambda: &Lambda<'s, 't>,
        parent_func_addr: StaticFunctionAddress,
        locals: &mut LocalPlaceGenerator, // we share the locals but not the function address
    ) -> Result<PlaceAddress, IrGenError<'s, 't>> {
        let parent_func_lambda_place = locals.next();
        let lambda_fun_name = format!(
            "--lambda-{}-{}",
            parent_func_addr.to_i32(),
            parent_func_lambda_place.offset()
        );

        self.variable_scope.enter_scope();
        let lambda_func_addr =
            self.functions.add_private_function(&lambda_fun_name);
        let lambda_func_table_idx =
            self.static_data.function_table_entry(lambda_func_addr);
        self.functions.add_attribute(
            lambda_func_addr,
            FunctionAttribute::AcceptsPersistentPlaces,
        );
        // the name of the lambda function is not added to any scopes (lambdas cannot be named)

        for arg in lambda.positional_args().iter() {
            let ident = arg.fragment(source).source();
            let address = locals.next();
            self.functions
                .implement_function(lambda_func_addr)
                .consume_param(address);
            self.variable_scope.add_binding(ident, address);
        }
        let mut last_place = None;
        for code in lambda.body() {
            last_place = Some(self.generate_code(
                source,
                code,
                lambda_func_addr,
                locals,
            )?);
        }
        self.functions
            .implement_function(lambda_func_addr)
            .add_return(last_place.unwrap_or_else(|| self.static_data.nil_place()));

        self.variable_scope.exit_scope();

        self.functions
            .implement_function(parent_func_addr)
            .create_function(lambda_func_table_idx, parent_func_lambda_place);

        Ok(parent_func_lambda_place)
    }
}

pub enum IrGenError<'s, 't> {
    NotInScope {
        source: Source<'s>,
        atom: &'t Atom<'s>,
    },
    FunctionNotFound {
        source: Source<'s>,
        ident: &'t Atom<'s>,
    },
    ReservedName {
        source: Source<'s>,
        ident: &'t Atom<'s>,
    },
    GlobalMustHaveConstantInitializer {
        source: Source<'s>,
        ident: &'t Atom<'s>,
    },
    StaticData {
        error: StaticDataError<'s, 't>
    },
}

impl<'s, 't> From<StaticDataError<'s, 't>> for IrGenError<'s, 't> {
    fn from(value: StaticDataError<'s, 't>) -> Self {
        Self::StaticData { error: value }
    }
}

impl<'s, 't> fmt::Display for IrGenError<'s, 't> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &IrGenError::NotInScope { atom, source } => {
                writeln!(
                    f,
                    "variable `{}` could not be resolved:",
                    atom.source_range().of(source).source()
                )?;
                writeln!(f, "{}", atom.fragment(source).source_context())
            }
            &IrGenError::FunctionNotFound { source, ident } => {
                writeln!(
                    f,
                    "function name `{}` could not be resolved:",
                    ident.source_range().of(source).source()
                )?;
                writeln!(f, "{}", ident.fragment(source).source_context())
            }
            &IrGenError::ReservedName { source, ident } => {
                writeln!(
                    f,
                    "{} is a reserved name:",
                    ident.fragment(source).source()
                )?;
                writeln!(f, "{}", ident.fragment(source).source_context())
            }
            &IrGenError::GlobalMustHaveConstantInitializer {
                source,
                ident,
            } => {
                writeln!(
                    f,
                    "global {} does not have a constant initial value, which is unsupported:",
                    ident.fragment(source).source()
                )?;
                writeln!(f, "{}", ident.fragment(source).source_context())
            }
            IrGenError::StaticData { error } => write!(f, "{}", error)
        }
    }
}
