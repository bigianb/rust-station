
use crate::system::ps2::Ps2;

use std::fs::File;
use std::io::Read;

pub mod system;

fn main() {
    
    let mut bios_file = File::open("bios/bios.bin").unwrap();
    let mut bios_data = Vec::new();
    bios_file.read_to_end(&mut bios_data).unwrap();

    let mut ps2 = Ps2::new(&bios_data);

    ps2.step();
    ps2.step();
    ps2.step();
    ps2.step();
    ps2.step();

 //   let mut val = ps2.read_ee_u32(0xBFC0_0000);
 //   println!("0x{:X}", val);

 //   val = ps2.read_ee_u32(0x0FC0_0004);

 //   println!("0x{:X}", val);
}
