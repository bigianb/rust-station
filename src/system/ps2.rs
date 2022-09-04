

pub struct Ps2
{
    // 32Mb EE RAM
    pub ee_ram: Vec<u8>, 

    // 2Mb IOP RAM (also mapped to EE space)
    pub iop_ram: Vec<u8>, 

    // 4Mb ROM mapped to both EE and IOP
    pub rom: Vec<u8>
}

const EE_RAM_SIZE:  usize = 0x200_0000;
const IOP_RAM_SIZE: usize = 0x20_0000;
const ROM_SIZE:     usize = 0x40_0000;

const ROM_START_ADDR: u32 = 0x1FC0_0000;


impl Ps2
{
    /// Creates a new Ps2 object
    pub fn new(bios_data: &[u8]) -> Box<Ps2>
    {
        let sys = Box::new(Ps2 { ee_ram: vec!(0; EE_RAM_SIZE), iop_ram: vec!(0; IOP_RAM_SIZE), rom: vec!(0; ROM_SIZE) });
        // TODO: copy bios data to rom
        
        return sys;
    }

    /// Reads a 32 bit unsigned value from the EE memory. Slow but simple.
    pub fn read_ee_u32(&self, addr: u32) -> u32
    {
        let phys_addr = addr & 0x1FFFFFFF;
        if phys_addr >= ROM_START_ADDR{
            let rom_addr = (phys_addr - ROM_START_ADDR) as usize;
            let subref = self.rom[rom_addr..(rom_addr + 4)].try_into().unwrap();
            return u32::from_le_bytes(subref);
        }
        return 0xDEAD_BEEF;
    }
}
