use crate::system::r5900;

pub struct Ps2
{
    // 32Mb EE RAM
    pub ee_ram: Vec<u32>, 

    // 2Mb IOP RAM (also mapped to EE space)
    pub iop_ram: Vec<u32>, 

    // 4Mb ROM mapped to both EE and IOP
    pub rom: Vec<u32>,

    pub r5900: r5900::R5900State
}

const EE_RAM_SIZE:  usize = 0x200_0000;
const IOP_RAM_SIZE: usize = 0x20_0000;
const ROM_SIZE:     usize = 0x40_0000;

const ROM_START_ADDR: u32 = 0x1FC0_0000;


impl Ps2
{
    /// Creates a new Ps2 object
    pub fn new(bios_data: &[u32]) -> Box<Ps2>
    {
        let sys = Box::new(Ps2 { ee_ram: vec!(0; EE_RAM_SIZE/4), iop_ram: vec!(0; IOP_RAM_SIZE/4), rom: bios_data.to_vec(), r5900: r5900::R5900State::new() });
        return sys;
    }

    pub fn step(&mut self)
    {
        r5900::R5900::step(self);
    }

    /// Reads a 32 bit unsigned value from the EE memory. Slow but simple.
    pub fn read_ee_u32(&self, addr: u32) -> u32
    {
        let phys_addr = addr & 0x1FFFFFFF;
        if phys_addr >= ROM_START_ADDR{
            let rom_addr = (phys_addr - ROM_START_ADDR) as usize;
            return self.rom[rom_addr/4];
        }
        return 0xDEAD_BEEF;
    }

    pub fn read_ee_i8(&self, addr: u32) -> i8
    {
        let word_addr = addr & !3;
        let word_val = self.read_ee_u32(word_addr);
        
        (word_val >> ((addr & 3) * 8) & 0xFF) as i8
    }

    /// Writes a 32 bit unsigned value to the EE memory. Slow but simple.
    pub fn write_ee_u32(&mut self, addr: u32, value: u32)
    {
        let phys_addr = (addr & 0x1FFFFFFF) as usize;
        if phys_addr < EE_RAM_SIZE {
            self.ee_ram[phys_addr/4] = value;
        }
    }
}
