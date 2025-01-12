use serialport::{SerialPortInfo, SerialPortType};

mod serial;
// use espflash::{self, flasher::Flasher, targets::Chip};

fn main() {
    println!("self-esp toolkit");

    println!(">>>> select port <<<<");
    let ports = serial::detect_usb_serial_ports(true);
    let mut port_names = vec![];
    match ports {
        Ok(ports) => {
            port_names = ports
                .iter()
                .map(|port_info| {
                    let name = port_info.port_name.as_str();
                    match &port_info.port_type {
                        SerialPortType::UsbPort(info) => {
                            if let Some(product) = &info.product {
                                format!("{} - {}", name, product)
                            } else {
                                name.to_string()
                            }
                        }
                        _ => name.to_string(),
                    }
                })
                .collect::<Vec<_>>();
        }
        Err(_) => {
            panic!("Error getting serial ports")
        }
    }

    println!("{:#?}", port_names);
    //serial::get_serial_port_info(, config);
    //espflash::connection::Connection::new(, port_info);
}
