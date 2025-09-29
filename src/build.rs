use halcyon_lib::*;
use gag::Gag;
use colored::{Colorize};

use crate::logging::*;


// builds .wasm binary out of input file
pub fn build (infiles : Vec<String>, outfile : Option<String> ) -> std::result::Result<Vec<Vec<u8>>, colored::ColoredString> {
    // combine all the .hc input files together
    let mut file = String::from("");
    let mut bins :Vec<Vec<u8>> = Vec::new();

    for infile in infiles {
        let path = std::path::PathBuf::from(&infile);
        match path.extension().unwrap().to_str(){
            Some("hc") => {
                file.push_str("\n");
                file.push_str(&mut std::fs::read_to_string(std::path::PathBuf::from(&infile))
                    .map_err(|e| e.to_string() + &infile)?);
            },
            Some("wasm") => bins.push(std::fs::read(path).unwrap()),
            _ => error("Build: Invalid file type detected"),
        }
        
    }

    // if .hc source files were provided, build binary (gag prevents side effects)
    if !file.is_empty(){
        debug("Build",&format!("Combined halcyon program:\n{}", file));
        info("Build","Building .wasm binary");
        let start_time =start_step("Build");
        let gag = Gag::stdout().unwrap();
        let binary = compile(&file)?;
        drop(gag);
        elapsed("Build", start_time);

        match outfile{
            Some(file) =>{
                // write to a file if so desired
                std::fs::write(std::path::PathBuf::from(&file), &binary)
                .map_err(|e| format!("{}: {} {}","Build error".red(), e.to_string(), &file))?;
                info("Build",&format!("Built .wasm binary at {}", file.blue()));
            },
            _ => {}
            
        }
        bins.push(binary);
    } else if outfile != None {
        warn("Build","Attempted to build with only .wasm input files. No output will be produced.")
    }

    // TODO: link multiple binaries if provided 
    
    
    Ok(bins)
}

// runs input files
pub fn build_and_run(infiles : Vec<String>, args : Option<Vec<String>> ) -> std::result::Result<(), colored::ColoredString> {
    
    // build the get binaries
    let binaries = build(infiles, None)?;
    for binary in binaries{
        info("Run","Running");
        // run it
        crate::execute(binary, args.clone().unwrap_or(vec![]));
    }
    Ok(())
}