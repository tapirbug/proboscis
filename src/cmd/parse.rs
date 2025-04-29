use std::{
    fs::{self, File},
    io::Write as _,
    path::PathBuf,
};

use super::err::CommandResult;
use crate::{
    args::TopLevelArgs,
    cmd::err::CommandError,
    parse::{Parser, Source},
};

pub fn parse(args: &TopLevelArgs) -> CommandResult<()> {
    let sources = Source::load_many(args.files())?;
    for source in &sources {
        let mut parser = Parser::new(source);
        let ast = parser.parse()?;
        match args.output_path() {
            // write to file if one is specified, no decoration
            Some(path) => {
                let mut file = File::create(path)?;
                write!(&mut file, "{:#?}", ast)?;
            }
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
