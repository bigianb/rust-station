use super::ps2::Ps2;


pub struct R5900State
{
    pub pc: u32
}

impl R5900State
{
    pub fn new() -> R5900State
    {
        R5900State {
            pc: 0xBFC0_0000
        }
    }
}

/*
    The R5900 processor is not encapsulated from the system, instead the state is part of the
    Ps2 system and the whole system state is passed in.
    This is a bit ugly but the issue is that when executing R5900 instructions we can mutate the state of the 
    Ps2 as a whole in addition to the registers in the R5900. We end up passing around both the R5900 state
    and the Ps2 state (both mutable) and if the Ps2 state contains the R5900 state then we end up borrowing
    a mutable reference twice.
    My solution then, as a Rust noob, is to consider the system state as a single mutable entity.

    The (theoretical) downside here is that we can't easily drop the R5900 into another emulated system. 
*/
pub struct R5900
{

}

impl R5900
{

    pub fn step(sys: &mut Ps2)
    {
        let instruction = sys.read_ee_u32(sys.r5900.pc);
        let op_code: usize = ((instruction >>26) & 0x3f).try_into().unwrap();
        println!("instruction 0x{:X}:  0x{:X}, opcode = 0x{:X}", sys.r5900.pc, instruction, op_code);
        OPCODE_HANDLERS[op_code](sys, instruction);
    }

    fn op_special(sys: &mut Ps2, instruction: u32)
    {
        let function_no: usize = (instruction  & 0x3f).try_into().unwrap();
        println!("SPECIAL 0x{:X}", function_no);
        SPECIAL_HANDLERS[function_no](sys, instruction);
    }

