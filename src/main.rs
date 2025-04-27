use clap::Parser;

mod args;
mod cmd;
mod parse;

use args::{TopLevelArgs, Command};

fn main() {
    let args = args::TopLevelArgs::parse();
    let result = match &args.command {
        Command::Parse(parse) => {
            cmd::parse(parse)
        }
    };
    match result {
        Ok(_) => {},
        Err(e) => {
            eprint!("{e}");
            std::process::exit(1)
        }
    }
}
