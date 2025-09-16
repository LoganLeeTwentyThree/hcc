//config file struct
#[derive(serde::Deserialize)]
#[derive(serde::Serialize)]
pub struct Config {
   pub infiles: Vec<String>,
   pub outfile: String,
   pub verbose: bool,
   pub docfile: Option<String>,
}

pub fn create_config_from_path(path : String) -> std::result::Result<Config, String>
{
    let cfgfile = std::fs::read_to_string(std::path::PathBuf::from(path)).map_err(|e| e.to_string())?;
    let cfg : Config = toml::from_str(&cfgfile).map_err(|e| e.to_string())?;
    validate_config(&cfg)?;
    Ok(cfg)
}

pub fn create_config(ins : Vec<String>, out : String, v : bool, dfile : Option<String>) -> std::result::Result<Config, String>
{
    let cfg : Config =
    Config {
        infiles: ins,
        outfile: out,
        verbose: v,
        docfile: dfile
    };

    validate_config(&cfg).map_err(|e| e.to_string())?;
    Ok(cfg)
}

pub fn validate_config(cfg : &Config) -> Result<(), String>
{
    if cfg.verbose {println!("Validating config...")};
    //check infiles for errors
    for arg in &cfg.infiles {
        let path= std::path::Path::new(&arg);
        match path.extension() {
            Some(ext) => if ext.to_str().expect("File should have filetype!") != "hc" {return std::result::Result::Err(String::from("Invalid input filename: ") + &arg)},
            None => return std::result::Result::Err(String::from("Invalid input filename: ") + &arg),
        }
    }

    //check outfile for errors
    match std::path::Path::new(&cfg.outfile).extension() {
        Some(ext) => if ext.to_str().expect("File should have filetype!") != "wasm" {
            return std::result::Result::Err(String::from("Invalid output filename: ") + &cfg.outfile)
        },
        None => return std::result::Result::Err(String::from("Invalid output filename: ") + &cfg.outfile),
    }

    if cfg.infiles.len() < 1
    {
        return std::result::Result::Err(String::from("Config Error: Please provide one or more input files!"))
    }

    if cfg.outfile.is_empty()
    {
        return std::result::Result::Err(String::from("Config Error: Please provide exactly one output file!"))
    }

    Ok(())
}