use std::{borrow::Cow, collections::HashMap, fmt, mem};

use scope::VariableScope;

use crate::{analysis::FunctionDefinition, ir::{DataAddress, FunctionsBuilder, PlaceAddress, Program, StaticDataBuilder, StaticFunctionAddress}, parse::{AstNode, Atom, List, TokenKind}, source::Source};

use super::{builtin::generate_builtin_functions, form::{self, AndForm, Call, Form, IfForm, LetForm, OrForm}, strings::decode_string, SemanticAnalysis};

mod scope;

pub struct IrGen<'a, 's, 't> {
    // TODO make this work without a single source
    static_data: StaticDataBuilder,
    functions: FunctionsBuilder,
    nil_address: DataAddress,
    nil_place: PlaceAddress,
    t_address: DataAddress,
    t_place: PlaceAddress,
    function_addresses: HashMap<String, StaticFunctionAddress>,
    global_string_addresses: HashMap<Cow<'s, str>, DataAddress>,
    /// Identifiers for static data, e.g. 'string for concatenate
    global_identifier_addresses: HashMap<&'s str, DataAddress>,
    global_number_addresses: HashMap<i32, DataAddress>,
    variable_scope: VariableScope<'s>,
    analysis: &'a SemanticAnalysis<'s, 't>
}

