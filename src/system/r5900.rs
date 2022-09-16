use super::ps2::Ps2;



pub struct R5900
{
    pub pc: u32
}

impl R5900
{
    pub fn new() -> R5900
    {
        R5900 {
            pc: 0xBFC0_0000
        }
    }

    pub fn step(&self, sys: &Ps2)
    {
        let instruction = sys.read_ee_u32(self.pc);

        println!("instruction 0x{:X}:  0x{:X}", self.pc, instruction);
    }
}
