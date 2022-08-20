
use crate::system::ps2::Ps2;

use std::fs::File;
use std::io::Read;

pub mod system;

fn main() {
    
    let mut bios_file = File::open("bios/bios.bin").unwrap();
    let mut bios_data = Vec::new();
    bios_file.read_to_end(&mut bios_data).unwrap();

    let ps2 = Ps2::new(&bios_data);

}
