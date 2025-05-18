use clap::Parser;

mod analysis;
mod args;
mod cmd;
mod codegen;
mod diagnostic;
mod ir;
mod parse;
mod source;

fn main() {
    let args = args::TopLevelArgs::parse();
    let result = cmd::compile(&args);
    match result {
        Ok(_) => {}
        Err(e) => {
            eprint!("{e}");
            std::process::exit(1)
        }
    }
}
