use std::{
    io::{self, Write},
    time::Duration,
};

use miette::{Context, IntoDiagnostic, Result};
use serialport::{
    available_ports, FlowControl, SerialPort, SerialPortInfo, SerialPortType, TTYPort, UsbPortInfo,
};

pub fn serial_port_selector() -> String {
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

    port_names[selected_port].clone()
}

// Returns a vector with available USB serial ports.
#[cfg(not(all(target_os = "linux", target_env = "musl")))]
pub fn detect_usb_serial_ports(list_all_ports: bool) -> Result<Vec<SerialPortInfo>> {
    let ports = available_ports().into_diagnostic()?;
    let ports = ports
        .into_iter()
        .filter(|port_info| {
            if list_all_ports {
                matches!(
                    &port_info.port_type,
                    SerialPortType::UsbPort(..) |
                    // Allow PciPort. The user may want to use it.
                    // The port might have been misdetected by the system as PCI.
                    SerialPortType::PciPort |
                    // Good luck.
                    SerialPortType::Unknown
                )
            } else {
                matches!(&port_info.port_type, SerialPortType::UsbPort(..))
            }
        })
        .collect::<Vec<_>>();

    Ok(ports)
}

// Returns a list of port names from the given serial port info.
pub fn get_serial_ports_name(serialports: &Vec<SerialPortInfo>) -> Vec<String> {
    serialports
        .iter()
        .map(|port_info| port_info.port_name.clone())
        .collect::<Vec<_>>()
}

// Opens a serial port by name and baud rate.
pub fn get_port_handler(port_info: &SerialPortInfo) -> TTYPort {
    let tty_port_result = serialport::new(port_info.port_name.clone(), 115_200)
        .flow_control(FlowControl::None)
        .open_native()
        .map_err(espflash::error::Error::from)
        .wrap_err_with(|| format!("Failed to open serial port {}", port_info.port_name));

    match tty_port_result {
        Ok(tty_port) => tty_port,
        Err(_) => {
            panic!("Cannot get serial port");
        }
    }
}

/// Finds the `SerialPortInfo` for a given port name.
pub fn get_serialport_info(port_name: &str) -> Option<SerialPortInfo> {
    available_ports().ok().and_then(|ports| {
        ports
            .into_iter()
            .find(|port| port.port_name == port_name)
            .and_then(|port| Some(port))
    })
}

// Get the `UsbPortInfo` on a given SerialPortInfo
pub fn get_usbport_info(port: &SerialPortInfo) -> UsbPortInfo {
    match &port.port_type {
        SerialPortType::UsbPort(port) => port.clone(),
        _ => {
            panic!("selected a non usb port for serialport usage")
        }
    }
}
