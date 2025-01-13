use std::{
    fs,
    io::{self, Write},
    path::Path,
    usize,
};

use espflash::{flasher::Flasher, targets::Chip};
use flasher::print_board_info;
use miette::{Error, IntoDiagnostic, Result};
use serial::{get_port_handler, get_serial_ports_name, get_usbport_info};
use serialport::{SerialPortInfo, SerialPortType};

mod flasher;
mod serial;

fn main() {
    println!("self-esp toolkit");
    println!("\n>>>> select port <<<<\n");

    /////// GET SERIAL PORT
    ///////
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

    /////// FLASH ESP32
    ///////
    let mut flasher = Flasher::connect(*Box::new(serialport_interface), port_info, None, false)
        .expect("cannot get the flasher object");
    print_board_info(&mut flasher).expect("print cannot get board info");
}
