use std::{alloc::System, fs::File, io::Read, path::Path, process::exit};

pub struct Load {
    args: Vec<String>,
}

impl Load {
    pub fn new(args: Vec<String>) -> Load {
        Load { args }
    }
    pub fn exec(&self) {
        if self.args.len() < 1 {
            println!("A file path is required");
            exit(1);
        }

        let path = &self.args[0];
        let path = Path::new(path);
        let module_name = path
            .file_name()
            .expect("Modulename cannot be getted")
            .to_string_lossy()
            .into_owned();
        let mut ego_code = String::new();
        let mut file = File::open(path).expect("Cannot get file based on path");
        file.read_to_string(&mut ego_code)
            .expect("Cannot read file");

        println!("Compiling '{module_name}' to bytecode...");
        let bytecode = ego::gen_bytecode(module_name, ego_code, &vec![]);
        println!("Bytecode generated");
    }
}
