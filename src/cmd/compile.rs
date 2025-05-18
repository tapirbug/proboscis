use std::{
    fs::{self, File},
    io::{self, Write, stdout},
    path::PathBuf,
};

use crate::{
    analysis::{IrGen, SemanticAnalysis},
    args::{OutputFormat, TopLevelArgs},
    codegen::{write_pirt, write_wat},
    diagnostic::Diagnostics,
    ir::Program,
    parse::{AstSet, Parser},
    source::SourceSet,
};

use super::err::CommandResult;

const LISP_RT_DIR: &str = "rt";

pub fn compile(args: &TopLevelArgs) -> CommandResult<()> {
    let mut diagnostics = Diagnostics::new();
    let format = args.output_format();

    let sources = load_sources(&mut diagnostics, args.files())?;
    let asts = parse(&mut diagnostics, &sources);

    // suppress output and quit early if sources are missing or a file doesn't parse
    diagnostics.ensure_no_errors()?;

    if let OutputFormat::Ast = format {
        // user only wants an AST, no need to analyze or generate code
        write_ast_out(args, asts)?;
        return Ok(());
    }

    let analysis = SemanticAnalysis::analyze(&mut diagnostics, &asts);

    // only continue to generating IR if semantic analysis did not produce errors
    diagnostics.ensure_no_errors()?;

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

fn load_sources(
    diagnostics: &mut Diagnostics,
    user_files: &[PathBuf],
) -> CommandResult<SourceSet> {
    let mut sources = SourceSet::new();
    // add sources for runtime functions like list and others on top of builtin
    // functions -> nothing will work without the runtime so we stop immediately on errors
    let rt_source = fs::read_dir(LISP_RT_DIR)?;
    for rt_source in rt_source {
        let rt_source = rt_source?;
        let rt_type = rt_source.file_type()?;
        let rt_path = rt_source.path();
        let rt_ext = rt_path.extension().map(|e| e.to_str().unwrap());
        if rt_type.is_file() && rt_ext == Some("lisp") {
            diagnostics.report_if_err(sources.load(&rt_path));
        }
    }

    // then the files the user wants us to compile, here we only report the error
    // and try to continue a bit to collect more errors.
    for file in user_files {
        diagnostics.report_if_err(sources.load(file));
    }

    Ok(sources)
}

fn parse<'d, 's>(
    diagnostics: &'d mut Diagnostics,
    source: &'s SourceSet,
) -> AstSet<'s> {
    source
        .iter()
        .filter_map(|s| {
            let mut parser = Parser::new(s);
            // report parse errors immediately,
            // but continue with parsing other files in case there are more errors to report
            diagnostics.ok(parser.parse())
        })
        .collect()
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
