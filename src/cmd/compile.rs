use std::{fs::File, io::{stdout}};

use crate::{analysis::{MultiStringTable, StringTable}, args::{OutputFormat, TopLevelArgs}, codegen::write_wat, ir::Program, parse::{Parser, Source}};

use super::err::CommandResult;

pub fn compile(args: &TopLevelArgs) -> CommandResult<()> {
    assert!(matches!(args.output_format(), OutputFormat::Wat));
    let sources = Source::load_many(args.files())?;
    let mut string_tables = vec![];
    for source in &sources {
        let ast = Parser::new(source).parse()?;
        let strings = StringTable::analyze(source, &ast);
        string_tables.push(strings);
    }
    let string_table : MultiStringTable = string_tables.into_iter().collect();
    let program = Program::new(string_table);
    match args.output_path() {
        Some(path) => {
            let mut file = File::create(path)?;
            write_wat(&mut file, &program)?;
        },
        None => {
            let mut stdout = stdout().lock();
            write_wat(&mut stdout, &program)?;
        }
    }
    Ok(())
}
