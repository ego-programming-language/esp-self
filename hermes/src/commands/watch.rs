use notify::{event::ModifyKind, Config, RecommendedWatcher, RecursiveMode, Watcher};
use reqwest::Url;
use std::{
    fs::File,
    io::Read,
    path::Path,
    process::exit,
    sync::mpsc::channel,
    time::{Duration, Instant},
};

pub struct Watch {
    args: Vec<String>,
}

impl Watch {
    pub fn new(args: Vec<String>) -> Watch {
        Watch { args }
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

        // start watching file
        self.watch(module_name, path);
    }

    fn watch(&self, module_name: String, filepath: &Path) {
        println!("\x1b[34m");
        println!("┌────────");
        println!("│ Watching for changes in {}", module_name);
        println!("└────────────");
        println!("\x1b[0m");

        // first publish
        self.publish(module_name.clone(), filepath);

        // watcher
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher =
            Watcher::new(tx, Config::default()).expect("Get watcher");

        watcher
            .watch(filepath, RecursiveMode::NonRecursive)
            .expect("Fatch a file");

        let debounce_duration = Duration::from_millis(700); // debounce to group multiples modify triggers of the same event
        let last_event_time = Instant::now() - debounce_duration;

        loop {
            match rx.recv() {
                Ok(event) => {
                    let event = match event {
                        Ok(e) => e,
                        Err(e) => {
                            println!("Error while getting fs event: {:#?}", e);
                            exit(1);
                        }
                    };

                    match event.kind {
                        notify::EventKind::Modify(ModifyKind::Data(_content)) => {
                            // publish on every file content change
                            // and debounce
                            let now = Instant::now();
                            if now.duration_since(last_event_time) >= debounce_duration {
                                // Publicar cambios
                                self.publish(module_name.clone(), filepath);
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => println!("Watch error: {:?}", e),
            }
        }
    }

    fn publish(&self, module_name: String, filepath: &Path) {
        let mut ego_code = String::new();

        let mut file = File::open(filepath).expect("Cannot get file based on path");
        file.read_to_string(&mut ego_code)
            .expect("Cannot read file");
        println!("Compiling '{module_name}'...");
        let bytecode = ego::gen_bytecode(module_name, ego_code, &vec![]);

        // POST the bytecode to the esp32
        println!("Uploading bytecode to esp32");
        let host = &self.args[1];
        let client = reqwest::blocking::Client::new();
        let url = Url::parse(&format!("{host}/upload")).expect("Cannot parse ip to url format");
        let response = client
            .post(url)
            .header("Content-Type", "application/octet-stream")
            .header("User-Agent", "self-esp/cli")
            .body(bytecode)
            .send()
            .expect("Cannot get response from host");
        println!("'- upload status: {}", response.status());
        println!();
    }
}
