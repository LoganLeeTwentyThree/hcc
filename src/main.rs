use clap::{Parser};
use colored::{Colorize};

mod config;
use config::*;
mod cli;
use cli::*;
mod parse;
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
        Commands::Check(group) => {
            // Check if infiles compile
            // create config
            let config = resolve_config(group.source, None, None)?;
            // check if the config compiles
            check_valid(&config)?;
            success("Success!");
        }
        Commands::Build( group ) => {
            // Compile infiles to .wasm binary
            // create config
            let config = resolve_config(group.source, group.output_path, None)?;
            // build config
            build(&config,true)?;
        }
        Commands::Run (group) => {
            // Run infiles
            //create config
            let config = resolve_config(group.source, None, None)?;
            // run config
            build_and_run(&config, group.parameters)?;
        }
        Commands::Init (group) => {
            // Initialize a new halcyon project
            // check if config already exists
            match  std::fs::exists("./Config.toml").unwrap() {
                true => {
                    warn("\"Config.toml\" already exists in this directory. Continue? (y/N)");
                    loop {
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).expect("Failed to read line");
                        match input.to_lowercase().trim() {
                            "y" => {break;}
                            "n" => {return Ok(())}
                            "" => {break;}
                            _ => {warn("Invalid input. Try again."); }
                        }
                    }
                },
                false => {}
            } 
            // create a config from input/defaults
            let cfg = create_config(
                vec!["./main.hc".into()], 
                group.output_path.unwrap(), 
                Some("./docs.md".into()))?;
                
            // write each infile as a .hc module
            for arg in cfg.infiles.clone() {
                let content = String::from("module ") + std::path::PathBuf::from(&arg).file_stem().unwrap().to_str().expect("Filename contains invalid characters") + " =\n(* Your code here! *)\nend";
                std::fs::write(std::path::PathBuf::from(arg), content).map_err(|e| e.to_string().red())?;
            }
            // write the config
            let config_contents = toml::to_string(&cfg).unwrap();
            std::fs::write(std::path::PathBuf::from("./Config.toml"), &config_contents).map_err(|e| e.to_string().red())?;
        }
        Commands::Doc(group) =>
        {
            let config = resolve_config(group.source, None, group.doc_file)?;
            parse::create_docs(config)?;
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
            error(&format!("\n{e}"));
            std::process::exit(1);
        }
    }
}
