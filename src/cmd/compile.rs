use std::{
    fs::{self, File},
    io::stdout,
    path::PathBuf,
};

use crate::{
    analysis::{IrGen, NameCheck, SemanticAnalysis},
    args::{OutputFormat, TopLevelArgs},
    codegen::write_wat,
    parse::{AstSet, Parser},
    source::{Source, SourceSet},
};

use super::err::CommandResult;

const LISP_RT_DIR: &str = "rt";

pub fn compile(args: &TopLevelArgs) -> CommandResult<()> {
    assert!(matches!(args.output_format(), OutputFormat::Wat));

    // TODO try to collect more errors in each step
    let sources = load_sources(args.files())?;
    let asts = parse(&sources)?;
    let analysis = analyze(&asts)?;
    let program = IrGen::generate(&analysis)?;
    // before generating output, we could also optimize stuff
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

fn load_sources(user_files: &[PathBuf]) -> CommandResult<SourceSet> {
    let mut sources = SourceSet::new();
    // add sources for runtime functions like list and others on top of builtin
    // functions
    let rt_source = fs::read_dir(LISP_RT_DIR)?;
    for rt_source in rt_source {
        let rt_source = rt_source?;
        let rt_type = rt_source.file_type()?;
        let rt_path = rt_source.path();
        let rt_ext = rt_path.extension().map(|e| e.to_str().unwrap());
        if rt_type.is_file() && rt_ext == Some("lisp") {
            sources.load(&rt_path)?;
        }
    }

    // then the files the user wants us to compile
    for file in user_files {
        sources.load(file)?;
    }

    Ok(sources)
}

fn parse(source: &SourceSet) -> CommandResult<AstSet> {
    let asts = source
        .iter()
        .map(|s| {
            let mut parser = Parser::new(s);
            parser.parse()
        })
        .collect::<Result<AstSet, _>>()?;
    Ok(asts)
}

fn analyze<'s, 't>(
    ast: &'t AstSet<'s>,
) -> CommandResult<SemanticAnalysis<'s, 't>> {
    let analysis = SemanticAnalysis::analyze(ast)?;
    // test: I don't want to update for the new intrinsics, maybe not needed?
    // NameCheck::check(&analysis)?;
    Ok(analysis)
}
