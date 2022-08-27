
pub struct Ps2
{
    // 32Mb EE RAM
    pub ee_ram: Vec<u8>, 

    // 2Mb IOP RAM (also mapped to EE space)
    pub iop_ram: Vec<u8>, 

    // 4Mb ROM mapped to both EE and IOP
    pub rom: [u8; ROM_SIZE], 
}

const EE_RAM_SIZE:  usize = 0x2000000;
const IOP_RAM_SIZE: usize = 0x200000;
const ROM_SIZE:     usize = 0x400000;

const ROM_START_ADDR: u32 = 0x1FC00000;


impl Ps2
{
    /// Creates a new Ps2 object
    pub fn new(bios_data: &[u8]) -> Ps2
    {
        let sys = Ps2 { ee_ram: vec!(0; EE_RAM_SIZE), iop_ram: vec!(0; IOP_RAM_SIZE), rom: [0; ROM_SIZE] };
        // TODO: copy bios data to rom
        
        return sys;
    }

    /// Reads a 32 bit unsigned value from the EE memory. Slow but simple.
    pub fn read_ee_u32(&self, addr: u32) -> u32
    {
        let phys_addr = addr & 0x1FFFFFFF;
        if phys_addr >= ROM_START_ADDR{
            let rom_addr = phys_addr -ROM_START_ADDR;
            let num = self.rom[rom_addr];

            return num;
        }
        return 0xDEADBEEF;
    }
}
