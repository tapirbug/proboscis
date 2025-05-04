use std::{fs::File, io::stdout};

use crate::{
    analysis::{
        IrGen, NameCheck, SemanticAnalysis
    },
    args::{OutputFormat, TopLevelArgs},
    codegen::write_wat,
    parse::Parser, source::{Source, SourceSet},
};

use super::err::CommandResult;

pub fn compile(args: &TopLevelArgs) -> CommandResult<()> {
    assert!(matches!(args.output_format(), OutputFormat::Wat));
    let mut sources = SourceSet::new();
    for file in args.files() {
        sources.load(file)?;
    }
    for source in sources.iter() {
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
