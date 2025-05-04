use std::{fs::File, io::stdout};

use crate::{
    analysis::{
        IrGen, NameCheck, SemanticAnalysis
    },
    args::{OutputFormat, TopLevelArgs},
    codegen::write_wat,
    parse::{AstSet, Parser}, source::{Source, SourceSet},
};

use super::err::CommandResult;

pub fn compile(args: &TopLevelArgs) -> CommandResult<()> {
    assert!(matches!(args.output_format(), OutputFormat::Wat));
    let mut sources = SourceSet::new();
    // TODO try to collect more errors in each step
    for file in args.files() {
        sources.load(file)?;
    }
    let asts  = sources.iter().map(|s| {
        let mut parser = Parser::new(s);
        parser.parse()
    }).collect::<Result<AstSet, _>>()?;

    let analysis = SemanticAnalysis::analyze(&asts)?;
    NameCheck::check(&analysis)?;
    let program = IrGen::generate(&analysis)?;

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

    Ok(())
}
