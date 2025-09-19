use colored::{ColoredString, Colorize};
//config file struct
#[derive(serde::Deserialize)]
#[derive(serde::Serialize)]
pub struct Config {
    pub infiles: Vec<String>,
    pub outfile: String,
    pub docfile: Option<String>,
}

pub fn create_config_from_path(path : String) -> std::result::Result<Config, ColoredString>
{
    log::debug!("Creating config from {}", path);
    let cfgfile = std::fs::read_to_string(std::path::PathBuf::from(path)).map_err(|e| format!("{} {}", "Config error:\n".red(), e.to_string()))?;
    let cfg : Config = toml::from_str(&cfgfile).map_err(|e| e.to_string() + &"\nCould not create config".red())?;
    validate_config(&cfg)?;
    Ok(cfg)
}

pub fn create_config(ins : Vec<String>, out : String, dfile : Option<String>) -> std::result::Result<Config, ColoredString>
{
    log::debug!("Creating config");
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
    log::debug!("Validating config");
    //check infiles for errors
    for arg in &cfg.infiles {
        let path= std::path::Path::new(&arg);
        std::fs::exists(path).map_err(|e| format!("{} {} {}", "Config error:".red(),  e.to_string(), &path.to_string_lossy().red()))?;
        match path.extension() {
            Some(ext) => if ext.to_str().expect("File extension should contain valid unicode") != "hc" {return std::result::Result::Err(format!("{} {}","Invalid input filename: ".red(), &arg).into())},
            None => return std::result::Result::Err(format!("{} {}", "Invalid input filename:".red(), &arg).into()),
        }
    }

    //check outfile for errors
    match std::path::Path::new(&cfg.outfile).extension() {
        Some(ext) => if ext.to_str().expect("File extension should contain valid unicode") != "wasm" {
            return std::result::Result::Err(format!("{} {}","Invalid output filename:".red(), &cfg.outfile).into())
        },
        None => return std::result::Result::Err(format!("{} {}","Invalid output filename:".red(), &cfg.outfile).into()),
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
            match std::path::Path::new(&path).extension() {
                Some(ext) => if ext.to_str().expect("File extension should contain valid unicode.") != "md" {return std::result::Result::Err(format!("{} {}","Invalid doc filename: ".red(), &path).into())},
                None => return std::result::Result::Err(format!("{} {}","Invalid doc filename: ".red(), &path).into()),
            }
        }
    }

    Ok(())
}