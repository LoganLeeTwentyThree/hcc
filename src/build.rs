use std::{time::Instant,time::Duration};
use indicatif::ProgressBar;
use halcyon_lib::*;
use gag::Gag;
use colored::{Colorize};

use crate::logging::*;
use crate::config::*;

// checks if the infiles in a given config are valid
pub fn check_valid(config : &Config) -> std::result::Result<(), colored::ColoredString> {
    // TODO: More checks?
    // does each infile compile?
    for infile in &config.infiles {
        info(&format!("Checking: {}", infile.blue()));
        let file_as_string = std::fs::read_to_string(std::path::PathBuf::from(infile))
            .map_err(|e| format!("{} {} {}", "Config error:".red(),  e.to_string(), &infile.red()))?; //this shouldnt trigger because the config should be valid, but just in case
        let gag = Gag::stdout().unwrap();
        let compilation_result = compile(&file_as_string);
        drop(gag);
        match compilation_result {
            std::result::Result::Err(err) => return std::result::Result::Err(format!("{}\n{}",infile.clone().red(), &err).into()),
            _ => {info(&format!("Success Checking {}", infile.blue()));},
        }
    }
    Ok(())

} 

// builds .wasm binary out of infiles in given config
pub fn build (config : &Config, with_binary : bool ) -> std::result::Result<Vec<u8>, colored::ColoredString> {
    // before building we might as well check if the input is valid
    check_valid(&config)?;

    //tracking
    let start_time =start_step("Build");

    // combine all the input files together
    let mut file = String::from("");
    for infile in &config.infiles {
        file.push_str("\n");
        file.push_str(&mut std::fs::read_to_string(std::path::PathBuf::from(infile))
            .map_err(|e| e.to_string() + infile)?);
    }

    debug(&format!("Combined halcyon program:\n{}", file));
    info("Building: .wasm binary");
    // build the binary (gag prevents side effects)
    let gag = Gag::stdout().unwrap();
    let binary = compile(&file)?;
    drop(gag);

    if with_binary {
        // write to a file if so desired
        std::fs::write(std::path::PathBuf::from(&config.outfile), &binary)
            .map_err(|e| format!("{}: {} {}","Build error".red(), e.to_string(), &config.outfile))?;
        info(&format!("Built .wasm binary at {}", config.outfile.blue()));
    }

    elapsed("Build", start_time);
    Ok(binary)
}

// runs infiles in a given config
pub fn build_and_run(config : &Config, args : Option<Vec<String>> ) -> std::result::Result<(), colored::ColoredString> {
    // build the binary
    let binary = build(config, false)?;
    info("Running");
    let start_time = Instant::now();
    //awesome progress bar
    let bar = ProgressBar::new_spinner();
    bar.set_message("Running");
    bar.enable_steady_tick(Duration::from_millis(100));
    // run it
    crate::execute(binary, args.unwrap_or(vec![]));
    bar.finish_and_clear();
    log::info!("Execution Completed in {}ms", start_time.elapsed().as_millis());
    Ok(())
}