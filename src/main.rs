use clap::{Parser, Subcommand, ArgGroup};
use halcyon_lib::*;
use serde::Deserialize;
use colored::{ColoredString, Colorize};
use std::{time::Instant};

//config file struct
#[derive(Deserialize)]
struct Config {
   infiles: Vec<String>,
   outfile: String,
   verbose: bool,
}

//---ARGS---
#[derive(Debug, Parser)]
#[command(group(
    ArgGroup::new("build_source")
        .required(true)         
        .args(&["config_file", "input_path", "output_path"])
))]
pub struct BuildGroup {
    /// Config file path
    #[arg(short, long)]
    config_file: Option<String>,

    /// File to build
    #[arg(short, long)]
    input_path: Option<String>,

    #[arg(short,long,action)]
    verbose: bool,

    /// Optional output file (only meaningful with --input-path)
    #[arg(short, long, requires = "input_path")]
    output_path: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Validate the program without producing output
    #[command(flatten_help = true)]
    Check(BuildGroup),
    /// Compile and link the program
    Build(BuildGroup),
    /// Compile, link, and execute the program
    Run(BuildGroup),
}

#[derive(Parser, Debug)]
#[command(name = "hcc", about = "Halcyon compiler")]
struct CmdArgs {
    #[command(subcommand)]
    command: Commands,
}

//---WASM---
pub fn execute(wasm: Vec<u8>) {
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
    let _instance = linker.instantiate(&mut store, &module).unwrap();
}

//---CLI_HELPERS---
fn check_valid(config : &Config) -> std::result::Result<(), colored::ColoredString> {

    
    let mut errors : Vec<ColoredString> = vec![];

    for infile in &config.infiles {
        if config.verbose {println!("Checking {}...", infile.blue());}
        let bytes = infile.as_bytes();
        if bytes[(bytes.len() - 3)..] != [b'.',b'h',b'c'] 
        {
            errors.push("Format Error: Input files must be Halcyon source files! (.hc)".truecolor(255,204,204));
        }
        let file_as_string = std::fs::read_to_string(std::path::PathBuf::from(infile)).map_err(|e| e.to_string())?;
        let compilation_result = compile(&file_as_string);
        match compilation_result {
            std::result::Result::Err(err) => return std::result::Result::Err((infile.clone() + "\n" + &err).red()),
            _ => {},
        }
        match errors.len() {
        0 => if config.verbose {println!("{} {} checks",infile.blue(), "passed".green())},
        _ => 
            {
                println!("{} {} {} {}",infile.blue(), "failed".red(), errors.len().to_string().red(), if errors.len() > 1 {"checks"} else {"check"});
                for error in errors {
                    println!("{}", error);
                }
                return std::result::Result::Err("One or more input files didn't pass checks".red())
            }
        };
    }
    Ok(())

} 

fn build (config : &Config, with_binary : bool ) -> std::result::Result<Vec<u8>, colored::ColoredString> {
    if config.infiles.len() < 1
    {
        return std::result::Result::Err("Config Error: Please provide one or more input files!".red())
    }
    check_valid(&config)?;
    let start_time = Instant::now();
    let mut file = String::from("");
    for infile in &config.infiles {
        file.push_str("\n");
        file.push_str(&mut std::fs::read_to_string(std::path::PathBuf::from(infile)).map_err(|e| e.to_string())?);
    }
    if config.verbose {println!("Building .wasm binary...")};
    let binary = compile(&file)?;
    if with_binary {
        std::fs::write(std::path::PathBuf::from(&config.outfile), &binary).map_err(|e| e.to_string())?;
        if config.verbose {println!("Built .wasm binary at {}", config.outfile.blue())};
    }
    
    if config.verbose {println!("{}! Build completed in {}ms", "Success".green(), start_time.elapsed().as_millis())};
    Ok(binary)
}

fn build_and_run(config : &Config) -> std::result::Result<(), colored::ColoredString> {
    let binary = build(config, false)?;
    if config.verbose { println!("Running...\n--------------------------------");}
    let start_time = Instant::now();
    execute(binary);
    if config.verbose {println!("--------------------------------\nExecution Completed in {}s", start_time.elapsed().as_secs())};
    Ok(())
}

//---CLI---
fn hcc_main() -> std::result::Result<(), colored::ColoredString> {
    let args = CmdArgs::parse();
    match args.command {
        Commands::Check(group) => {
            
            
            let config : Config = match (group.config_file, group.input_path) {
                (Some(cfg), None) => {
                    let cfgfile = std::fs::read_to_string(std::path::PathBuf::from(cfg)).map_err(|e| e.to_string())?;
                    let temp : Config = toml::from_str(&cfgfile).unwrap();
                    Config {
                        infiles: temp.infiles,
                        outfile: temp.outfile,
                        verbose: temp.verbose || group.verbose
                    }
                }
                (None, Some(inp)) => {
                    Config {
                        infiles: [inp].to_vec(),
                        outfile: String::from("./a.wasm"),
                        verbose: group.verbose
                    }
                }
                _ => unreachable!("Clap enforces mutual exclusion"),
            };
            check_valid(&config)?;
        }
        Commands::Build( group ) => {

            let config : Config = match (group.config_file, group.input_path, group.output_path) {
                (Some(cfg), None, None) => {
                    let cfgfile = std::fs::read_to_string(std::path::PathBuf::from(cfg)).map_err(|e| e.to_string())?;
                    let temp : Config = toml::from_str(&cfgfile).unwrap();
                    Config {
                        infiles: temp.infiles,
                        outfile: temp.outfile,
                        verbose: temp.verbose || group.verbose
                    }
                }
                (None, Some(inp), out) => {
                    Config {
                        infiles: [inp].to_vec(),
                        outfile: out.unwrap_or(String::from("./a.wasm")),
                        verbose: group.verbose
                    }
                    
                }
                _ => unreachable!("Clap enforces mutual exclusion"),
            };

            build(&config,true)?;

        }
        Commands::Run (group) => {
            let config: Config = match (group.config_file, group.input_path) {
                (Some(cfg), None) => {
                    let cfgfile = std::fs::read_to_string(std::path::PathBuf::from(cfg)).map_err(|e| e.to_string())?;
                    let temp : Config = toml::from_str(&cfgfile).unwrap();
                    Config {
                        infiles: temp.infiles,
                        outfile: temp.outfile,
                        verbose: temp.verbose || group.verbose
                    }
                    
                }
                (None, Some(inp)) => {
                    Config {
                        infiles: [inp].to_vec(),
                        outfile: group.output_path.unwrap_or(String::from("./a.wasm")),
                        verbose: group.verbose
                    }
                }
                _ => unreachable!("Clap enforces mutual exclusion"),
            };

            build_and_run(&config)?;
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
