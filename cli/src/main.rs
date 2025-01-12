mod serial;
// use espflash::{self, flasher::Flasher, targets::Chip};

fn main() {
    println!("Hello, cli!");
    let ports = serial::detect_usb_serial_ports(true);
    println!("{:#?}", ports);
    //serial::get_serial_port_info(, config);
    //espflash::connection::Connection::new(, port_info);
}
