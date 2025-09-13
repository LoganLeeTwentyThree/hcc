# hcc
hcc is a command line utility for checking, building, and running Halcyon source files (.hc). 

## Usage
hcc \<command\> <--config-file <CONFIG_FILE>|--input-path <INPUT_PATH>|--output-path <OUTPUT_PATH>>  

Commands:  
*  `check`  Validate the program without producing output
*  `build`  Compile and link the program
*  `run`    Compile, link, and execute the program
*  `help`   Print this message or the help of the given subcommand(s)

## Config
hcc config files are .toml format files that specify two variables  
`infiles`: A string array of .hc filenames to process  
`outfile`: A string .hc filename to output built .wasm files to  
`verbose` : A boolean to turn on or off verbose mode
