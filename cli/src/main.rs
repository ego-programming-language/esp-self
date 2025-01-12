use std::{
    io::{self, Write},
    time::Duration,
    usize,
};

use serialport::{SerialPortInfo, SerialPortType};

mod serial;
// use espflash::{self, flasher::Flasher, targets::Chip};

fn main() {
    println!("self-esp toolkit");

    println!("\n>>>> select port <<<<\n");
    let ports = serial::detect_usb_serial_ports(true);
    let mut port_names = vec![];
    let ports = match ports {
        Ok(ports) => {
            port_names = ports
                .iter()
                .map(|port_info| port_info.port_name.clone())
                .collect::<Vec<_>>();
            ports
        }
        Err(_) => {
            panic!("Error getting serial ports")
        }
    };

    port_names
        .iter()
        .enumerate()
        .for_each(|(i, port)| println!("[{i}] {port}"));

    print!("> ");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let input = input.trim();
    let port_name = port_names[input
        .parse::<usize>()
        .expect("Input must be a number within the options range")]
    .clone();
    let baud_rate = 115200;
    let port;

    match serialport::new(port_name.clone(), baud_rate)
        .timeout(Duration::from_millis(100)) // Set a timeout
        .open() // Open the port
    {
        Ok(_port) => {
            println!("Serial port {} opened successfully!", port_name);
            port = _port;
        }
        Err(e) => {
            eprintln!("Failed to open serial port: {}", e);
        }
    }

    //serial::get_serial_port_info(, config);

    //espflash::connection::Connection::new(, );
}
