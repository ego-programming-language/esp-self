use std::{fs, path::Path};

use esp_idf_part::PartitionTable;
use espflash::{error::Error, flasher::Flasher};
use miette::{IntoDiagnostic, Result};

pub fn print_board_info(flasher: &mut Flasher) -> Result<()> {
    let info = flasher.device_info().expect("cannot get device info");

    print!("Chip type:         {}", info.chip);
    if let Some((major, minor)) = info.revision {
        println!(" (revision v{major}.{minor})");
    } else {
        println!();
    }
    println!("Crystal frequency: {}", info.crystal_frequency);
    println!("Flash size:        {}", info.flash_size);
    println!("Features:          {}", info.features.join(", "));
    println!("MAC address:       {}", info.mac_address);

    Ok(())
}

pub fn parse_partition_table(path: &Path) -> Result<PartitionTable, Error> {
    let data = fs::read(path)
        .into_diagnostic()
        .expect("Cannot get partition table file");

    Ok(PartitionTable::try_from(data).expect("possible"))
}
