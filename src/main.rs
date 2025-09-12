use clap::{Parser, Subcommand, ArgGroup};
use halcyon_lib::*;
use serde::Deserialize;
use colored::{ColoredString, Colorize};
use std::time::{Instant};

//config file struct
#[derive(Deserialize)]
struct Config {
   infiles: Vec<String>,
   outfile: String,
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
fn check_valid(file_name : &String) -> std::result::Result<(), colored::ColoredString> {
    println!("Checking {}...", file_name.blue());
    let bytes = file_name.as_bytes();
    let mut errors : Vec<ColoredString> = vec![];
    if bytes[(bytes.len() - 3)..] != [b'.',b'h',b'c'] 
    {
        errors.push("Format Error: Input files must be Halcyon source files! (.hc)".truecolor(255,204,204));
    }
    let file_as_string = std::fs::read_to_string(std::path::PathBuf::from(file_name)).map_err(|e| e.to_string())?;
    compile(&file_as_string)?;
    match errors.len() {
        0 => println!("{} {} checks",file_name.blue(), "passed".green()),
        _ => 
        {
            println!("{} {} {} {}",file_name.blue(), "failed".red(), errors.len().to_string().red(), if errors.len() > 1 {"checks"} else {"check"});
            for error in errors {
                println!("{}", error);
            }
            return std::result::Result::Err("One or more input files didn't pass checks".red())
        }
    };
    Ok(())

} 

fn build (in_file_names : Vec<String>, out_file_name : &String, with_binary : bool) -> std::result::Result<Vec<u8>, colored::ColoredString> {
    let start_time = Instant::now();
    let mut file = String::from("");
    for infile in in_file_names.into_iter() {
        check_valid(&infile)?;
        
        file.push_str("\n");
        file.push_str(&mut std::fs::read_to_string(std::path::PathBuf::from(infile)).map_err(|e| e.to_string())?);
    }
    let binary = compile(&file)?;
    if with_binary {
        std::fs::write(std::path::PathBuf::from(out_file_name), &binary).map_err(|e| e.to_string())?;
        println!("Built .wasm binary at {}", out_file_name.blue());
    }
    
    println!("Build completed in {}ms", start_time.elapsed().as_millis());
    Ok(binary)
}

//---CLI---
fn hcc_main() -> std::result::Result<(), colored::ColoredString> {
    let args = CmdArgs::parse();
    match args.command {
        Commands::Check(group) => {
            match (group.config_file, group.input_path) {
                (Some(cfg), None) => {
                    let cfgfile = std::fs::read_to_string(std::path::PathBuf::from(cfg)).map_err(|e| e.to_string())?;
                    let config: Config = toml::from_str(&cfgfile).unwrap();
                    
                    for infile in config.infiles.into_iter() {
                        check_valid(&infile)?;
                    }
                }
                (None, Some(inp)) => {
                    check_valid(&inp)?;
                }
                _ => unreachable!("Clap enforces mutual exclusion"),
            }
        }
        Commands::Build( group ) => {

            match (group.config_file, group.input_path, group.output_path) {
                (Some(cfg), None, None) => {
                    let cfgfile = std::fs::read_to_string(std::path::PathBuf::from(cfg)).map_err(|e| e.to_string())?;
                    let config: Config = toml::from_str(&cfgfile).unwrap();
                    build(config.infiles.to_vec(), &config.outfile, true)?;
                }
                (None, Some(inp), out) => {
                    build([inp].to_vec(), &out.unwrap_or(String::from("./a.wasm")),true)?;
                }
                _ => unreachable!("Clap enforces mutual exclusion"),
            }

        }
        Commands::Run (group) => {
            match (group.config_file, group.input_path) {
                (Some(cfg), None) => {
                    let cfgfile = std::fs::read_to_string(std::path::PathBuf::from(cfg)).map_err(|e| e.to_string())?;
                    let config: Config = toml::from_str(&cfgfile).unwrap();
                    let binary = build(config.infiles.to_vec(), &config.outfile, false)?;
                    println!("Running...\n--------------------------------");
                    execute(binary);
                    
                }
                (None, Some(inp)) => {
                    let binary = build([inp].to_vec(), &String::from("Spink"), false)?;
                    println!("Running...\n--------------------------------");
                    execute(binary);
                }
                _ => unreachable!("Clap enforces mutual exclusion"),
            }
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
