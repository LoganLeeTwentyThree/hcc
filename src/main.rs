use std::{env::home_dir, path::PathBuf};

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
use toml::Table;
use wasm::*;

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
            match  std::fs::exists("./Config.toml").unwrap() {
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
            // create a config from input/defaults
            let cfg = create_config(
                vec!["./main.hc".into()], 
                group.output_path.unwrap(), 
                Some("./docs.md".into()),
                None)?;
                
            // write each infile as a .hc module
            for arg in cfg.build.infiles.clone() {
                let content = String::from("module ") + std::path::PathBuf::from(&arg).file_stem().unwrap().to_str().expect("Filename contains invalid characters") + " =\n(* Your code here! *)\nend";
                std::fs::write(std::path::PathBuf::from(arg), content)
                    .map_err(|e| e.to_string().red())?;
            }
            // write the config
            let config_contents = toml::to_string(&cfg).unwrap();
            std::fs::write(std::path::PathBuf::from("./Config.toml"), &config_contents)
                .map_err(|e| e.to_string().red())?;

            if !group.no_git {
                // make a git repo
                let repo = git2::Repository::init(".")
                    .map_err(|e| e.to_string())?;

                let content = r#"
                # Halcyon build artifacts
                *.wasm"#;
                std::fs::write(".gitignore", content).map_err(|e| e.to_string())?;
                
                info("Init", &format!("Initialized empty Git repository at {:?}", repo.path()));
            }
            

        }
        Commands::Add(group)=>
        {
            validate_dependency(group.url.clone())?;


            let hcc_path : PathBuf = [home_dir().unwrap(), PathBuf::from("hc")].iter().collect();
            if !std::fs::exists(hcc_path.clone()).unwrap() {
                std::fs::create_dir(hcc_path.clone())
                    .map_err(|e| e.to_string().red())?;
            }

            git2::Repository::clone(&group.url, hcc_path.join(group.name.clone()))
                .map_err(|e| e.to_string().red())?;

            let config = create_config_from_path(group.config_file.clone())?;
            
            let mut new_deps = match config.dependencies {
                Some(list) => list,
                None => Table::new(),
            };

            new_deps.insert(group.name, toml::Value::String(group.url));

            let new_config = Config {
                dependencies: Some(new_deps),
                ..config
            };

            let config_contents = toml::to_string(&new_config).unwrap();
            std::fs::write(std::path::PathBuf::from(group.config_file), &config_contents)
                .map_err(|e| e.to_string().red())?;
                
            
            
            
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
