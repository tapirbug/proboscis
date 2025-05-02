use clap::Parser;

mod analysis;
mod args;
mod cmd;
mod codegen;
mod ir;
mod parse;
mod rt;

use args::{OutputFormat, TopLevelArgs};

fn main() {
    let args = args::TopLevelArgs::parse();
    let result = match args.output_format() {
        OutputFormat::Wat => cmd::compile(&args),
        OutputFormat::Ast => cmd::parse(&args),
    };
    match result {
        Ok(_) => {}
        Err(e) => {
            eprint!("{e}");
            std::process::exit(1)
        }
    }
}
