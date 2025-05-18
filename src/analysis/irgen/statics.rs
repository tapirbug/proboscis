use std::{borrow::Cow, collections::HashMap, fmt};

use crate::{analysis::strings::decode_string, diagnostic::Diagnostic, ir::{DataAddress, FunctionTableIndex, PlaceAddress, StaticData, StaticDataBuilder, StaticFunctionAddress}, parse::{AstNode, Atom, TokenKind}, source::Source};

pub struct StaticsGen<'s> {
    static_data: StaticDataBuilder,
    nil_place: PlaceAddress,
    t_place: PlaceAddress,
    func_table_indexes: HashMap<StaticFunctionAddress, FunctionTableIndex>,
    global_string_addresses: HashMap<Cow<'s, str>, DataAddress>,
    global_identifier_addresses: HashMap<&'s str, DataAddress>,
    global_number_addresses: HashMap<i32, DataAddress>,
    global_function_addresses: HashMap<StaticFunctionAddress, DataAddress>,
}

impl<'s> StaticsGen<'s> {
    pub fn new() -> Self {
        let mut static_data = StaticDataBuilder::new();
        let nil_place = static_data.static_place(static_data.nil_data());
        let t_place = static_data.static_place(static_data.t_data());
        Self {
            static_data,
            nil_place,
            t_place,
            func_table_indexes: HashMap::new(),
            global_string_addresses: HashMap::new(),
            global_identifier_addresses: HashMap::new(),
            global_number_addresses: HashMap::new(),
            global_function_addresses: HashMap::new(),
        }
    }

    pub fn nil_place(&self) -> PlaceAddress {
        self.nil_place
    }

    pub fn t_place(&self) -> PlaceAddress {
        self.t_place
    }

    pub fn nil_data(&self) -> DataAddress {
        self.static_data.nil_data()
    }

    pub fn t_data(&self) -> DataAddress {
        self.static_data.t_data()
    }

    /// Gets a reference to static data that encodes the given AST.
    /// 
    /// Functions etc. are not evaluated, so this makes sense primarily for
    /// quoted data and literals.
    /// 
    /// Re-uses already existing strings and numbers, so the addresses returned
    /// may not be unique.
    pub fn for_node<'t>(
        &mut self,
        source: Source<'s>,
        value: &'t AstNode<'s>,
    ) -> Result<DataAddress, StaticDataError<'s, 't>> {
        Ok(match value {
            AstNode::Atom(atom) => {
                match atom.token().kind() {
                    TokenKind::Comment
                    | TokenKind::Ws
                    | TokenKind::LeftParen
                    | TokenKind::RightParen
                    | TokenKind::Quote => unreachable!(),
                    TokenKind::StringLit => {
                        let decoded =
                            decode_string(atom.fragment(source).source());
                        *self
                            .global_string_addresses
                            .entry(decoded)
                            .or_insert_with_key(|decoded| {
                                self.static_data
                                    .static_string(decoded.as_ref())
                            })
                    }
                    TokenKind::IntLit => {
                        let decoded = i32::from_str_radix(
                            atom.fragment(source).source(),
                            10,
                        )
                        .map_err(|_| {
                            StaticDataError::NumberParseError {
                                atom,
                                source: source,
                            }
                        })?;
                        *self
                            .global_number_addresses
                            .entry(decoded)
                            .or_insert_with(|| self.static_data.static_number(decoded))
                    }
                    TokenKind::FloatLit => todo!("floats are unimplemented"),
                    // identifiers in an escaped context, here the #' will be included for functions
                    TokenKind::FuncIdent => {
                        let value = atom.fragment(source).source();
                        *self
                            .global_identifier_addresses
                            .entry(value)
                            .or_insert_with(|| {
                                self.static_data.static_identifier(value)
                            })
                    }
                    // variable identifiers are as-is in an escaped context
                    TokenKind::Ident => {
                        let value = atom.fragment(source).source();
                        *self
                            .global_identifier_addresses
                            .entry(value)
                            .or_insert_with(|| {
                                self.static_data.static_identifier(value)
                            })
                    }
                }
            }
            AstNode::List(l) => {
                let mut successor = self.static_data.nil_data();
                for predecessor in l.elements().iter().rev() {
                    let predecessor =
                        self.for_node(source, predecessor)?;
                    successor = self
                        .static_data
                        .static_list_node(predecessor, successor);
                }
                successor
            }
            AstNode::Quoted(q) => {
                self.for_node(source, q.quoted())?
            }
        })
    }

    pub fn static_function(&mut self, addr: StaticFunctionAddress) -> DataAddress {
        *self
            .global_function_addresses
            .entry(addr)
            .or_insert_with_key(|&addr| {
                self.static_data.static_function(addr)
            })
    }

    pub fn function_table_entry(&mut self, addr: StaticFunctionAddress) -> FunctionTableIndex {
        *self.func_table_indexes
            .entry(addr)
            .or_insert_with_key(|&addr|
                self.static_data.function_table_entry(addr))
    }

    /// Creates a new non-shared place that statically holds the given data
    /// address.
    pub fn static_place(&mut self, addr: DataAddress) -> PlaceAddress {
        self.static_data.static_place(addr)
    }

    pub fn build(mut self) -> StaticData {
        self.static_data.build()
    }
}

pub enum StaticDataError<'s, 't> {
    NumberParseError {
        source: Source<'s>,
        atom: &'t Atom<'s>,
    },
}

impl<'s, 't> fmt::Display for StaticDataError<'s, 't> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &StaticDataError::NumberParseError { atom, source } => {
                writeln!(
                    f,
                    "number cannot be parsed as a 32-bit signed integer `{}`:",
                    atom.source_range().of(source).source()
                )?;
                writeln!(f, "{}", atom.fragment(source).source_context())
            }
        }
    }
}

impl<'s, 't> Diagnostic for StaticDataError<'s, 't> {
    fn kind(&self) -> crate::diagnostic::DiagnosticKind {
        crate::diagnostic::DiagnosticKind::Error
    }
}
