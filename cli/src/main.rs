use std::{
    fs,
    io::{self, Write},
    path::Path,
    usize,
};

use espflash::{
    cli::print_board_info,
    connection::reset::{ResetAfterOperation::HardReset, ResetBeforeOperation::DefaultReset},
    flasher::{Flasher, ProgressCallbacks},
    targets::Chip,
};
use indicatif::{ProgressBar, ProgressStyle};
use serial::{
    detect_usb_serial_ports, get_port_handler, get_serial_ports_name, get_serialport_info,
    get_usbport_info,
};

mod flasher;
mod serial;

fn main() {
    println!("self-esp toolkit");
    println!("\n>>>> select port <<<<\n");

    /////// GET USER INPUT FOR SERIALPORT
    ///////
    let ports = match detect_usb_serial_ports(true) {
        Ok(ports) => ports,
        Err(_) => panic!("Cannot get ports"),
    };
    let port_names = get_serial_ports_name(&ports);

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

    /////// GET SERIALPORT
    ///////
    let port_name = port_names[selected_port].clone();
    let serialport_info = match get_serialport_info(&port_name) {
        Some(info) => info,
        None => {
            panic!("SerialPortInfo is a required info")
        }
    };
    let usbport_info = get_usbport_info(&serialport_info);
    let serialport = get_port_handler(&serialport_info);

    /////// FLASH ESP32
    ///////
    let mut flasher = Flasher::connect(
        *Box::new(serialport),
        usbport_info,
        Some(115_200),
        false,
        false,
        false,
        Some(Chip::Esp32),
        HardReset,
        DefaultReset,
    )
    .expect("Cannot get the flasher");

    print_board_info(&mut flasher).expect("print cannot get board info");

    // DETECT USED CHIP
    // let mut connection = espflash::connection::Connection::new(
    //     *Box::new(serialport), // TTyPort
    //     usbport_info,          // UsbPortInfo
    //     HardReset,
    //     DefaultReset,
    // );
    // let detected_chip = {
    //     // Detect which chip we are connected to.
    //     let magic = connection
    //         .read_reg(0x40001000)
    //         .expect("not magic number getted");
    //     let detected_chip = Chip::from_magic(magic).expect("chip cannot be detected");
    //     detected_chip
    // };
}
