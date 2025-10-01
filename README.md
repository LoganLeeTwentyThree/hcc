# hcc
hcc is a command line utility for checking, building, and running Halcyon source files (.hc). 

## Commands:  
*  `build`  Compile and link the program
*  `run`    Compile, link, and execute the program
*  `help`   Print help message or the help of the given subcommand(s)

## Config
hcc config files are .toml format files that specify variables for configuring hcc  
`infiles`: A string array of .hc filenames to process  
`outfile`: A string .hc filename to output built .wasm files to  
`docfile` : A string path to output docs to

## Args
hcc supports a currently experimental feature to provide halcyon programs with arguments, like C's `argv`. 
To do so, run with the -p flag and specify one or more string arguments.
This will create an importable function in halcyon for each arg that returns that arg in binary form (`arg0, arg1, etc...`). 
For details on how to use this feature, check out `argdemo.hc`.
