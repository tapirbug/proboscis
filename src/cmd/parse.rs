use std::{
    fs::{self, File},
    io::Write as _,
    path::PathBuf,
};

use super::err::CommandResult;
use crate::{
    args::TopLevelArgs,
    cmd::err::CommandError,
    parse::Parser, source::SourceSet,
};

pub fn parse(args: &TopLevelArgs) -> CommandResult<()> {
    let mut source_set = SourceSet::new();
    for file in args.files() {
        source_set.load(file)?;
    }
    for source in source_set.iter() {
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
                eprintln!("file {} is valid.", source.path().unwrap().display());
                eprintln!("AST:");
                println!("{:#?}", ast)
            }
        }
    }
    Ok(())
}
