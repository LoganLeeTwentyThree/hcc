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
    
    //print string function
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
    

    //creates arg functions
    let mut index = 0;
    for arg in args{
        let name : &str = &(String::from("arg") + &index.to_string());
        linker.
            func_wrap("args",
            name,
                move |index: i64| -> i64 {
                //due to strange cast errors in halcyon, i need to reverse the arg here
                let binding = arg.chars().rev().collect::<String>();
                let bytes = binding.as_bytes();
                let start:usize = index.try_into().unwrap_or(bytes.len());

                //prevent invalid indexing
                if start >= bytes.len() {
                    return 0; 
                }
                
                //return byte at requested index
                bytes[start] as i64
        })
        .expect("Fail");
        index += 1;
        
    }

    let _instance = linker.instantiate(&mut store, &module).unwrap();
}