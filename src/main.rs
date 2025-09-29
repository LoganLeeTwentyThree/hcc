use clap::{Parser};

mod cli;
use cli::*;
mod build;
use build::*;
mod logging;
use logging::*;
mod wasm;
use wasm::*;


fn hcc_main() -> std::result::Result<(), colored::ColoredString> {
    let args = CmdArgs::parse();
    env_logger::Builder::new()
        .filter_module("hcc",args.verbose.log_level_filter())
        .init();
    match args.command {
        Commands::Build( group ) => 
        {
            // Compile infiles to .wasm binary
            // build config
            build(group.source, Some(group.output_path))?;
        }
        Commands::Run (group) => 
        {
            // Run infiles
            build_and_run(group.source, group.parameters)?;
        }
        Commands::Version =>
        {
            println!("hcc version: {}", env!("CARGO_PKG_VERSION"));
        }
    }
    Ok(())
}

fn main() {
    human_panic::setup_panic!();
    match hcc_main() {
        Ok(()) => (),
        Err(e) => {
            error(&format!("{e}"));
            std::process::exit(1);
        }
    }
}
