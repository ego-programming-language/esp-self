use std::time::Duration;

use miette::{IntoDiagnostic, Result};
use serialport::{available_ports, SerialPort, SerialPortInfo, SerialPortType, UsbPortInfo};

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
pub fn get_port_handler(port_name: &String, baud_rate: u32) -> Box<dyn SerialPort> {
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
            panic!("Failed to open serial port: {}", e);
        }
    }

    port
}

/// Finds the `UsbPortInfo` for a given port name.
pub fn get_usbport_info(port_name: &str) -> Option<UsbPortInfo> {
    available_ports().ok().and_then(|ports| {
        ports
            .into_iter()
            .find(|port| port.port_name == port_name)
            .and_then(|port| {
                if let SerialPortType::UsbPort(usb_info) = port.port_type {
                    Some(usb_info)
                } else {
                    None
                }
            })
    })
}
