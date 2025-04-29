use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, Args, ValueEnum};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(name = "proboscis")]
#[command(about = "Haphazard experimental LISP compiler", long_about = None)]
pub struct TopLevelArgs {
    /// files to compile or check
    files: Vec<PathBuf>,
    /// output file, if omitted write to stdout
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
    #[arg(short, long, value_enum, default_value_t)]
    format: OutputFormat
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OutputFormat {
    /// web assembly text
    Wat,
    /// raw AST for debugging
    Ast
}

impl Default for OutputFormat {
    fn default() -> Self {
        return Self::Wat
    }
}

impl TopLevelArgs {
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    pub fn output_path(&self) -> Option<&Path> {
        self.output.as_ref().map(|o| o.as_ref())
    }

    pub fn output_format(&self) -> OutputFormat {
        self.format
    }
}
