use std::path::PathBuf;

use clap::{Parser, Subcommand, Args};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(name = "proboscis")]
#[command(about = "Haphazard experimental LISP compiler", long_about = None)]
pub struct TopLevelArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// checks files without generating output
    Parse(ParseArgs),
}

#[derive(Args)]
pub struct ParseArgs {
    /// files to check for syntax without generating output
    files: Vec<PathBuf>,
}

impl ParseArgs {
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }
}
