use std::{borrow::Cow, collections::HashMap, fmt, mem};

use crate::{analysis::FunctionDefinition, ir::{DataAddress, FunctionsBuilder, PlaceAddress, Program, StaticDataBuilder, StaticFunctionAddress}, parse::{AstNode, Atom, Source, TokenKind}};

use super::{strings::decode_string, SemanticAnalysis};

pub struct IrGen<'a, 's, 't> {
    source: &'s Source,
    static_data: StaticDataBuilder,
    functions: FunctionsBuilder,
    nil_address: DataAddress,
    nil_place: PlaceAddress,
    function_addresses: HashMap<String, StaticFunctionAddress>,
    global_string_addresses: HashMap<Cow<'s, str>, DataAddress>,
    global_number_addresses: HashMap<i32, DataAddress>,
    /// Maps identifiers for global variables to a place address
    global_variables: HashMap<&'s str, PlaceAddress>,
    analysis: &'a SemanticAnalysis<'s, 't>
}

impl<'a: 't, 's, 't> IrGen<'a, 's, 't> {
    pub fn new(analysis: &'a SemanticAnalysis<'s, 't>) -> Self {
        let source = analysis.source();
        let mut static_data = StaticDataBuilder::new();
        let functions = FunctionsBuilder::new();
        let nil_address = static_data.static_nil();
        let nil_place = static_data.static_place(nil_address);
        // for now truth is just a string
        let t_address = static_data.static_string("T");
        let t_place = static_data.static_place(t_address);
        let function_addresses: HashMap<String, StaticFunctionAddress> = HashMap::<String, StaticFunctionAddress>::new();
        let mut global_variables = HashMap::new();
        global_variables.insert("nil", nil_place);
        global_variables.insert("t", t_place);
        Self {
            source,
            static_data,
            functions,
            nil_address,
            nil_place,
            function_addresses,
            global_string_addresses: HashMap::new(),
            global_number_addresses: HashMap::new(),
            global_variables,
            analysis
        }
    }

    pub fn generate(analysis: &'a SemanticAnalysis<'s, 't>) -> Result<Program, IrGenError<'s, 't>> {
        let mut generator = Self::new(analysis);
        //generator.generate_static_data()?;
        generator.generate_global_variables()?;
        generator.generate_functions()?;
        Ok(Program::new(generator.static_data.build(), generator.functions.build()))
    }

    /// Gets data addresses for static data, generating it only if needed.
    pub fn static_data_for_node(&mut self, value: &'t AstNode<'s>) -> Result<DataAddress, IrGenError<'s, 't>> {
        Ok(match value {
            AstNode::Atom(atom) => {
                match atom.token().kind() {
                    TokenKind::Comment | TokenKind::Ws | TokenKind::LeftParen | TokenKind::RightParen => unreachable!(),
                    TokenKind::StringLit => {
                        let decoded = decode_string(atom.fragment(self.source).source());
                        let decoded2 = decoded.clone();
                        *self.global_string_addresses.entry(decoded)
                            .or_insert_with(|| self.static_data.static_string(decoded2.as_ref()))
                    },
                    TokenKind::IntLit => {
                        let decoded = i32::from_str_radix(atom.fragment(self.source).source(), 10).map_err(|_| IrGenError::NumberParseError { atom, source: self.source })?;
                        *self.global_number_addresses.entry(decoded)
                            .or_insert_with(|| self.static_data.static_number(decoded))
                    },
                    TokenKind::FloatLit => todo!("floats are unimplemented"),
                    TokenKind::Ident => todo!("identifiers in static not supported")
                }
            },
            // could be a list within a quoted list, but we'll save that for later
            AstNode::List(_) => todo!("functions in global unimplemented"),
            AstNode::QuotedList(l) => {
                let mut successor = self.nil_address;
                for predecessor in l.elements().iter().rev() {
                    let predecessor = self.static_data_for_node(predecessor)?;
                    successor = self.static_data.static_list_node(predecessor, successor);
                }
                successor
            }
        })
    }

    pub fn generate_global_variables(&mut self) -> Result<(), IrGenError<'s, 't>> {
        for global in self.analysis.global_definitions() {
            let name = global.name().fragment(self.source).source();
            let value = global.value();
            let data_address = self.static_data_for_node(value)?;
            let data_place = self.static_data.static_place(data_address);
            self.global_variables.insert(name, data_place);
        }
        Ok(())
    }

    fn generate_functions(&mut self)  -> Result<(), IrGenError<'s, 't>> {
        // first give them all an index
        for function in self.analysis.function_definitions() {
            let name = function.name().fragment(self.source).source();
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
        let mut local_vars = HashMap::new();
        let func_address = self.function_addresses[definition.name().fragment(self.source).source()];
        let arg_count = definition.args().elements().len() as u32;
        self.functions.implement_function(func_address).alloc_places(arg_count);
        for (arg_idx, arg) in definition.args().elements().iter().enumerate() {
            let ident = arg.atom().unwrap().fragment(self.source).source();
            let address = PlaceAddress::new_local((arg_idx * mem::size_of::<i32>()) as i32);
            self.functions.implement_function(func_address).consume_param(address);
            local_vars.insert(ident, address);
        }
        let mut next_local_place_address = (definition.args().elements().len() * mem::size_of::<i32>()) as i32;
        let mut last_place = None;
        for code in definition.body() {
            last_place = Some(self.generate_code(code, func_address, &local_vars, &mut next_local_place_address)?);
        }
        self.functions.implement_function(func_address).dealloc_places(next_local_place_address as u32)
            .add_return(last_place.unwrap_or_else(|| self.nil_place));
        Ok(())
    }

    fn generate_root_code(&mut self) -> Result<(), IrGenError<'s, 't>> {
        let conflicting_definition = self.analysis.function_definitions()
            .iter()
            .find(|d| d.name().fragment(self.source).source() == "main");
        if let Some(conflicting_definition) = conflicting_definition {
            return Err(IrGenError::ReservedName { ident: conflicting_definition.name(), source: self.source });
        }
        let main_addr = self.functions.add_exported_function("main");
        //self.functions.implement_function(main_addr);
        let mut next_local_place_address = 0;
        let no_local_vars = HashMap::new();
        for &code in self.analysis.root_code() {
            self.generate_code(code, main_addr, &no_local_vars, &mut next_local_place_address)?;
        }
        self.functions.implement_function(main_addr)
            .dealloc_places(next_local_place_address as u32)
            .add_return(self.nil_place);
        Ok(())
    }

    fn generate_code(&mut self, code: &'t AstNode<'s>, addr: StaticFunctionAddress, local_vars: &HashMap<&'s str, PlaceAddress>, next_local_place_address: &mut i32) -> Result<PlaceAddress, IrGenError<'s, 't>> {
        Ok(match code {
            // quoted lists evaluate to a reference to static data stored in a local place
            AstNode::QuotedList(_) => {
                let data_address = self.static_data_for_node(code)?;
                let place_address = PlaceAddress::new_local(*next_local_place_address);
                *next_local_place_address += mem::size_of::<i32>() as i32;
                self.functions.implement_function(addr)
                    .alloc_places(1)
                    .load_data(data_address, place_address);
                place_address
            },
            // numbers and strings also evaluate to a reference to static data stored in a local place
            AstNode::Atom(atom) if matches!(atom.token().kind(), TokenKind::StringLit | TokenKind::IntLit) => {
                let data_address = self.static_data_for_node(code)?;
                let place_address = PlaceAddress::new_local(*next_local_place_address);
                *next_local_place_address += mem::size_of::<i32>() as i32;
                self.functions.implement_function(addr)
                    .alloc_places(1)
                    .load_data(data_address, place_address);
                place_address
            },
            // identifiers can refer to arguments or global variables and write their value to a place
            AstNode::Atom(atom) if matches!(atom.token().kind(), TokenKind::Ident) => {
                let ident = atom.fragment(self.source).source();
                let src_place = local_vars.get(ident).or_else(|| self.global_variables.get(ident))
                    .ok_or_else(|| IrGenError::NotInScope { atom, source: self.source })
                    .copied()?;
                let dst_place = PlaceAddress::new_local(*next_local_place_address);
                *next_local_place_address += mem::size_of::<i32>() as i32;
                self.functions.implement_function(addr)
                    .alloc_places(1)
                    .write_place(src_place, dst_place);
                dst_place
            },
            // () evaluates to the empty list
            AstNode::List(list) if list.elements().is_empty() => self.nil_place,
            // other atoms that are not yet supported
            AstNode::Atom(_) => unimplemented!("floats and other stuff are not yet supported"),
            // non-empty unquoted lists are function applications, evaluate the arguments first and then call them
            AstNode::List(list) => {
                let args = &list.elements()[1..];
                let mut evaluated_arg_places: Vec<PlaceAddress> = Vec::with_capacity(args.len());
                for arg in args {
                    let place = self.generate_code(arg, addr, local_vars, next_local_place_address)?;
                    evaluated_arg_places.push(place);
                }

                let arguments_place = PlaceAddress::new_local(*next_local_place_address);
                *next_local_place_address += mem::size_of::<i32>() as i32;
                let result_place = PlaceAddress::new_local(*next_local_place_address);
                *next_local_place_address += mem::size_of::<i32>() as i32;
                // start with empty argument list
                let instructions = self.functions.implement_function(addr);
                instructions.alloc_places(2)
                    .write_place(self.nil_place, arguments_place);
                // and then start pushing elements from the back to the front
                for &arg in evaluated_arg_places.iter().rev() {
                    instructions.cons(arg, arguments_place, arguments_place);
                }

                let func_ident = list.elements()[0]
                    .atom()
                    .filter(|a| a.token().kind() == TokenKind::Ident)
                    .ok_or_else(|| IrGenError::ExpectedFunctionIdentifier { found_instead: &list.elements()[0], source: self.source })?;
                let func_address = self.function_addresses.get(func_ident.fragment(self.source).source())
                    .cloned()
                    .ok_or_else(|| IrGenError::FunctionNotFound { ident: func_ident, source: self.source })?;
                instructions.call(func_address, arguments_place, result_place);
                
                result_place
            }
        })
    }
}

pub enum IrGenError<'s, 't> {
    NumberParseError {
        source: &'s Source,
        atom: &'t Atom<'s>,
    },
    NotInScope {
        source: &'s Source,
        atom: &'t Atom<'s>
    },
    ExpectedFunctionIdentifier {
        source: &'s Source,
        found_instead: &'t AstNode<'s>
    },
    FunctionNotFound {
        source: &'s Source,
        ident: &'t Atom<'s>
    },
    ReservedName {
        source: &'s Source,
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
            }
        }
    }
}
