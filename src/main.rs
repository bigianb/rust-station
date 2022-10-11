
use crate::system::ps2::Ps2;

use std::fs::File;
use std::io::Read;

pub mod system;

fn main() {
    
    let mut bios_file = File::open("bios/bios.bin").unwrap();
    let mut bios_data = Vec::new();
    bios_file.read_to_end(&mut bios_data).unwrap();

    // convert the bios data to words from bytes
    let bios_u32_data = unsafe {
        // Ensure the original vector is not dropped.
        let mut v_clone = std::mem::ManuallyDrop::new(bios_data);
        Vec::from_raw_parts(v_clone.as_mut_ptr() as *mut u32,
                            v_clone.len() / 4,
                            v_clone.capacity() / 4)
    };

    let mut ps2 = Ps2::new(&bios_u32_data);

    for _i in 0 .. 60 {
        ps2.step();
    }
    
 //   let mut val = ps2.read_ee_u32(0xBFC0_0000);
 //   println!("0x{:X}", val);

 //   val = ps2.read_ee_u32(0x0FC0_0004);

 //   println!("0x{:X}", val);
}
