use clap::{Parser, Subcommand, Args, ArgGroup};
use clap_verbosity_flag::{Verbosity, InfoLevel};
//---ARGS---
// Config/Input file args (reused in multiple places)
#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("source")
        .required(false)
        .args(&["config_file", "input_path"]),
))]
pub struct Source {
    /// Config file path
    #[arg(short, long, default_value = "./Config.toml")]
    pub config_file: Option<String>,

    /// File to build
    #[arg(short, long)]
    pub input_path: Option<String>,
}

/// Args for building
#[derive(Debug, Args)]
pub struct BuildGroup {
    /// Config/Input file
    #[command(flatten)]
    pub source: Source,

    /// Optional output file (only meaningful with --input-path)
    #[arg(short, long, requires = "input_path")]
    pub output_path: Option<String>,
}

/// Args for running
#[derive(Debug, Args)]
pub struct RunGroup {
    /// Config/Input file
    #[command(flatten)]
    pub source: Source,

    /// Launch parameters for Halcyon program
    #[arg(short, long, num_args = 0..)]
    pub parameters: Option<Vec<String>>,
}

/// Args for docs
#[derive(Debug, Args)]
pub struct DocGroup {
    /// Config/Input file
    #[command(flatten)]
    pub source: Source,

    /// File to write docs to
    #[arg(short, long, requires = "input_path", default_value = "./docs.md")]
    pub doc_file: Option<String>,
}

/// Args for initializing
#[derive(Debug, Args)]
pub struct InitGroup {
    /// Output path
    #[arg(short, long, default_value = "./a.wasm")]
    pub output_path: Option<String>,
}

/// Subcommands
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Validate the program without producing output
    Check(BuildGroup),
    /// Compile and link the program
    Build(BuildGroup),
    /// Compile, link, and execute the program
    Run(RunGroup),
    /// Initialize a new Halcyon project in the current directory
    Init(InitGroup),
    /// Create documentation based off line comments
    Doc(DocGroup),
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