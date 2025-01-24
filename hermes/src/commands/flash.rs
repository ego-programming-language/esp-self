use std::io;
use std::{fs, io::Write};
use std::{process::exit, usize};

use crate::serial::{
    detect_usb_serial_ports, get_port_handler, get_serial_ports_name, get_serialport_info,
    get_usbport_info, serial_port_selector,
};
use crate::{core::temp_file::TempFile, flasher::print_board_info};
use espflash::{
    cli::flash_elf_image,
    connection::reset::{ResetAfterOperation::HardReset, ResetBeforeOperation::DefaultReset},
    flasher::{FlashData, FlashSettings, Flasher, ProgressCallbacks},
    targets::Chip,
};
use indicatif::{ProgressBar, ProgressStyle};
use miette::{Error, IntoDiagnostic};
use tempfile::{Builder, NamedTempFile};

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

const ELF_DATA: &[u8] = include_bytes!("../../self-esp");
const BOOTLOADER_BIN: &[u8] = include_bytes!("../../bootloader.bin");
const PARTITION_TABLE_CSV: &[u8] = include_bytes!("../../partition-table.csv");

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
        let port_name = serial_port_selector();

        /////// GET SERIALPORT
        ///////
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

        // create temp file for reading firmware files (elf, partition table and bootloader)
        let elf = Flash::initialize_firmware_files("program", ".elf", ELF_DATA);
        let bootloader = Flash::initialize_firmware_files("bootloader", ".bin", BOOTLOADER_BIN);
        let partition_table =
            Flash::initialize_firmware_files("partable", ".csv", PARTITION_TABLE_CSV);

        let flash_data = FlashData::new(
            Some(bootloader.path),
            Some(partition_table.path),
            None,
            None,
            FlashSettings::default(),
            0,
        )
        .expect("error generating flash data");

        let elf_data = fs::read(elf.path).into_diagnostic()?;
        let flash_result = flash_elf_image(&mut flasher, &elf_data, flash_data, target_xtal_freq);

        match flash_result {
            Ok(_) => println!("Flash finished!"),
            Err(err) => panic!("Cannot flash device {}", err),
        }

        Ok(())
    }

    fn initialize_firmware_files(filename: &str, extension: &str, data: &[u8]) -> TempFile {
        let mut temp_file = match Builder::new()
            .prefix(filename)
            .suffix(extension)
            .rand_bytes(5)
            .tempfile()
        {
            Ok(v) => v,
            Err(err) => {
                eprintln!("Error initializing temporal files {}", err);
                exit(1)
            }
        };

        if let Err(err) = temp_file.write_all(data) {
            eprintln!("Error writting temporal files: {}", err);
            exit(1);
        }

        // Box::leak to convert the PathBuf to 'static lifetime.
        let path = temp_file.path().to_path_buf();
        let static_path = Box::leak(Box::new(path)).as_path();

        // Maintain file in scope
        TempFile {
            _file: temp_file,
            path: static_path,
        }
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