impl<'a: 't, 's, 't> IrGen<'a, 's, 't> {
    pub fn new(analysis: &'a SemanticAnalysis<'s, 't>) -> Self {
        let mut static_data = StaticDataBuilder::new();
        let functions = FunctionsBuilder::new();
        let nil_address = static_data.static_nil();
        let nil_place = static_data.static_place(nil_address);
        // for now truth is just a string
        let t_address = static_data.static_identifier("T");
        let t_place = static_data.static_place(t_address);
        let function_addresses: HashMap<String, StaticFunctionAddress> = HashMap::<String, StaticFunctionAddress>::new();
        let mut global_variables = VariableScope::new();
        global_variables.add_binding("nil", nil_place);
        global_variables.add_binding("t", t_place);
        Self {
            static_data,
            functions,
            nil_address,
            nil_place,
            t_address,
            t_place,
            function_addresses,
            global_string_addresses: HashMap::new(),
            global_identifier_addresses: HashMap::new(),
            global_number_addresses: HashMap::new(),
            variable_scope: global_variables,
            analysis
        }
    }

    pub fn generate(analysis: &'a SemanticAnalysis<'s, 't>) -> Result<Program, IrGenError<'s, 't>> {
        let mut generator = Self::new(analysis);
        generator.generate_builtin_functions()?;
        generator.generate_source_global_variables()?;
        generator.generate_source_functions()?;
        Ok(Program::new(generator.static_data.build(), generator.functions.build()))
    }

    /// Gets data addresses for static data, reusing existing data if possible.
    pub fn static_data_for_node(&mut self, source: Source<'s>, value: &'t AstNode<'s>) -> Result<DataAddress, IrGenError<'s, 't>> {
        Ok(match value {
            AstNode::Atom(atom) => {
                match atom.token().kind() {
                    TokenKind::Comment | TokenKind::Ws | TokenKind::LeftParen | TokenKind::RightParen | TokenKind::Quote => unreachable!(),
                    TokenKind::StringLit => {
                        let decoded = decode_string(atom.fragment(source).source());
                        let decoded2 = decoded.clone();
                        *self.global_string_addresses.entry(decoded)
                            .or_insert_with(|| self.static_data.static_string(decoded2.as_ref()))
                    },
                    TokenKind::IntLit => {
                        let decoded = i32::from_str_radix(atom.fragment(source).source(), 10).map_err(|_| IrGenError::NumberParseError { atom, source: source })?;
                        self.static_number(decoded)
                    },
                    TokenKind::FloatLit => todo!("floats are unimplemented"),
                    TokenKind::Ident => {
                        let value = atom.fragment(source).source();
                        *self.global_identifier_addresses.entry(value)
                            .or_insert_with(|| self.static_data.static_identifier(value))
                    }
                }
            },
            AstNode::List(l) => {
                let mut successor = self.nil_address;
                for predecessor in l.elements().iter().rev() {
                    let predecessor = self.static_data_for_node(source, predecessor)?;
                    successor = self.static_data.static_list_node(predecessor, successor);
                }
                successor
            },
            AstNode::Quoted(q) => self.static_data_for_node(source, q.quoted())?,
        })
    }

    fn static_number(&mut self, decoded: i32) -> DataAddress {
        *self.global_number_addresses.entry(decoded)
            .or_insert_with(|| self.static_data.static_number(decoded))
    }
    
    pub fn generate_source_global_variables(&mut self) -> Result<(), IrGenError<'s, 't>> {
        for global in self.analysis.global_definitions() {
            let name = global.name().fragment(global.source()).source();
            let value = global.value().constant()
                .ok_or_else(|| IrGenError::GlobalMustHaveConstantInitializer { source: global.source(), ident: global.name() })?;
            let data_address = self.static_data_for_node(global.source(), value.node())?;
            let data_place = self.static_data.static_place(data_address);
            self.variable_scope.add_binding(name, data_place);
        }
        Ok(())
    }

    fn generate_builtin_functions(&mut self) -> Result<(), IrGenError<'s, 't>> {
        generate_builtin_functions(&mut self.static_data, &mut self.functions, &mut self.function_addresses, self.nil_address, self.nil_place);
        Ok(())
    }

    fn generate_source_functions(&mut self)  -> Result<(), IrGenError<'s, 't>> {
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

    fn generate_function(&mut self, definition: &'t FunctionDefinition<'s, 't>) -> Result<(), IrGenError<'s, 't>> {
        let mut next_local_place_address = 0;
        self.variable_scope.enter_scope();
        let func_address = self.function_addresses[definition.name().fragment(definition.source()).source()];
        self.functions.implement_function(func_address);
        for (arg_idx, arg) in definition.positional_args().iter().enumerate() {
            let ident = arg.fragment(definition.source()).source();
            let address = PlaceAddress::new_local(next_local_place_address);
            next_local_place_address += mem::size_of::<i32>() as i32;
            self.functions.implement_function(func_address).consume_param(address);
            self.variable_scope.add_binding(ident, address);
        }
        if let Some(rest_arg) = definition.rest_arg() {
            let rest_ident = rest_arg.fragment(definition.source()).source();
            let rest_address = PlaceAddress::new_local(next_local_place_address);
            next_local_place_address += mem::size_of::<i32>() as i32;
            self.functions.implement_function(func_address).consume_rest(rest_address);
            self.variable_scope.add_binding(rest_ident, rest_address);
        }
        let mut last_place = None;
        for code in definition.body() {
            last_place = Some(self.generate_code(definition.source(), code, func_address, &mut next_local_place_address)?);
        }
        self.functions.implement_function(func_address)
            .add_return(last_place.unwrap_or_else(|| self.nil_place));
        self.variable_scope.exit_scope();
        Ok(())
    }

    fn generate_root_code(&mut self) -> Result<(), IrGenError<'s, 't>> {
        let conflicting_definition = self.analysis.function_definitions()
            .iter()
            .find(|d| d.name().fragment(d.source()).source() == "main");
        if let Some(conflicting_definition) = conflicting_definition {
            return Err(IrGenError::ReservedName { ident: conflicting_definition.name(), source: conflicting_definition.source() });
        }
        let main_addr = self.functions.add_exported_function("main");
        //self.functions.implement_function(main_addr);
        let mut next_local_place_address = 0;
        let mut last_place = None;
        for code in self.analysis.root_code() {
            for each_code in code.code() {
                last_place = Some(self.generate_code(code.source(), each_code, main_addr, &mut next_local_place_address)?);
            }
        }
        self.functions.implement_function(main_addr)
            .add_return(last_place.unwrap_or(self.nil_place));
        Ok(())
    }

    fn generate_code(&mut self, source: Source<'s>, code: &Form<'s, 't>, addr: StaticFunctionAddress, next_local_place_address: &mut i32) -> Result<PlaceAddress, IrGenError<'s, 't>> {
        Ok(match code {
            // names refer directly to the place they are bound to, so they can be written
            Form::Name(name) => self.variable_scope.resolve(name.as_str()).map_err(|_| IrGenError::NotInScope { source, atom: name.ident() })?,
            // numbers, strings and quoted stuff evaluate to a reference to static data stored in a local place
            Form::Constant(constant) => {
                let data_address = self.static_data_for_node(source, constant.node())?;
                let place_address = PlaceAddress::new_local(*next_local_place_address);
                *next_local_place_address += mem::size_of::<i32>() as i32;
                self.functions.implement_function(addr)
                    .load_data(data_address, place_address);
                place_address
            },
            Form::IfForm(form) => self.generate_code_for_if_form(source, form, addr, next_local_place_address)?,
            Form::AndForm(form) => self.generate_code_for_and_form(source, form, addr, next_local_place_address)?,
            Form::OrForm(form) => self.generate_code_for_or_form(source, form, addr, next_local_place_address)?,
            Form::LetForm(let_form) => self.generate_code_for_let_form(source, let_form, addr, next_local_place_address)?,
            Form::Call(call) => self.generate_code_for_function_application(source, call, addr, next_local_place_address)?,
        })
    }

    fn generate_code_for_if_form(&mut self, source: Source<'s>, form: &IfForm<'s, 't>, addr: StaticFunctionAddress, next_local_place_address: &mut i32)-> Result<PlaceAddress, IrGenError<'s, 't>> {
        // generate something like: result = test(); a:{ b:{ if result != nil { break b; } … else code …  break a } … then code … }

        // evaluate the test
        let test_result_place = self.generate_code(source, form.test_form(), addr, next_local_place_address)?;
        let result_place = PlaceAddress::new_local(*next_local_place_address);
        *next_local_place_address += mem::size_of::<i32>() as i32;

        // then do some checking and the else branch
        self.functions.implement_function(addr)
            .enter_block()
                .enter_block()
                    .break_if_not_nil(1, test_result_place);
        let else_result_place = match form.else_form() {
            Some(else_form) => self.generate_code(source, else_form, addr, next_local_place_address)?,
            None => self.nil_place
        };
        self.functions.implement_function(addr).write_place(else_result_place, result_place)
                    .add_break(2)
                .exit_block();
        
        // and the then branch after the inner block
        let then_result_place = self.generate_code(source, form.then_form(), addr, next_local_place_address)?;

        self.functions.implement_function(addr).write_place(then_result_place, result_place).exit_block();
        Ok(result_place)
    }

    fn generate_code_for_and_form(&mut self, source: Source<'s>, form: &AndForm<'s, 't>, addr: StaticFunctionAddress, next_local_place_address: &mut i32)-> Result<PlaceAddress, IrGenError<'s, 't>> {
        match form.forms().len() {
            0 => Ok(self.t_place),
            1 => self.generate_code(source, &form.forms()[0], addr, next_local_place_address),
            // short-circuiting makes sense for two or more forms
            _ => {
                let result_place = PlaceAddress::new_local(*next_local_place_address);
                *next_local_place_address += mem::size_of::<i32>() as i32;
                
                self.functions.implement_function(addr)
                    .load_data(self.t_address, result_place)
                    .enter_block();

                for form in form.forms() {
                    let form_result = self.generate_code(source, form, addr, next_local_place_address)?;
                    self.functions.implement_function(addr)
                        .write_place(form_result, result_place)
                        .break_if_nil(1, result_place);
                }

                self.functions.implement_function(addr).exit_block();

                Ok(result_place)
            }
        }
    }

    fn generate_code_for_or_form(&mut self, source: Source<'s>, form: &OrForm<'s, 't>, addr: StaticFunctionAddress, next_local_place_address: &mut i32)-> Result<PlaceAddress, IrGenError<'s, 't>> {
        match form.forms().len() {
            0 => Ok(self.nil_place),
            1 => self.generate_code(source, &form.forms()[0], addr, next_local_place_address),
            // short-circuiting makes sense for two or more forms
            _ => {
                let result_place = PlaceAddress::new_local(*next_local_place_address);
                *next_local_place_address += mem::size_of::<i32>() as i32;
                
                self.functions.implement_function(addr)
                    .load_data(self.nil_address, result_place)
                    .enter_block();

                for form in form.forms() {
                    let form_result = self.generate_code(source, form, addr, next_local_place_address)?;
                    self.functions.implement_function(addr)
                        .write_place(form_result, result_place)
                        .break_if_not_nil(1, result_place);
                }

                self.functions.implement_function(addr).exit_block();

                Ok(result_place)
            }
        }
    }

    fn generate_code_for_let_form(&mut self, source: Source<'s>, form: &LetForm<'s, 't>, addr: StaticFunctionAddress, next_local_place_address: &mut i32)-> Result<PlaceAddress, IrGenError<'s, 't>> {
        let mut places_to_add_simultaneously = Vec::with_capacity(form.bindings().len());
        for binding in form.bindings() {
            let place = self.generate_code(source, binding.value(), addr, next_local_place_address)?;
            places_to_add_simultaneously.push((binding.name().fragment(source).source(), place));
        }
        self.variable_scope.enter_scope();
        for (name, address) in places_to_add_simultaneously {
            self.variable_scope.add_binding(name, address);
        }

        let mut last_result = None;
        for body in form.body() {
            last_result = Some(self.generate_code(source, body, addr, next_local_place_address)?);
        }
        let last_result = last_result.unwrap_or(self.nil_place);
        self.variable_scope.exit_scope();
        Ok(last_result)
    }

    fn generate_code_for_function_application(&mut self, source: Source<'s>, call: &Call<'s, 't>, addr: StaticFunctionAddress, next_local_place_address: &mut i32) -> Result<PlaceAddress, IrGenError<'s, 't>> {
        let args = call.args();
        let mut evaluated_arg_places: Vec<PlaceAddress> = Vec::with_capacity(args.len());
        for arg in args {
            let place = self.generate_code(source, arg, addr, next_local_place_address)?;
            evaluated_arg_places.push(place);
        }

        let arguments_place = PlaceAddress::new_local(*next_local_place_address);
        *next_local_place_address += mem::size_of::<i32>() as i32;
        let result_place = PlaceAddress::new_local(*next_local_place_address);
        *next_local_place_address += mem::size_of::<i32>() as i32;
        // start with empty argument list
        let instructions = self.functions.implement_function(addr);
        instructions.load_data(self.nil_address, arguments_place);
        // and then start pushing elements from the back to the front
        for &arg in evaluated_arg_places.iter().rev() {
            instructions.cons(arg, arguments_place, arguments_place);
        }

        let func_ident = call.function();
        let func_address = self.function_addresses.get(func_ident.fragment(source).source())
            .cloned()
            .ok_or_else(|| IrGenError::FunctionNotFound { ident: func_ident, source })?;
        instructions.call(func_address, arguments_place, result_place);
        
        Ok(result_place)
    }
}

