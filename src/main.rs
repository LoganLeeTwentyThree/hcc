use clap::{Parser, Subcommand, ArgGroup};
use halcyon_lib::*;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Config {
   infiles: Vec<String>,
   outfile: String,
}

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
    output_path: Option<PathBuf>,
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
            move |_callee: Caller<'_, ()>, ptr: i32, len: i32| {
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

fn hcc_main() -> std::result::Result<(), String> {
    use toml::from_str;
    let args = CmdArgs::parse();
    match args.command {
        Commands::Check(group) => {
            match (group.config_file, group.input_path) {
                (Some(cfg), None) => {
                    let cfgfile = std::fs::read_to_string(std::path::PathBuf::from(cfg)).map_err(|e| e.to_string())?;
                    let config: Config = toml::from_str(&cfgfile).unwrap();
                    
                    
                    let mut file = String::from("");
                    for infile in config.infiles.into_iter() {
                        file.push_str("\n");
                        file.push_str(&mut std::fs::read_to_string(std::path::PathBuf::from(infile)).map_err(|e| e.to_string())?);
                    }
                    let byte_array = String::into_bytes(file);
                    let _binary = compile(&String::from_utf8_lossy(&byte_array))?;
                    println!("Success")
                }
                (None, Some(inp)) => {
                    println!("Checking {}...", inp);
                    let file = std::fs::read(inp).map_err(|e| e.to_string())?;
                    let _binary = compile(&String::from_utf8_lossy(&file))?;
                    println!("Success")
                }
                _ => unreachable!("Clap enforces mutual exclusion"),
            }
        }
        Commands::Build( group ) => {

            match (group.config_file, group.input_path, group.output_path) {
                (Some(cfg), None, None) => {
                    let cfgfile = std::fs::read_to_string(std::path::PathBuf::from(cfg)).map_err(|e| e.to_string())?;
                    let config: Config = toml::from_str(&cfgfile).unwrap();

                    let mut file = String::from("");
                    for infile in config.infiles.into_iter() {
                        //error check 
                        println!("Compiling \"{}...\"", infile);
                        let errortest = std::fs::read(&infile).map_err(|e| e.to_string())?;
                        let _binarytest = compile(&String::from_utf8_lossy(&errortest))?;
                        
                        //concat
                        file.push_str("\n");
                        file.push_str(&mut std::fs::read_to_string(std::path::PathBuf::from(infile)).map_err(|e| e.to_string())?);
                    }
                    let byte_array = String::into_bytes(file);
                    let binary = compile(&String::from_utf8_lossy(&byte_array))?;
                    std::fs::write(std::path::PathBuf::from(config.outfile), binary).map_err(|e| e.to_string())?;
                    
                }
                (None, Some(inp), out) => {
                    let file = std::fs::read(inp).map_err(|e| e.to_string())?;
                    let binary = compile(&String::from_utf8_lossy(&file))?;
                    std::fs::write(out.unwrap_or("./a.wasm".into()), binary).map_err(|e| e.to_string())?;//no matching required thanks to default "./a.wasm"
                }
                _ => unreachable!("Clap enforces mutual exclusion"),
            }

        }
        Commands::Run (group) => {
            match (group.config_file, group.input_path) {
                (Some(cfg), None) => {
                    let cfgfile = std::fs::read_to_string(std::path::PathBuf::from(cfg)).map_err(|e| e.to_string())?;
                    let config: Config = toml::from_str(&cfgfile).unwrap();

                    for infile in config.infiles.into_iter() {
                        let file = std::fs::read(std::path::PathBuf::from(infile)).map_err(|e| e.to_string())?;
                        let binary = compile(&String::from_utf8_lossy(&file))?;
                        execute(binary);
                    }
                }
                (None, Some(inp)) => {
                    let file = std::fs::read(inp).map_err(|e| e.to_string())?;
                    let binary = compile(&String::from_utf8_lossy(&file))?;
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
