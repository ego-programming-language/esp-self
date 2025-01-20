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

mod configurator;
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
use lib::{storage::KeyValStore, wifi::wifi};
use shtcx::{self, shtc3, PowerMode};
use std::result::Result::Ok;
use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

struct Config {
    wifi_ssid: String,
    wifi_psk: String,
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let namespace = "wifi-config";
    let mut key_value_store = KeyValStore::new(namespace);
    let (wifi_ssid, wifi_psk): (String, String) = match key_value_store.get("ssid") {
        Some(v) => {
            // GET WIFI CREDENTIALS
            // from the KeyValueStore (nvs)
            let psk = key_value_store.get("psk");
            if psk.is_none() {
                panic!("Cannot get psk from the nvs")
            }
            let psk = String::from_utf8(psk.unwrap()).expect("To get a string");
            let wifi = String::from_utf8(v).expect("To get a string");

            (wifi, psk)
        }
        None => {
            // CONFIGURATING FIRMWARE
            // get wifi credentials from serialport and then
            // save it to the KeyValueStore (nvs)
            let (wifi_ssid, wifi_psk) = configurator::configure();
            let ssid_result = key_value_store.set("ssid", wifi_ssid.as_bytes());
            let psk_result = key_value_store.set("psk", wifi_psk.as_bytes());
            if ssid_result.is_err() | psk_result.is_err() {
                panic!("Cannot set ssid or psk to nvs")
            }

            (wifi_ssid, wifi_psk)
        }
    };

    // STARTING HTTP SERVER
    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    let app_config = Config {
        wifi_ssid,
        wifi_psk,
    };

    // Connect to the Wi-Fi network
    let _wifi = wifi(
        app_config.wifi_ssid.as_str(),
        app_config.wifi_psk.as_str(),
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
            if request.header("content-type").unwrap_or("Unknown") != "application/octet-stream" {
                println!("Bad request, content-type not \"application/octect-stream\"");
                let mut response = request.into_status_response(400)?;
                response
                    .write_all(b"Bad request, content-type not \"application/octect-stream\"")?;
                return Ok(());
            }

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

            println!("uploaded bytecode: {:#?}", buffer);
            let mut vm = self_vm::new(buffer);
            vm.run(&vec!["-d".to_string()]);

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
