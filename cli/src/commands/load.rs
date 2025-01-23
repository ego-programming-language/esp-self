use std::{alloc::System, fs::File, io::Read, path::Path, process::exit};

use reqwest::Url;

pub struct Load {
    args: Vec<String>,
}

impl Load {
    pub fn new(args: Vec<String>) -> Load {
        Load { args }
    }
    pub fn exec(&self) {
        if self.args.len() < 2 {
            println!("Load needs: <filepath> <host>");
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

        // POST the bytecode to the esp32
        println!("Uploading bytecode to esp32");
        let host = &self.args[1];
        let client = reqwest::blocking::Client::new();
        let url = Url::parse(&format!("{host}/upload")).expect("Cannot parse ip to url format");
        let response = client
            .post(url)
            .header("Content-Type", "application/octet-stream")
            .body(bytecode)
            .send()
            .expect("Cannot get response from host");
        println!("'-request status: {}", response.status());
    }
}
