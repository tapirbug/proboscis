use std::{fs::{self, File}, path::PathBuf, io::Write as _};

use crate::{args::TopLevelArgs, cmd::err::CommandError, parse::{Parser, Source}};
use super::err::CommandResult;

pub fn parse(args: &TopLevelArgs) -> CommandResult<()> {
    let sources = load_sources(args.files())?;
    for source in &sources {
        let mut parser = Parser::new(source);
        let ast = parser.parse()?;
        match args.output_path() {
            // write to file if one is specified, no decoration
            Some(path) => {
                let mut file = File::create(path)?;
                write!(&mut file, "{:#?}", ast)?;
            },
            // write to stdout with some extra decoration on stderr
            None => {
                eprintln!("file {} is valid.", source.path().display());
                eprintln!("AST:");
                println!("{:#?}", ast)
            }
        }
    }
    Ok(())
}

pub fn load_sources(paths: &[PathBuf]) -> CommandResult<Vec<Source>> {
    let sources = paths.iter().map(Source::load).collect::<Result<Vec<_>, _>>();
    sources.map_err(CommandError::from)
}
