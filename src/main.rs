use clap::{Parser, Subcommand, ArgGroup};
use halcyon_lib::*;

use colored::{Colorize};
use std::{time::Instant};

mod config;
use config::*;

//---ARGS---
//args for building
//need at least config file or input path
#[derive(Debug, Parser)]
#[command(group(
    ArgGroup::new("build_source")
        .required(true)         
        .args(&["config_file", "input_path"]),
))]
pub struct BuildGroup {
    /// Config file path
    #[arg(short, long)]
    config_file: Option<String>,

    /// File to build
    #[arg(short, long)]
    input_path: Option<String>,

    /// Print status messages
    #[arg(short,long,action)]
    verbose: bool,

    /// Optional output file (only meaningful with --input-path)
    #[arg(short, long, requires = "input_path")]
    output_path: Option<String>,
}

//args for running
//need at least config file or input path
#[derive(Debug, Parser)]
#[command(group(
    ArgGroup::new("run_source")
        .required(true)         
        .args(&["config_file", "input_path"]),
))]
pub struct RunGroup{
    /// Config file path
    #[arg(short, long)]
    config_file: Option<String>,

    /// File to run
    #[arg(short, long)]
    input_path: Option<String>,

    /// Print status messages
    #[arg(short,long,action)]
    verbose: bool,

    /// Launch parameters for Halcyon program
    #[arg(short, long, num_args = 0..)]
    parameters: Option<Vec<String>>,
}

//args for initializing
//none required
#[derive(Debug, Parser)]
#[command(group(
ArgGroup::new("init")
    .required(false)         
    .args(&["config_path", "input_paths", "output_path"]),
))]
struct InitGroup {
    /// Config file path
    #[arg(short,long, default_value = "./config.toml")]
    config_path: Option<String>,

    /// Input files
    #[arg(short,long, default_value = "./main.hc")]
    input_paths: Option<Vec<String>>,

    /// Output path
    #[arg(short,long, default_value = "./a.wasm")]
    output_path: Option<String>,
}

//base commands
#[derive(Debug, Subcommand)]
enum Commands {
    /// Validate the program without producing output
    #[command(flatten_help = true)]
    Check(BuildGroup),
    /// Compile and link the program
    Build(BuildGroup),
    /// Compile, link, and execute the program
    Run(RunGroup),
    /// Initialize a new Halcyon project in the current directory
    Init(InitGroup),
}

#[derive(Parser, Debug)]
#[command(name = "hcc", about = "Halcyon compiler")]
struct CmdArgs {
    #[command(subcommand)]
    command: Commands,
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
        if config.verbose {println!("Checking {}...", infile.blue());}
        let file_as_string = std::fs::read_to_string(std::path::PathBuf::from(infile)).map_err(|e| e.to_string())?;
        let compilation_result = compile(&file_as_string);
        match compilation_result {
            std::result::Result::Err(err) => return std::result::Result::Err((infile.clone() + "\n" + &err).red()),
            _ => {},
        }
    }
    Ok(())

} 

// builds .wasm binary out of infiles in given config
fn build (config : &Config, with_binary : bool ) -> std::result::Result<Vec<u8>, colored::ColoredString> {
    // before building we might as well check if the input is valid
    check_valid(&config)?;
    let start_time = Instant::now();
    // combine all the input files together
    let mut file = String::from("");
    for infile in &config.infiles {
        file.push_str("\n");
        file.push_str(&mut std::fs::read_to_string(std::path::PathBuf::from(infile)).map_err(|e| e.to_string())?);
    }
    if config.verbose {println!("Building .wasm binary...")};
    // build the binary
    let binary = compile(&file)?;
    if with_binary {
        // write to a file if so desired
        std::fs::write(std::path::PathBuf::from(&config.outfile), &binary).map_err(|e| e.to_string())?;
        if config.verbose {println!("Built .wasm binary at {}", config.outfile.blue())};
    }
    
    if config.verbose {println!("{}! Build completed in {}ms", "Success".green(), start_time.elapsed().as_millis())};
    Ok(binary)
}

// runs infiles in a given config
fn build_and_run(config : &Config, args : Option<Vec<String>> ) -> std::result::Result<(), colored::ColoredString> {
    // build the binary
    let binary = build(config, false)?;
    if config.verbose { println!("Running...\n--------------------------------");}
    let start_time = Instant::now();
    // run it
    execute(binary, args.unwrap_or(vec![]));
    if config.verbose {println!("--------------------------------\nExecution Completed in {}ms", start_time.elapsed().as_millis())};
    Ok(())
}

//---CLI---
fn hcc_main() -> std::result::Result<(), colored::ColoredString> {
    let args = CmdArgs::parse();
    match args.command {
        Commands::Check(group) => {
            // Check if infiles compile
            // create config
            let config : Config = match (group.config_file, group.input_path) {
                (Some(cfg), None) => {
                    create_config_from_path(cfg)?
                }
                (None, Some(inp)) => {
                    create_config([inp].to_vec(), String::from("./a.wasm"), group.verbose || false)?
                }
                _ => unreachable!("Clap enforces mutual exclusion"),
            };
            // check if the config compiles
            check_valid(&config)?;
            println!("{}!", "Success".green());
        }
        Commands::Build( group ) => {
            // Compile infiles to .wasm binary
            // create config
            let config : Config = match (group.config_file, group.input_path, group.output_path) {
                (Some(cfg), None, None) => {
                    create_config_from_path(cfg)?
                }
                (None, Some(inp), out) => { 
                    create_config([inp].to_vec(), out.unwrap_or(String::from("./a.wasm")), group.verbose || false)?
                }
                _ => unreachable!("Clap enforces mutual exclusion"),
            };
            // build config
            build(&config,true)?;
        }
        Commands::Run (group) => {
            // Run infiles
            //create config
            let config: Config = match (group.config_file, group.input_path) {
                (Some(cfg), None) => {
                    create_config_from_path(cfg)?
                }
                (None, Some(inp)) => {
                    create_config([inp].to_vec(), String::from("./a.wasm"), group.verbose || false)?
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
                    println!("\"Config.toml\" already exists in this directory. Continue? (y/N)");
                    loop {
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).expect("Failed to read line");
                        match input.to_lowercase().trim() {
                            "y" => {break;}
                            "n" => {return Ok(())}
                            "" => {break;}
                            _ => {println!("Invalid input. Try again."); }
                        }
                    }
                },
                false => {}
            } 
            // create a config from input/defaults
            let cfg = create_config(group.input_paths.unwrap(), group.output_path.unwrap(), false)?;
            // write each infile as a .hc module
            for arg in cfg.infiles.clone() {
                let content = String::from("module ") + std::path::PathBuf::from(&arg).file_stem().unwrap().to_str().expect("Filename contains invalid characters") + " =\n(* Your code here! *)\n end";
                std::fs::write(std::path::PathBuf::from(arg), content).map_err(|e| e.to_string().red())?;
            }
            // write the config
            let config_contents = toml::to_string(&cfg).unwrap();
            std::fs::write(std::path::PathBuf::from("./Config.toml"), &config_contents).map_err(|e| e.to_string().red())?;
        }
    }
    Ok(())
}

fn main() {
    match hcc_main() {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
