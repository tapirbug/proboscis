use std::{collections::{HashMap, HashSet}, fs::File, io::stdout, os::linux::raw::stat};

use crate::{
    analysis::{
        FunctionDefinition, GlobalDefinition, IrGen, MultiStringTable, NameCheck, SemanticAnalysis, StringTable
    },
    args::{OutputFormat, TopLevelArgs},
    codegen::write_wat,
    ir::{DataAddress, FunctionsBuilder, Program, StaticDataBuilder, StaticFunctionAddress},
    parse::{Parser, Source},
};

use super::err::CommandResult;

pub fn compile(args: &TopLevelArgs) -> CommandResult<()> {
    assert!(matches!(args.output_format(), OutputFormat::Wat));
    let sources = Source::load_many(args.files())?;
    let mut static_data = StaticDataBuilder::new();
    let mut functions = FunctionsBuilder::new();
    let nil_address = static_data.static_nil();
    let nil_place = static_data.static_place(nil_address);
    let mut added_static_strings = HashMap::<String, DataAddress>::new();
    let mut added_functions = HashMap::<String, StaticFunctionAddress>::new();
    for source in &sources {
        let ast = Parser::new(source).parse()?;
        
        let analysis = SemanticAnalysis::analyze(source, &ast)?;
        NameCheck::check(&analysis)?;
        let program = IrGen::generate(&analysis)?;

        // FIXME this does not really support multiple input files
        match args.output_path() {
            Some(path) => {
                let mut file = File::create(path)?;
                write_wat(&mut file, &program)?;
            }
            None => {
                let mut stdout = stdout().lock();
                write_wat(&mut stdout, &program)?;
            }
        }
    }
    Ok(())
}