    fn op_regimm(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_j(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_jal(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_beq(sys: &mut Ps2, instruction: u32)
    {
        println!("BEQ");
        sys.r5900.pc += 4;
    }

    fn op_bne(sys: &mut Ps2, instruction: u32)
    {
        println!("BNE");
        sys.r5900.pc += 4;
    }

    fn op_blez(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_bgtz(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_addi(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_addiu(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_slti(sys: &mut Ps2, instruction: u32)
    {
        println!("SLTI");
        sys.r5900.pc += 4;
    }

    fn op_sltiu(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_andi(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_ori(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_xori(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_lui(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_cop0(sys: &mut Ps2, instruction: u32)
    {
        let rs = (instruction >>21) & 0x1f;
        match rs {
            0 => {
                println!("MF0");
                sys.r5900.pc += 4;
            }
            4 => {
                println!("MT0");
            }
            8 => {
                println!("BC0");
            }
            0x10 => {
                println!("C0");
            }
            _ => ()
        }
    }

    fn op_cop1(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_cop2(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_illegal(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_beql(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_bnel(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_blezl(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_bgtzl(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_daddi(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_daddiu(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_ldl(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_ldr(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_special2(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_lb(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_lh(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_lwl(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_lw(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_lbu(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_lhu(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_lwr(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_lwu(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_sb(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_sh(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_swl(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_sw(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_sdl(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_sdr(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_swr(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_cache(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_lwc1(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_pref(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_ldc2(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_ld(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_swc1(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_sdc2(sys: &mut Ps2, instruction: u32)
    {
    }

    fn op_sd(sys: &mut Ps2, instruction: u32)
    {
    }

    
    fn op_sll(sys: &mut Ps2, instruction: u32)
    {
        println!("SLL");
        sys.r5900.pc += 4;
    }

    fn op_srl(sys: &mut Ps2, instruction: u32)
    {
        println!("SRL");
        sys.r5900.pc += 4;
    }

    fn op_sra(sys: &mut Ps2, instruction: u32)
    {
        println!("SRA");
        sys.r5900.pc += 4;
    }

    fn op_sllv(sys: &mut Ps2, instruction: u32)
    {
        println!("SLLV");
        sys.r5900.pc += 4;
    }

    fn op_srlv(sys: &mut Ps2, instruction: u32)
    {
        println!("SRLV");
        sys.r5900.pc += 4;
    }

    fn op_srav(sys: &mut Ps2, instruction: u32)
    {
        println!("SRAV");
        sys.r5900.pc += 4;
    }

}

const OPCODE_HANDLERS: [fn(&mut Ps2, u32); 0x40] = [
    /* 0x00 */ R5900::op_special, R5900::op_regimm, R5900::op_j,       R5900::op_jal,     R5900::op_beq,      R5900::op_bne,     R5900::op_blez,    R5900::op_bgtz,
    /* 0x08 */ R5900::op_addi,    R5900::op_addiu,  R5900::op_slti,    R5900::op_sltiu,   R5900::op_andi,     R5900::op_ori,     R5900::op_xori,    R5900::op_lui,
    /* 0x10 */ R5900::op_cop0,    R5900::op_cop1,   R5900::op_cop2,    R5900::op_illegal, R5900::op_beql,     R5900::op_bnel,    R5900::op_blezl,   R5900::op_bgtzl,
    /* 0x18 */ R5900::op_daddi,   R5900::op_daddiu, R5900::op_ldl,     R5900::op_ldr,     R5900::op_special2, R5900::op_illegal, R5900::op_illegal, R5900::op_illegal,
    /* 0x20 */ R5900::op_lb,      R5900::op_lh,     R5900::op_lwl,     R5900::op_lw,      R5900::op_lbu,      R5900::op_lhu,     R5900::op_lwr,     R5900::op_lwu, 
    /* 0x28 */ R5900::op_sb,      R5900::op_sh,     R5900::op_swl,     R5900::op_sw,      R5900::op_sdl,      R5900::op_sdr,     R5900::op_swr,     R5900::op_cache, 
    /* 0x30 */ R5900::op_illegal, R5900::op_lwc1,   R5900::op_illegal, R5900::op_pref,    R5900::op_illegal,  R5900::op_illegal, R5900::op_ldc2,    R5900::op_ld, 
    /* 0x38 */ R5900::op_illegal, R5900::op_swc1,   R5900::op_illegal, R5900::op_illegal, R5900::op_illegal,  R5900::op_illegal, R5900::op_sdc2,    R5900::op_sd, 
];

const SPECIAL_HANDLERS: [fn(&mut Ps2, u32); 0x40] = [
    /* 0x00 */ R5900::op_sll, R5900::op_illegal, R5900::op_srl,       R5900::op_sra,     R5900::op_sllv,      R5900::op_illegal,     R5900::op_srlv,    R5900::op_srav,

    /* 0x08 */ R5900::op_illegal, R5900::op_illegal,   R5900::op_illegal, R5900::op_illegal, R5900::op_illegal,  R5900::op_illegal, R5900::op_illegal,    R5900::op_illegal, 
    /* 0x10 */ R5900::op_illegal, R5900::op_illegal,   R5900::op_illegal, R5900::op_illegal, R5900::op_illegal,  R5900::op_illegal, R5900::op_illegal,    R5900::op_illegal, 
    /* 0x18 */ R5900::op_illegal, R5900::op_illegal,   R5900::op_illegal, R5900::op_illegal, R5900::op_illegal,  R5900::op_illegal, R5900::op_illegal,    R5900::op_illegal, 
    /* 0x20 */ R5900::op_illegal, R5900::op_illegal,   R5900::op_illegal, R5900::op_illegal, R5900::op_illegal,  R5900::op_illegal, R5900::op_illegal,    R5900::op_illegal, 
    /* 0x28 */ R5900::op_illegal, R5900::op_illegal,   R5900::op_illegal, R5900::op_illegal, R5900::op_illegal,  R5900::op_illegal, R5900::op_illegal,    R5900::op_illegal, 
    /* 0x30 */ R5900::op_illegal, R5900::op_illegal,   R5900::op_illegal, R5900::op_illegal, R5900::op_illegal,  R5900::op_illegal, R5900::op_illegal,    R5900::op_illegal, 
    /* 0x38 */ R5900::op_illegal, R5900::op_illegal,   R5900::op_illegal, R5900::op_illegal, R5900::op_illegal,  R5900::op_illegal, R5900::op_illegal,    R5900::op_illegal, 
];