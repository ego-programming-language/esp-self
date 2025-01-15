use log::*;
use std::io::{BufRead, BufReader, ErrorKind};

pub fn configure() -> (String, String) {
    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin);

    println!("         (SELF-ESP FIRMWARE v0)");
    let mut wifi_ssid = "unknown".to_string();
    let mut wifi_psk = "unknown".to_string();

    loop {
        let mut line = String::new();

        // try read
        match reader.read_line(&mut line) {
            // EOF
            Ok(0) => {
                print!("SELF-ESP: write <WIFI_SSID>:::<WIFI_PSK> to serial port to continue\r");
            }
            // InputString
            Ok(_n) => {
                let trimmed = line.trim();
                // generate and exit command if needed
                // if trimmed.eq_ignore_ascii_case("exit") {
                //     info!("Saliendo...");
                //     break;
                // }

                let splitted: Vec<String> = trimmed
                    .split(":::")
                    .map(|string| string.to_string())
                    .collect();

                if splitted.len() >= 2 {
                    wifi_ssid = splitted[0].clone();
                    wifi_psk = splitted[1].clone();
                    break;
                }
            }
            // EAGAIN: not data for the moment
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    print!("SELF-ESP: write <WIFI_SSID>:::<WIFI_PSK> to serial port to continue\r");
                    std::thread::sleep(core::time::Duration::from_millis(200));
                    continue;
                } else {
                    // Good luck.
                    error!("Error de lectura: {:?}", e);
                    break;
                }
            }
        }

        std::thread::sleep(core::time::Duration::from_millis(500));
    }

    println!("Wifi SSID: {wifi_ssid}");
    println!("Wifi PASSWORD: {wifi_psk}");
    (wifi_ssid, wifi_psk)
}
