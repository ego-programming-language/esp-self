// fn main() {
//     // It is necessary to call this function once. Otherwise some patches to the runtime
//     // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
//     esp_idf_svc::sys::link_patches();

//     // Bind the log crate to the ESP Logging facilities
//     esp_idf_svc::log::EspLogger::initialize_default();

//     // do your stuff here
//     log::info!("Hello, ego!");
//     // print "HELLO" bytecode
//     let mut vm = self_vm::new(vec![1, 5, 3, 5, 0, 0, 0, 72, 69, 76, 76, 79, 2, 1, 0, 0, 0]);
//     vm.run(&vec![]);
// }

mod lib;

use anyhow::Result;
use core::str;
use embedded_svc::{http::Method, io::Write};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        i2c::{I2cConfig, I2cDriver},
        prelude::*,
    },
    http::server::{Configuration, EspHttpServer},
    io::{EspIOError, Read},
};
use lib::wifi::wifi;
use shtcx::{self, shtc3, PowerMode};
use std::result::Result::Ok;
use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

struct Config {
    wifi_ssid: &'static str,
    wifi_psk: &'static str,
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    let app_config = Config {
        wifi_ssid: "Sercomm08C0",
        wifi_psk: "GE6HH747Z9QBQ3",
    };

    // Connect to the Wi-Fi network
    let _wifi = wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
    )?;

    // `EspHttpServer` instance using a default configuration
    let mut server = EspHttpServer::new(&Configuration::default())?;

    server.fn_handler(
        "/",
        Method::Get,
        |request| -> core::result::Result<(), EspIOError> {
            println!("request from: {:?}", request.header("user-agent"));
            let html = index_html();
            let mut response = request.into_ok_response()?;
            response.write_all(html.as_bytes())?;
            Ok(())
        },
    )?;

    server.fn_handler(
        "/",
        Method::Post,
        |mut request| -> core::result::Result<(), EspIOError> {
            println!(
                "request from: {:?}",
                request.header("user-agent").unwrap_or("Unknown")
            );
            let (_headers, connection) = request.split();

            const MAX_BODY_SIZE: usize = 4096; // 4KB
            let mut buffer: Vec<u8> = Vec::with_capacity(MAX_BODY_SIZE);
            let mut temp_buffer = [0u8; 1024];

            loop {
                let bytes_read = connection.read(&mut temp_buffer)?;

                if bytes_read == 0 {
                    break;
                }

                // exceed MAX_BODY_SIZE
                if buffer.len() + bytes_read > MAX_BODY_SIZE {
                    println!("Exceed the body size");
                    let mut response = request.into_status_response(413)?;
                    response.write_all(b"Payload Too Large")?;
                    return Ok(());
                }

                buffer.extend_from_slice(&temp_buffer[..bytes_read]);
            }

            let body_str = match std::str::from_utf8(&buffer) {
                Ok(s) => s,
                Err(_) => {
                    println!("Cuerpo de la solicitud no es UTF-8 válido");
                    let mut response = request.into_status_response(400)?;
                    response.write_all(b"Invalid UTF-8 in body")?;
                    return Ok(());
                }
            };

            println!("request body: {:#?}", body_str);

            let html = templated("post /");
            let mut response = request.into_ok_response()?;

            response.write_all(html.as_bytes())?;
            Ok(())
        },
    )?;

    println!("Server awaiting connection");

    loop {
        sleep(Duration::from_millis(1000));
    }
}

fn templated(content: impl AsRef<str>) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8">
        <title>esp-rs web server</title>
    </head>
    <body>
        {}
    </body>
</html>
"#,
        content.as_ref()
    )
}

fn index_html() -> String {
    templated("Hello from mcu!")
}
