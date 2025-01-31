pub mod configure;
pub mod flash;
pub mod load;
pub mod watch;

use configure::Configure;
use flash::Flash;
use load::Load;
use watch::Watch;

use std::env;

pub enum Command {
    Flash(Flash),
    Load(Load),
    Watch(Watch),
    Configure(Configure),
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
            "watch" => Command::Watch(Watch::new(args)),
            "configure" => Command::Configure(Configure::new(args)),
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
            Command::Watch(v) => v.exec(),
            Command::Configure(v) => v.exec(),
        }
    }
}
