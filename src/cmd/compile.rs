use std::{fs::File, io::{stdout}};

use crate::{analysis::{FunctionDefinition, GlobalDefinition, MultiStringTable, StringTable}, args::{OutputFormat, TopLevelArgs}, codegen::write_wat, ir::Program, parse::{Parser, Source}};

use super::err::CommandResult;

pub fn compile(args: &TopLevelArgs) -> CommandResult<()> {
    assert!(matches!(args.output_format(), OutputFormat::Wat));
    let sources = Source::load_many(args.files())?;
    let mut string_tables = vec![];
    for source in &sources {
        let ast = Parser::new(source).parse()?;
        let strings = StringTable::analyze(source, &ast);
        // find constant strings
        // find constant numbers
        // find definitions for global variables
        // find functions, including the function that is the global scope
        // check that all names refer to something valid
        // generate code
        string_tables.push(strings);

        let mut root_code = vec![];
        let mut function_definitions = vec![];
        let mut global_definitions = vec![];
        for root_node in &ast {
            // try parsing root-level element as a function first
            let def = FunctionDefinition::extract(source, root_node)?;
            match def {
                Some(def) => {
                    function_definitions.push(def);
                    continue;
                },
                None => {}
            }

            // then as a global
            let def = GlobalDefinition::extract(source, root_node)?;
            match def {
                Some(def) => {
                    global_definitions.push(def);
                },
                None => {}
            }

            // all other cases are considered to be top-level code
            root_code.push(root_node);
        }
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
