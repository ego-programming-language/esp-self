use espflash::flasher::Flasher;
use miette::Result;

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
