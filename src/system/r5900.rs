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

    fn op_special(&self)
    {

    }

}

const OPCODE_HANDLERS: [fn(&R5900); 1] = [
    R5900::op_special
];

/* 
CMA_MIPSIV::InstructionFuncConstant CMA_MIPSIV::m_cOpGeneral[MAX_GENERAL_OPS] =
{
	//0x00
	&CMA_MIPSIV::SPECIAL,		&CMA_MIPSIV::REGIMM,		&CMA_MIPSIV::J,				&CMA_MIPSIV::JAL,			&CMA_MIPSIV::BEQ,			&CMA_MIPSIV::BNE,			&CMA_MIPSIV::BLEZ,			&CMA_MIPSIV::BGTZ,
	//0x08
	&CMA_MIPSIV::ADDI,			&CMA_MIPSIV::ADDIU,			&CMA_MIPSIV::SLTI,			&CMA_MIPSIV::SLTIU,			&CMA_MIPSIV::ANDI,			&CMA_MIPSIV::ORI,			&CMA_MIPSIV::XORI,			&CMA_MIPSIV::LUI,
	//0x10
	&CMA_MIPSIV::COP0,			&CMA_MIPSIV::COP1,			&CMA_MIPSIV::COP2,			&CMA_MIPSIV::Illegal,		&CMA_MIPSIV::BEQL,			&CMA_MIPSIV::BNEL,			&CMA_MIPSIV::BLEZL,			&CMA_MIPSIV::BGTZL,
	//0x18
	&CMA_MIPSIV::DADDI,			&CMA_MIPSIV::DADDIU,		&CMA_MIPSIV::LDL,			&CMA_MIPSIV::LDR,			&CMA_MIPSIV::SPECIAL2,		&CMA_MIPSIV::Illegal,		&CMA_MIPSIV::Illegal,		&CMA_MIPSIV::Illegal,
	//0x20
	&CMA_MIPSIV::LB,			&CMA_MIPSIV::LH,			&CMA_MIPSIV::LWL,			&CMA_MIPSIV::LW,			&CMA_MIPSIV::LBU,			&CMA_MIPSIV::LHU,			&CMA_MIPSIV::LWR,			&CMA_MIPSIV::LWU,
	//0x28
	&CMA_MIPSIV::SB,			&CMA_MIPSIV::SH,			&CMA_MIPSIV::SWL,			&CMA_MIPSIV::SW,			&CMA_MIPSIV::SDL,			&CMA_MIPSIV::SDR,			&CMA_MIPSIV::SWR,			&CMA_MIPSIV::CACHE,
	//0x30
	&CMA_MIPSIV::Illegal,		&CMA_MIPSIV::LWC1,			&CMA_MIPSIV::Illegal,		&CMA_MIPSIV::PREF,	    	&CMA_MIPSIV::Illegal,		&CMA_MIPSIV::Illegal,		&CMA_MIPSIV::LDC2,			&CMA_MIPSIV::LD,
	//0x38
	&CMA_MIPSIV::Illegal,		&CMA_MIPSIV::SWC1,			&CMA_MIPSIV::Illegal,		&CMA_MIPSIV::Illegal,		&CMA_MIPSIV::Illegal,		&CMA_MIPSIV::Illegal,		&CMA_MIPSIV::SDC2,			&CMA_MIPSIV::SD,
};
*/