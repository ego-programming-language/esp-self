use std::{
    io::{self, Write},
    time::Duration,
    usize,
};

use espflash::{interface::Interface, targets::Chip};
use miette::Error;
use serial::{get_port_handler, get_serial_ports_name, get_usbport_info};
use serialport::{SerialPortInfo, SerialPortType};

mod serial;
// use espflash::{self, flasher::Flasher, targets::Chip};

fn main() {
    println!("self-esp toolkit");
    println!("\n>>>> select port <<<<\n");

    let ports = match serial::detect_usb_serial_ports(true) {
        Ok(ports) => ports,
        Err(_) => panic!("Cannot get ports"),
    };
    let port_names = serial::get_serial_ports_name(&ports);

    // print portnames
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
    let selected_port = input
        .parse::<usize>()
        .expect("Input must be a number within the options range");

    let port_name = port_names[selected_port].clone();
    let baud_rate = 115200;
    //let port = get_port_handler(&port_name, baud_rate);
    let port_info = match get_usbport_info(&port_name) {
        Some(info) => info,
        None => {
            panic!("UsbPortInfo is a required info")
        }
    };

    let serialport_interface =
        match espflash::interface::Interface::new(&ports[selected_port], None, None) {
            Ok(interface) => interface,
            Err(_) => {
                panic!("Cannot get serialport interface")
            }
        };
    let mut connection = espflash::connection::Connection::new(serialport_interface, port_info);

    /////BIG REFACTOR MUST BE DONE HERE/////
    connection.begin().expect("error control");
    connection
        .set_timeout(Duration::from_secs(3))
        .expect("error control");

    let detected_chip = {
        // Detect which chip we are connected to.
        let magic = connection
            .read_reg(0x40001000)
            .expect("not magic number getted");
        let detected_chip = Chip::from_magic(magic).expect("chip cannot be detected");
        detected_chip
    };

    println!("{:#?}", detected_chip);
}
