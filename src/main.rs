use clap::{Parser, Subcommand, Args, ArgGroup};
use halcyon_lib::*;
use gag::Gag;


use colored::{Colorize};
use std::{time::Instant,time::Duration};
use clap_verbosity_flag::{Verbosity, InfoLevel};
use indicatif::ProgressBar;


mod config;
use config::*;
mod parse;

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
    config_file: Option<String>,

    /// File to build
    #[arg(short, long)]
    input_path: Option<String>,
}

/// Args for building
#[derive(Debug, Args)]
pub struct BuildGroup {
    /// Config/Input file
    #[command(flatten)]
    source: Source,

    /// Optional output file (only meaningful with --input-path)
    #[arg(short, long, requires = "input_path")]
    output_path: Option<String>,
}

/// Args for running
#[derive(Debug, Args)]
pub struct RunGroup {
    /// Config/Input file
    #[command(flatten)]
    source: Source,

    /// Launch parameters for Halcyon program
    #[arg(short, long, num_args = 0..)]
    parameters: Option<Vec<String>>,
}

/// Args for docs
#[derive(Debug, Args)]
pub struct DocGroup {
    /// Config/Input file
    #[command(flatten)]
    source: Source,

    /// File to write docs to
    #[arg(short, long, requires = "input_path", default_value = "./docs.md")]
    doc_file: Option<String>,
}

/// Args for initializing
#[derive(Debug, Args)]
pub struct InitGroup {
    /// Output path
    #[arg(short, long, default_value = "./a.wasm")]
    output_path: Option<String>,
}

/// Subcommands
#[derive(Debug, Subcommand)]
enum Commands {
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
struct CmdArgs {
    #[command(subcommand)]
    command: Commands,
    /// Verbosity
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

//---WASM---
pub fn execute(wasm: Vec<u8>, args: Vec<String> ) {
    use wasmtime::*;
    let mut config = Config::default();
    config.wasm_gc(true);
    config.wasm_function_references(true);
    let engine = Engine::new(&config).unwrap();
    let module = Module::new(&engine, &wasm).unwrap();
    let mut linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    let memory = Memory::new(&mut store, MemoryType::new(1, None)).unwrap();
    
    //print string function
    linker
        .func_wrap(
            "sys",
            "print_string",
            move |_callee: Caller<'_, ()>, ptr: i64, len: i64| {
                let mut buffer = vec![0; len as usize];
                memory.read(_callee, ptr as usize, &mut buffer).unwrap();
                let s = String::from_utf8(buffer).unwrap();
                println!("{s}");
            },
        )
        .unwrap()
        .define(&mut store, "sys", "memory", Extern::Memory(memory))
        .unwrap();
    

    //creates arg functions
    let mut index = 0;
    for arg in args{
        let name : &str = &(String::from("arg") + &index.to_string());
        linker.
            func_wrap("args",
            name,
                move |index: i64| -> i64 {
                //due to strange cast errors in halcyon, i need to reverse the arg here
                let binding = arg.chars().rev().collect::<String>();
                let bytes = binding.as_bytes();
                let start:usize = index.try_into().unwrap_or(bytes.len());

                //prevent invalid indexing
                if start >= bytes.len() {
                    return 0; 
                }
                
                //return byte at requested index
                bytes[start] as i64
        })
        .expect("Fail");
        index += 1;
        
    }

