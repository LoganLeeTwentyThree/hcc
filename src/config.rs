use colored::{ColoredString, Colorize};
use crate::cli::Source;
use crate::logging::*;
//config file struct
#[derive(serde::Deserialize)]
#[derive(serde::Serialize)]
pub struct Config {
    pub infiles: Vec<String>,
    pub outfile: String,
    pub docfile: Option<String>,
}

pub fn resolve_config(
    source: Source,
    output: Option<String>,
    docs: Option<String>
) -> std::result::Result<Config, ColoredString> {
    match (source.config_file, source.input_path, output) {
        (Some(cfg), None, None) => create_config_from_path(cfg),
        (Some(cfg), None, Some(out)) => {
            let config = create_config_from_path(cfg)?;
            create_config(
                config.infiles, 
                out, 
                docs)
        },
        (_, Some(inp), out) => create_config(
            vec![inp], 
            out.unwrap_or("./a.wasm".into()), 
            docs),
        _ => unreachable!("Clap enforces mutual exclusion"),
    }
}

pub fn create_config_from_path(path : String) -> std::result::Result<Config, ColoredString>
{
    debug(&format!("Config: Creating config from \"{}\"", path));
    let cfgfile = std::fs::read_to_string(std::path::PathBuf::from(path)).map_err(|e| format!("{} {}", "Config error:\n".red(), e.to_string()))?;
    let cfg : Config = toml::from_str(&cfgfile).map_err(|e| e.to_string() + &"\nCould not create config".red())?;
    validate_config(&cfg)?;
    Ok(cfg)
}

pub fn create_config(ins : Vec<String>, out : String, dfile : Option<String>) -> std::result::Result<Config, ColoredString>
{
    log::debug!("Config: Creating config");
    let cfg : Config =
    Config {
        infiles: ins,
        outfile: out,
        docfile: dfile
    };

    validate_config(&cfg)?;
    Ok(cfg)
}

pub fn validate_config(cfg : &Config) -> Result<(), ColoredString>
{
    debug("Validating config");
    //check infiles for errors
    for arg in &cfg.infiles {
        debug(&format!("Config: Checking input file \"{}\" ", arg));
        let path= std::path::Path::new(&arg);
        if std::fs::exists(path).unwrap() == true {
            match path.extension().unwrap().to_str() {
                Some("hc") => {},
                Some("wasm") =>{},
                _ => return std::result::Result::Err(format!("{}: {} \"{}\"","Config Error:".red(), "Invalid input filename", &arg).into()),
            }
        }
        
    }

    debug(&format!("Config: Checking output file {} ", cfg.outfile));
    //check outfile for errors
    match std::path::Path::new(&cfg.outfile).extension().unwrap().to_str() {
        Some("wasm") => {},
        _ => return std::result::Result::Err(format!("{}: {} \"{}\"","Config error".red(), "Invalid output filename:", &cfg.outfile).into()),
    }

    if cfg.infiles.len() < 1
    {
        return std::result::Result::Err(String::from(format!("{} {}", "Config Error:".red(), "Please provide one or more input files!")).into())
    }

    if cfg.outfile.is_empty()
    {
        return std::result::Result::Err(String::from(format!("{} {}", "Config Error:".red(), "Please provide exactly one output file!")).into())
    }

    
    //check docfile if it exists
    match &cfg.docfile {
        None => {},
        Some(path) => {
            debug(&format!("Config: checking docfile \"{}\"", path));
            match std::path::Path::new(&path).extension().unwrap().to_str() {
                Some("md") => {},
                _ => return std::result::Result::Err(format!("{}: {} \"{}\"","Config error:".red(), "Invalid doc filename: ".red(), &path).into()),
            }
        }
    }

    Ok(())
}
