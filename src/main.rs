use clap::{Parser, Subcommand, ArgGroup};
use halcyon_lib::*;
use serde::Deserialize;
use colored::{ColoredString, Colorize};
use std::{time::Instant};

//config file struct
#[derive(Deserialize)]
#[derive(serde::Serialize)]
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

#[derive(Debug, Parser)]
#[command(group(
    ArgGroup::new("run")
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
    

    let mut index = 0;
    for arg in args{
        let name : &str = &(String::from("arg") + &index.to_string());
        linker.
            func_wrap("args",
            name,
                move |index: i64| -> i64 {
                let binding = arg.chars().rev().collect::<String>();
                let bytes = binding.as_bytes();
                let start:usize = index.try_into().unwrap_or(bytes.len());
                if start >= bytes.len() {
                    return 0; 
                }
                
                bytes[start] as i64
        })
        .expect("Fail");
        index += 1;
        
    }

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

fn build_and_run(config : &Config, args : Option<Vec<String>> ) -> std::result::Result<(), colored::ColoredString> {
    let binary = build(config, false)?;
    if config.verbose { println!("Running...\n--------------------------------");}
    let start_time = Instant::now();
    execute(binary, args.unwrap_or(vec![]));
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
                        verbose: group.verbose || false
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
                        verbose: group.verbose || false
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
                        outfile: String::from("./a.wasm"),
                        verbose: group.verbose || false
                    }
                }
                _ => unreachable!("Clap enforces mutual exclusion"),
            };

            build_and_run(&config, group.parameters)?;
        }
        Commands::Init (group) => {
            for arg in group.input_paths.clone().unwrap() {
                let path= std::path::Path::new(&arg);
                match path.extension() {
                    Some(ext) => if ext.to_str().expect("File should have filetype!") != "hc" {return std::result::Result::Err((String::from("Invalid input filename: ") + &arg).red())},
                    None => return std::result::Result::Err((String::from("Invalid input filename: ") + &arg).red()),
                }
                let content = String::from("module ") + path.file_stem().unwrap().to_str().unwrap() + " =\n(* Your code here! *)\n end";
                std::fs::write(std::path::PathBuf::from(arg), content).map_err(|e| e.to_string().red())?;
            }

            //check outfile for errors
            match std::path::Path::new(&group.output_path.clone().unwrap()).extension() {
                Some(ext) => if ext.to_str().expect("File should have filetype!") != "wasm" {return std::result::Result::Err((String::from("Invalid output filename: ") + &group.output_path.unwrap()).red())},
                None => return std::result::Result::Err((String::from("Invalid input filename: ") + &group.output_path.unwrap()).red()),
            }

            let cfg : Config = Config {
                infiles: group.input_paths.unwrap(),
                outfile: group.output_path.unwrap(),
                verbose: false,
            };
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
