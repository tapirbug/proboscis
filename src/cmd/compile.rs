use std::{
    fs::{self, File},
    io::{self, Write, stdout},
    path::PathBuf,
};

use crate::{
    analysis::{IrGen, SemanticAnalysis},
    args::{OutputFormat, TopLevelArgs},
    codegen::{write_pirt, write_wat},
    ir::Program,
    parse::{AstSet, Parser},
    source::{Source, SourceSet},
};

use super::err::CommandResult;

const LISP_RT_DIR: &str = "rt";

pub fn compile(args: &TopLevelArgs) -> CommandResult<()> {
    let format = args.output_format();

    // TODO try to collect more errors in each step
    let sources = load_sources(args.files())?;
    let asts = parse(&sources)?;
    if let OutputFormat::Ast = format {
        // user only wants an AST, no need to analyze or generate code
        write_ast_out(args, asts)?;
        return Ok(());
    }

    let analysis = analyze(&asts)?;
    let program = IrGen::generate(&analysis)?;

    // here we would call an optimizer if we had one

    if let OutputFormat::Pirt = format {
        // user only wants IR, let's give em the unoptimized version
        write_pirt_out(args, &program)?;
        return Ok(());
    }

    assert!(matches!(format, OutputFormat::Wat)); // last option
    write_wat_out(args, &program)?;
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
    Ok(analysis)
}

fn write_ast_out(args: &TopLevelArgs, asts: AstSet) -> io::Result<()> {
    match args.output_path() {
        Some(path) => {
            let mut file = File::create(path)?;
            for ast in asts {
                write!(&mut file, "{:#?}\n\n", ast)?;
            }
        }
        None => {
            let mut stdout = stdout().lock();
            for ast in asts {
                write!(&mut stdout, "{:#?}\n\n", ast)?;
            }
        }
    }
    Ok(())
}

fn write_pirt_out(args: &TopLevelArgs, ir: &Program) -> io::Result<()> {
    match args.output_path() {
        Some(path) => {
            let mut file = File::create(path)?;
            write_pirt(&mut file, ir)?;
        }
        None => {
            let mut stdout = stdout().lock();
            write_pirt(&mut stdout, ir)?;
        }
    }
    Ok(())
}

fn write_wat_out(args: &TopLevelArgs, program: &Program) -> io::Result<()> {
    match args.output_path() {
        Some(path) => {
            let mut file = File::create(path)?;
            write_wat(&mut file, program)?;
        }
        None => {
            let mut stdout = stdout().lock();
            write_wat(&mut stdout, program)?;
        }
    }
    Ok(())
}
