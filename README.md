# hcc
hcc is a command line utility for checking, building, and running Halcyon source files (.hc). 

## Commands:  
*  `check`  Validate the program without producing output
*  `build`  Compile and link the program
*  `run`    Compile, link, and execute the program
*  `init`   Initialize a config file and main file
*  `doc`    Creates documentation based off comment annotations
*  `help`   Print this message or the help of the given subcommand(s)

## Config
hcc config files are .toml format files that specify variables for configuring hcc  
`infiles`: A string array of .hc filenames to process  
`outfile`: A string .hc filename to output built .wasm files to  
`verbose` : A boolean to turn on or off verbose mode

## Args
hcc supports a currently experimental feature to provide halcyon programs with arguments, like C's `argv`. 
To do so, run with the -p flag and specify one or more string arguments.
This will create an importable function in halcyon for each arg that returns that arg in binary form (`arg0, arg1, etc...`). 
For details on how to use this feature, check out `argdemo.hc`.

## Docs
You can add documentation to a halcyon source file inside of comments.
The `doc` command looks for three fields: `@title:`, `@signature:`, and `@description:`, then formats them to markdown in an output file.
`@description` must end with an @ symbol as a delimiter.
`argdemo.hc` also contains example documentation comments.
