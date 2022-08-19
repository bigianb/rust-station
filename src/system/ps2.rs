
pub struct Ps2
{
    // 32Mb EE RAM
    pub ee_ram: Vec<u8>, 

    // 2Mb IOP RAM (also mapped to EE space)
    pub iop_ram: Vec<u8>, 

    // 4Mb ROM mapped to both EE and IOP
    pub rom: Vec<u8>, 
}

const EE_RAM_SIZE:  usize = 0x2000000;
const IOP_RAM_SIZE: usize = 0x200000;
const ROM_SIZE:     usize = 0x400000;

impl Ps2
{
    // use vec!
    pub fn new() -> Ps2
    {
        Ps2 { ee_ram: vec!(0; EE_RAM_SIZE), iop_ram: vec!(0; IOP_RAM_SIZE), rom: vec!(0; ROM_SIZE) }
    }
}
