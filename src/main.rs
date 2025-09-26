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

use crate::pdm::add_dependency;
mod pdm;

fn hcc_main() -> std::result::Result<(), colored::ColoredString> {
    let args = CmdArgs::parse();
    env_logger::Builder::new()
        .filter_module("hcc",args.verbose.log_level_filter())
        .init();
    match args.command {
        Commands::Check(group) => 
        {
            // Check if infiles compile
            // create config
            let config = resolve_config(group.source, None, None)?;
            // check if the config compiles
            check_valid(&config)?;
            success("Success!");
        }
        Commands::Build( group ) => 
        {
            // Compile infiles to .wasm binary
            // create config
            let config = resolve_config(group.source, group.output_path, None)?;
            // build config
            build(&config,true)?;
        }
        Commands::Run (group) => 
        {
            // Run infiles
            //create config
            let config = resolve_config(group.source, None, None)?;
            //run config
            build_and_run(&config, group.parameters)?;
            
            
        }
        Commands::Init (group) => 
        {
            // Initialize a new halcyon project
            // check if config already exists
            match std::fs::exists("./Config.toml").unwrap() {
                true => {
                    warn("Init","\"Config.toml\" already exists in this directory. Continue? (y/N)");
                    loop {
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).expect("Failed to read line");
                        match input.to_lowercase().trim() {
                            "y" => {break;}
                            "n" => {return Ok(())}
                            "" => {break;}
                            _ => {warn("Init","Invalid input. Try again."); }
                        }
                    }
                },
                false => {}
            } 

            println!("Input Project Name: ");
            let mut proj_name = String::new();
            std::io::stdin().read_line(&mut proj_name).expect("Failed to read line");
            
            
            // create a config from input/defaults
            let mut cfg = create_config(
                vec!["./main.hc".into()], 
                group.output_path.unwrap(), 
                Some("./docs.md".into()),
                None)?;
                
            // add package
            cfg.package =  Some(
                Package {
                name: proj_name.trim().into(),
                version: "0.1.0".into(),
            });
            
            // write each infile as a .hc module
            for arg in cfg.build.infiles.clone() {
                let content = String::from("module ") + std::path::PathBuf::from(&arg).file_stem().unwrap().to_str().expect("Filename contains invalid characters") + " =\n(* Your code here! *)\nend";
                std::fs::write(std::path::PathBuf::from(arg), content)
                    .map_err(|e| e.to_string().red())?;
            }
            // write the config
            write_config(&cfg, "./Config.toml".into())?;

            if !group.no_git {
                // make a git repo
                let repo = git2::Repository::init(".")
                    .map_err(|e| e.to_string())?;

                let content = "# Halcyon build artifacts\n*.wasm";
                std::fs::write(".gitignore", content).map_err(|e| e.to_string())?;
                
                info("Init", &format!("Initialized empty Git repository at {:?}", repo.path()));
            }
            

        }
        Commands::Add(group)=>
        {
            let new_package = add_dependency(group.url.clone())?;
            add_dep_to_config(&new_package, &group.url, &group.config_file)?;            
            info("Add", &format!("Successfully added {} as a dependency.", &new_package));
        }
        Commands::Update=>
        {

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
            error(&format!("{e}"));
            std::process::exit(1);
        }
    }
}
