use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    usize,
};

use crate::flasher::print_board_info;
use crate::serial::{
    detect_usb_serial_ports, get_port_handler, get_serial_ports_name, get_serialport_info,
    get_usbport_info,
};
use espflash::{
    cli::flash_elf_image,
    connection::reset::{ResetAfterOperation::HardReset, ResetBeforeOperation::DefaultReset},
    flasher::{FlashData, FlashSettings, Flasher, ProgressCallbacks},
    targets::Chip,
};
use indicatif::{ProgressBar, ProgressStyle};
use miette::{Error, IntoDiagnostic};

#[derive(Default)]
pub struct EspflashProgress {
    pb: Option<ProgressBar>,
}

impl ProgressCallbacks for EspflashProgress {
    /// Initialize the progress bar
    fn init(&mut self, addr: u32, len: usize) {
        let pb = ProgressBar::new(len as u64)
            .with_message(format!("{addr:#X}"))
            .with_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] [{bar:40}] {pos:>7}/{len:7} {msg}")
                    .unwrap()
                    .progress_chars("-> "),
            );

        self.pb = Some(pb);
    }

    /// Update the progress bar
    fn update(&mut self, current: usize) {
        if let Some(ref pb) = self.pb {
            pb.set_position(current as u64);
        }
    }

    /// End the progress bar
    fn finish(&mut self) {
        if let Some(ref pb) = self.pb {
            pb.finish();
        }
    }
}

pub struct Flash {
    args: Vec<String>,
}

impl Flash {
    pub fn new(args: Vec<String>) -> Flash {
        Flash { args }
    }
    pub fn exec(&self) -> Result<(), Error> {
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

        // override flash size in config provided
        // if let Some(flash_size) = config.flash.size {
        //     flasher.set_flash_size(flash_size);
        // }

        let chip = flasher.chip();
        let target = chip.into_target();
        let target_xtal_freq = target
            .crystal_freq(flasher.connection())
            .expect("Get crystal frequency");

        flasher.disable_watchdog()?;

        // Read the ELF data from the build path and load it to the target.
        let elf_data = fs::read("./self-esp").into_diagnostic()?;

        let flash_result = flash_elf_image(
            &mut flasher,
            &elf_data,
            FlashData::new(
                Some(Path::new("bootloader.bin")),
                Some(Path::new("partition-table.bin")),
                None,
                None,
                FlashSettings::default(),
                0,
            )
            .into_diagnostic()?,
            target_xtal_freq,
        );

        match flash_result {
            Ok(_) => println!("Flash finished!"),
            Err(_) => panic!("Cannot flash device"),
        }

        Ok(())
    }
}

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
