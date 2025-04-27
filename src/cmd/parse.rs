use std::path::PathBuf;

use crate::{args::ParseArgs, cmd::err::CommandError, parse::{Parser, Source}};
use super::err::CommandResult;

pub fn parse(args: &ParseArgs) -> CommandResult<()> {
    let sources = load_sources(args.files())?;
    for source in &sources {
        let mut parser = Parser::new(source);
        let ast = parser.parse()?;
        eprintln!("file {} is valid.", source.path().display());
        eprintln!("AST:");
        println!("{:#?}", ast);
    }
    Ok(())
}

pub fn load_sources(paths: &[PathBuf]) -> CommandResult<Vec<Source>> {
    let sources = paths.iter().map(Source::load).collect::<Result<Vec<_>, _>>();
    sources.map_err(CommandError::from)
}