pub enum IrGenError<'s, 't> {
    NumberParseError {
        source: Source<'s>,
        atom: &'t Atom<'s>,
    },
    NotInScope {
        source: Source<'s>,
        atom: &'t Atom<'s>
    },
    ExpectedFunctionIdentifier {
        source: Source<'s>,
        found_instead: &'t AstNode<'s>
    },
    FunctionNotFound {
        source: Source<'s>,
        ident: &'t Atom<'s>
    },
    ReservedName {
        source: Source<'s>,
        ident: &'t Atom<'s>
    },
    GlobalMustHaveConstantInitializer {
        source: Source<'s>,
        ident: &'t Atom<'s>
    }
}

impl<'s, 't> fmt::Display for IrGenError<'s, 't> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &IrGenError::NumberParseError { atom, source } => {
                writeln!(
                    f,
                    "number cannot be parsed as a 32-bit signed integer `{}`:",
                    atom.source_range().of(source).source()
                )?;
                writeln!(f, "{}", atom.fragment(source).source_context())
            },
            &IrGenError::NotInScope { atom, source } => {
                writeln!(
                    f,
                    "variable `{}` could not be resolved:",
                    atom.source_range().of(source).source()
                )?;
                writeln!(f, "{}", atom.fragment(source).source_context())
            },
            &IrGenError::FunctionNotFound { source, ident } => {
                writeln!(
                    f,
                    "function name `{}` could not be resolved:",
                    ident.source_range().of(source).source()
                )?;
                writeln!(f, "{}", ident.fragment(source).source_context())
            },
            &IrGenError::ExpectedFunctionIdentifier { source, found_instead } => {
                writeln!(f, "expected a function identifier here:")?;
                writeln!(f, "{}", found_instead.fragment(source).source_context())
            },
            &IrGenError::ReservedName { source, ident } => {
                writeln!(f, "{} is a reserved name:", ident.fragment(source).source())?;
                writeln!(f, "{}", ident.fragment(source).source_context())
            },
            &IrGenError::GlobalMustHaveConstantInitializer { source, ident } => {
                writeln!(f, "global {} does not have a constant initial value, which is unsupported:", ident.fragment(source).source())?;
                writeln!(f, "{}", ident.fragment(source).source_context())
            }
        }
    }
}
