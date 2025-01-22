pub mod flash;
pub mod load;

use load::Load;

use self::flash::Flash;

use std::env;

pub enum Command {
    Flash(Flash),
    Load(Load),
}

impl Command {
    pub fn parse() -> Command {
        let args: Vec<String> = env::args().collect();
        if args.len() >= 2 {
            let command = args[1].clone();
            let remaining_args = &args[2..];
            return Command::cmd_from_str(command.as_str(), remaining_args.to_vec());
        } else {
            // print help message instead of error
            println!("Command is required");
            std::process::exit(1); // to avoid types error
        };
    }
    fn cmd_from_str(command: &str, args: Vec<String>) -> Command {
        match command {
            "flash" => Command::Flash(Flash::new(args)),
            "load" => Command::Load(Load::new(args)),
            _ => {
                panic!("Unknown command")
            } // if unknown command, assumes it's a .ego file
        }
    }
    pub fn exec(&self) {
        match self {
            Command::Flash(v) => {
                let _ = v.exec();
            }
            Command::Load(v) => v.exec(),
        }
    }
}
