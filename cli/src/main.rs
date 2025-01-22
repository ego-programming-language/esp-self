use commands::Command;

mod commands;
mod flasher;
mod serial;

fn main() {
    let command = Command::parse();
    command.exec();
}
