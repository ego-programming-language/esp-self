use std::{
    alloc::System,
    fs::File,
    io::{self, Read},
    path::Path,
    process::exit,
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

        // Establish a connection to the device using the default baud rate of 115,200
        // and timeout of 3 seconds.
        let mut connection = Connection::new(
            serialport,
            usbport_info,
            ResetAfterOperation::HardReset,
            ResetBeforeOperation::DefaultReset,
        );

        if let Err(err) = connection.begin() {
            println!("err: {}", err)
        }
        connection.set_timeout(CONNECTION_TIMEOUT).expect("to wait");

        let mut serial = connection.into_serial();

        let mut buf = [0u8; 1024];

        loop {
            match serial.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let data = &buf[..n];
                    match std::str::from_utf8(data) {
                        Ok(text) => print!("{}", text),
                        Err(_) => {
                            println!("{:?}", data);
                        }
                    }
                }
                Ok(_) => {} // no data
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                    // Tiempo de espera agotado, continuar
                    continue;
                }
                Err(e) => {
                    eprintln!("Error leyendo del puerto serial: {}", e);
                    break;
                }
            }
        }
    }
}
