use std::{
    alloc::System,
    fs::File,
    io::{self, Read, Write},
    path::Path,
    process::exit,
    thread,
    time::Duration,
};

use espflash::connection::{
    reset::{ResetAfterOperation, ResetBeforeOperation},
    Connection,
};
use reqwest::Url;

use crate::serial::{
    get_port_handler, get_serialport_info, get_usbport_info, serial_port_selector,
};

pub struct Configure {
    args: Vec<String>,
}

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
impl Configure {
    pub fn new(args: Vec<String>) -> Configure {
        Configure { args }
    }
    pub fn exec(&self) {
        // get selected usb-serial
        let port_name = serial_port_selector();
        let serialport_info = match get_serialport_info(&port_name) {
            Some(info) => info,
            None => {
                panic!("SerialPortInfo is a required info")
            }
        };
        let usbport_info = get_usbport_info(&serialport_info);
        let serialport = get_port_handler(&serialport_info);

        let mut connection = Connection::new(
            serialport,
            usbport_info,
            ResetAfterOperation::NoReset,
            ResetBeforeOperation::DefaultReset,
        );
        if let Err(err) = connection.begin() {
            println!("err: {}", err)
        }
        connection.set_timeout(CONNECTION_TIMEOUT).expect("to wait");
        let mut serial = connection.into_serial();
        thread::sleep(Duration::from_secs(2));

        // get config credentials
        print!("wifi ssid: ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let wifi_ssid = input.trim();

        print!("wifi psk: ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let wifi_psk = input.trim();

        let string = format!("{}:::{}\n\n", wifi_ssid, wifi_psk);
        serial.flush().expect("To be flushed");
        if let Err(err) = serial.write_all(string.as_bytes()) {
            println!("Error configuring device {}", err);
            exit(1);
        } else {
            println!("Device configured correctly");
        }
        serial.flush().expect("To be flushed");

        // let detected_chip = if before_operation != ResetBeforeOperation::NoResetNoSync {
        //     // Detect which chip we are connected to.
        //     let magic = connection.read_reg(CHIP_DETECT_MAGIC_REG_ADDR)?;
        //     let detected_chip = Chip::from_magic(magic)?;
        //     if let Some(chip) = chip {
        //         if chip != detected_chip {
        //             return Err(Error::ChipMismatch(
        //                 chip.to_string(),
        //                 detected_chip.to_string(),
        //             ));
        //         }
        //     }
        //     detected_chip
        // } else if before_operation == ResetBeforeOperation::NoResetNoSync && chip.is_some() {
        //     chip.unwrap()
        // } else {
        //     return Err(Error::ChipNotProvided);
        // };
    }
}
