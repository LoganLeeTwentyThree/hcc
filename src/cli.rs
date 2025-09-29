use clap::{Parser, Subcommand, Args};
use clap_verbosity_flag::{Verbosity, InfoLevel};
//---ARGS---
/// Args for building
#[derive(Debug, Args)]
pub struct BuildGroup {
    /// Input files
    #[arg(short, long, num_args = 1..)]
    pub input_file: Vec<String>,

    /// Output file 
    #[arg(short, long, default_value = "./a.wasm")]
    pub output_path: String,
}

/// Args for running
#[derive(Debug, Args)]
pub struct RunGroup {
    /// Input file
    #[arg(short, long, num_args = 1..)]
    pub input_file: Vec<String>,

    /// Launch parameters for Halcyon program
    #[arg(short, long, num_args = 0..)]
    pub parameters: Option<Vec<String>>,
}

/// Subcommands
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Compile and link the program
    Build(BuildGroup),
    /// Compile, link, and execute the program
    Run(RunGroup),
    /// Print version
    Version,
}

#[derive(Parser, Debug)]
#[command(name = "hcc", about = "Halcyon compiler")]
pub struct CmdArgs {
    #[command(subcommand)]
    pub command: Commands,
    /// Verbosity
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,
}