    let _instance = linker.instantiate(&mut store, &module).unwrap();
}

//---CLI_HELPERS---
// checks if the infiles in a given config are valid
fn check_valid(config : &Config) -> std::result::Result<(), colored::ColoredString> {
    // TODO: More checks?
    // does each infile compile?
    for infile in &config.infiles {
        log::info!("Checking: {}", infile.blue());
        let file_as_string = std::fs::read_to_string(std::path::PathBuf::from(infile))
            .map_err(|e| format!("{} {} {}", "Config error:".red(),  e.to_string(), &infile.red()))?; //this shouldnt trigger because the config should be valid, but just in case
        let gag = Gag::stdout().unwrap();
        let compilation_result = compile(&file_as_string);
        drop(gag);
        match compilation_result {
            std::result::Result::Err(err) => return std::result::Result::Err(format!("{}\n{}",infile.clone().red(), &err).into()),
            _ => {log::info!("Success Checking {}", infile.blue());},
        }
    }
    Ok(())

} 

// builds .wasm binary out of infiles in given config
fn build (config : &Config, with_binary : bool ) -> std::result::Result<Vec<u8>, colored::ColoredString> {
    // before building we might as well check if the input is valid
    check_valid(&config)?;

    //tracking
    let start_time = Instant::now();

    // combine all the input files together
    let mut file = String::from("");
    for infile in &config.infiles {
        file.push_str("\n");
        file.push_str(&mut std::fs::read_to_string(std::path::PathBuf::from(infile))
            .map_err(|e| e.to_string() + infile)?);
    }

    log::debug!("Combined halcyon program:\n{}", file);
    log::info!("Building: .wasm binary");
    // build the binary (gag prevents side effects)
    let gag = Gag::stdout().unwrap();
    let binary = compile(&file)?;
    drop(gag);

    if with_binary {
        // write to a file if so desired
        std::fs::write(std::path::PathBuf::from(&config.outfile), &binary)
            .map_err(|e| format!("{}: {} {}","Build error".red(), e.to_string(), &config.outfile))?;
        log::info!("Built .wasm binary at {}", config.outfile.blue());
    }

    log::info!("{}! Build completed in {}ms", "Success".green(), start_time.elapsed().as_millis());
    Ok(binary)
}

// runs infiles in a given config
fn build_and_run(config : &Config, args : Option<Vec<String>> ) -> std::result::Result<(), colored::ColoredString> {
    // build the binary
    let binary = build(config, false)?;
    log::info!("Running");
    let start_time = Instant::now();
    //awesome progress bar
    let bar = ProgressBar::new_spinner();
    bar.set_message("Running");
    bar.enable_steady_tick(Duration::from_millis(100));
    // run it
    execute(binary, args.unwrap_or(vec![]));
    bar.finish_and_clear();
    log::info!("Execution Completed in {}ms", start_time.elapsed().as_millis());
    Ok(())
}

//---CLI---
fn hcc_main() -> std::result::Result<(), colored::ColoredString> {
    let args = CmdArgs::parse();
    env_logger::Builder::new()
        .filter_module("hcc",args.verbose.log_level_filter())
        .init();
    match args.command {
        Commands::Check(group) => {
            // Check if infiles compile
            // create config
            let config : Config = match (group.source.config_file, group.source.input_path) {
                (Some(cfg), None) => {
                    create_config_from_path(cfg)?
                }
                (None, Some(inp)) => {
                    create_config([inp].to_vec(), String::from("./a.wasm"), None)?
                }
                _ => unreachable!("Clap enforces mutual exclusion"),
            };
            // check if the config compiles
            check_valid(&config)?;
            log::info!("{}!", "Success".green());
        }
        Commands::Build( group ) => {
            // Compile infiles to .wasm binary
            // create config
            let config : Config = match (group.source.config_file, group.source.input_path, group.output_path) {
                (Some(cfg), None, None) => {
                    create_config_from_path(cfg)?
                }
                (None, Some(inp), out) => { 
                    create_config([inp].to_vec(), out.unwrap_or(String::from("./a.wasm")), None)?
                }
                _ => unreachable!("Clap enforces mutual exclusion"),
            };
            // build config
            build(&config,true)?;
        }
        Commands::Run (group) => {
            // Run infiles
            //create config
            let config: Config = match (group.source.config_file, group.source.input_path) {
                (Some(cfg), None) => {
                    create_config_from_path(cfg)?
                }
                (None, Some(inp)) => {
                    create_config([inp].to_vec(), String::from("./a.wasm"), None)?
                }
                _ => unreachable!("Clap enforces mutual exclusion"),
            };
            // run config
            build_and_run(&config, group.parameters)?;
        }
        Commands::Init (group) => {
            // Initialize a new halcyon project
            // check if config already exists
            match  std::fs::exists("./Config.toml").unwrap() {
                true => {
                    log::warn!("\"Config.toml\" already exists in this directory. Continue? (y/N)");
                    loop {
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).expect("Failed to read line");
                        match input.to_lowercase().trim() {
                            "y" => {break;}
                            "n" => {return Ok(())}
                            "" => {break;}
                            _ => {log::warn!("Invalid input. Try again."); }
                        }
                    }
                },
                false => {}
            } 
            // create a config from input/defaults
            let cfg = create_config(
                vec![String::from("./main.hc")], 
                group.output_path.unwrap(), 
                Some(String::from("./docs.md")))?;
                
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
            
            let config : Config = match (group.source.config_file, group.source.input_path, group.doc_file)
            {
                (Some(cfg), None, _) => {
                    create_config_from_path(cfg)?
                }
                (None, Some(inp), Some(out)) => {
                create_config([inp].to_vec(), out.clone(), Some(out.clone()))?
                }
            _ => unreachable!("Clap enforces mutual exclusion"),
            };

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
            log::error!("\n{e}");
            std::process::exit(1);
        }
    }
}
