use super::ps2::Ps2;

// allows us to quickly turn off tracing
macro_rules! trace {
    ($fmt:expr) => (print!($fmt));
    ($fmt:expr, $($arg:tt)*) => (print!($fmt, $($arg)*));
}

// trace to a minimum width. Always expects parameters.
macro_rules! tracew {
    ($width:expr, $fmt:expr, $($arg:tt)*) => {
        let str = format!($fmt, $($arg)*);
        let width = $width;
        print!("{:width$}", str);
    };
}

// trace the op-code disassembly portion into a minimum width area.
macro_rules! trace_opdis {
    ($fmt:expr) => (tracew!(30, $fmt));
    ($fmt:expr, $($arg:tt)*) => (tracew!(30, $fmt, $($arg)*));
}

pub struct R5900State {
    pub pc: u32,

    /* The address to branch to after a delay slot */
    pub branch_address: u32,

    /* The address of the current delay slot. If zero, we're not subject to a delay slot. */
    pub delay_slot_addr: u32,

    pub gpr_regs: [[u32; 4]; 32],

    pub fpr_regs: [f32; 32],

    pub cop0_regs: [u32; 32],

    pub lo: u32,
    pub hi: u32
}

const COP0_PRID: usize = 0x0f;

impl R5900State {
    pub fn new() -> R5900State {
        let mut it = R5900State { pc: 0xBFC0_0000, branch_address: 0, delay_slot_addr: 0, gpr_regs: [[0;4]; 32], fpr_regs: [0.0; 32], cop0_regs: [0; 32], lo: 0, hi: 0 };
        it.cop0_regs[COP0_PRID] = 0x00002e20;
        return it;
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
pub struct R5900 {}

impl R5900 {
    pub fn step(sys: &mut Ps2) {
        let instruction = sys.read_ee_u32(sys.r5900.pc);
        let op_code: usize = ((instruction >> 26) & 0x3f).try_into().unwrap();
        trace!(
            "{:#010X}:  {:#010X}    ",
            sys.r5900.pc, instruction
        );
        trace!("{:#04X} ", op_code);
        let in_branch_delay = sys.r5900.delay_slot_addr == sys.r5900.pc;
        Self::OPCODE_HANDLERS[op_code](sys, instruction);
        trace!("\n");
        if in_branch_delay {
            trace!("Branching\n");
            sys.r5900.pc = sys.r5900.branch_address;
            sys.r5900.delay_slot_addr = 0;
        }
    }

    fn op_special(sys: &mut Ps2, instruction: u32) {
        let function_no: usize = (instruction & 0x3f).try_into().unwrap();
        SPECIAL_HANDLERS[function_no](sys, instruction);
    }

    fn op_regimm(sys: &mut Ps2, instruction: u32) {
        let rt = ((instruction >> 16) & 0x1f) as usize;
        trace!("{}", rt);
        REGIMM_HANDLERS[rt](sys, instruction);
    }

    fn op_j(sys: &mut Ps2, instruction: u32) {
        trace!("J");
        sys.r5900.pc += 4;
    }

    fn op_jal(sys: &mut Ps2, instruction: u32) {
        let instr_index = instruction & 0x03FF_FFFF;
        let jump_addr = instr_index*4 | (sys.r5900.pc & 0xf000_0000);

        trace!("JAL {:#10X}", jump_addr);
        Self::set_gpr_unsigned(sys, 31, sys.r5900.pc + 8);
        Self::schedule_jump(sys, jump_addr);
        sys.r5900.pc += 4;
    }

    fn op_jalr(sys: &mut Ps2, instruction: u32) {
        let rs = ((instruction >> 21) & 0x1f) as usize;
        let rd = ((instruction >> 11) & 0x1f) as usize;
        trace!("JALR {}, {}", MIPS_GPR_NAMES[rd], MIPS_GPR_NAMES[rs]);

        Self::set_gpr_unsigned(sys, rd, sys.r5900.pc + 8);
        Self::schedule_jump(sys, sys.r5900.gpr_regs[rs][0]);
        sys.r5900.pc += 4;
    }
    
    fn op_jr(sys: &mut Ps2, instruction: u32) {
        let rs = ((instruction >> 21) & 0x1f) as usize;
        trace!("JR {}", MIPS_GPR_NAMES[rs]);

        Self::schedule_jump(sys, sys.r5900.gpr_regs[rs][0]);
        sys.r5900.pc += 4;
    }

    fn op_beq(sys: &mut Ps2, instruction: u32) {
        let rs = ((instruction >> 21) & 0x1f) as usize;
        let rt = ((instruction >> 16) & 0x1f) as usize;
        let offset = (instruction & 0xffff) as i16;

        trace!("BEQ {}, {}, {:#06X}", MIPS_GPR_NAMES[rs], MIPS_GPR_NAMES[rt], offset);

        if sys.r5900.gpr_regs[rs][0] == sys.r5900.gpr_regs[rt][0] {
            Self::schedule_branch(sys, offset);
        }
        sys.r5900.pc += 4;
    }

    fn schedule_branch(sys: &mut Ps2, offset: i16) {
        sys.r5900.branch_address = ((offset as i32)* 4 + sys.r5900.pc as i32 + 4) as u32;
        sys.r5900.delay_slot_addr = sys.r5900.pc+4;
    }

    fn schedule_jump(sys: &mut Ps2, addr: u32) {
        sys.r5900.branch_address = addr;
        sys.r5900.delay_slot_addr = sys.r5900.pc+4;
    }

    fn set_gpr_unsigned(sys: &mut Ps2, gpr: usize, value: u32) {
        sys.r5900.gpr_regs[gpr][0] = value;
        sys.r5900.gpr_regs[gpr][1] = 0; 
    }

    fn get_64_bit_reg(sys: &mut Ps2, gpr: usize) -> u64 {
        (sys.r5900.gpr_regs[gpr][1] as u64)  << 32 | sys.r5900.gpr_regs[gpr][0] as u64
    }

    fn set_64_bit_reg(sys: &mut Ps2, gpr: usize, value: u64) {
        sys.r5900.gpr_regs[gpr][0] = (value & 0xFFFFFFFF) as u32;
        sys.r5900.gpr_regs[gpr][1] = (value >> 32) as u32;
    }

    fn op_addi(sys: &mut Ps2, instruction: u32) {
        // TODO: should be checked and throw an exception
        trace!("ADDI delegating to ");
        Self::op_addiu(sys, instruction);
    }

    fn op_addiu(sys: &mut Ps2, instruction: u32) {
        let rs = ((instruction >> 21) & 0x1f) as usize;
        let rt = ((instruction >> 16) & 0x1f) as usize;
        let imm = (instruction & 0xFFFF) as i16;

        trace!("ADDIU {}, {}, {:#06X}", MIPS_GPR_NAMES[rt], MIPS_GPR_NAMES[rs], imm);

        let rsval = sys.r5900.gpr_regs[rs][0] as i32;
        sys.r5900.gpr_regs[rt][0] = (rsval + i32::from(imm)) as u32;
        sys.r5900.pc += 4;
    }

    fn op_slti(sys: &mut Ps2, instruction: u32) {
        let rs = ((instruction >> 21) & 0x1f) as usize;
        let rt = ((instruction >> 16) & 0x1f) as usize;
        let imm = (instruction & 0xffff) as i16;

        trace_opdis!("SLTI {}, {}, {:#06X}", MIPS_GPR_NAMES[rt], MIPS_GPR_NAMES[rs], imm);
        trace!("-> SLTI {:#06X}, {:#06X}", sys.r5900.gpr_regs[rs][0], imm);

        let signed_rs = sys.r5900.gpr_regs[rs][0] as i32;
        if signed_rs < imm.into() {
            sys.r5900.gpr_regs[rt][0] = 1;
        } else {
            sys.r5900.gpr_regs[rt][0] = 0;
        }

        sys.r5900.pc += 4;
    }

    fn op_sltiu(sys: &mut Ps2, instruction: u32) {
        let rs = ((instruction >> 21) & 0x1f) as usize;
        let rt = ((instruction >> 16) & 0x1f) as usize;
        let imm = (((instruction & 0xffff) as i16) as i32) as u32;

        trace_opdis!("SLTIU {}, {}, {:#06X}", MIPS_GPR_NAMES[rt], MIPS_GPR_NAMES[rs], imm);

        let rs_64 = Self::get_64_bit_reg(sys, rs);
        
        if rs_64 < imm.into() {
            sys.r5900.gpr_regs[rt][0] = 1;
        } else {
            sys.r5900.gpr_regs[rt][0] = 0;
        }
        sys.r5900.gpr_regs[rt][1] = 0;

        trace!("-> {:#X} < {:#X} -> {} = {}", rs_64, imm, MIPS_GPR_NAMES[rt], sys.r5900.gpr_regs[rt][0]);

        sys.r5900.pc += 4;
    }

    fn op_andi(sys: &mut Ps2, instruction: u32) {
        let rs = ((instruction >> 21) & 0x1f) as usize;
        let rt = ((instruction >> 16) & 0x1f) as usize;
        let imm = instruction & 0xFFFF;

        trace_opdis!("ANDI {}, {}, {:#06X}", MIPS_GPR_NAMES[rt], MIPS_GPR_NAMES[rs], imm);

        sys.r5900.gpr_regs[rt][0] = imm & sys.r5900.gpr_regs[rs][0];

        trace!("->  {} = {:#10X}", MIPS_GPR_NAMES[rt], sys.r5900.gpr_regs[rt][0]);
        sys.r5900.pc += 4;
    }

    fn op_ori(sys: &mut Ps2, instruction: u32) {
        let rs = ((instruction >> 21) & 0x1f) as usize;
        let rt = ((instruction >> 16) & 0x1f) as usize;
        let imm = instruction & 0xFFFF;

        trace_opdis!("ORI {}, {}, {:#06X}", MIPS_GPR_NAMES[rt], MIPS_GPR_NAMES[rs], imm);

        sys.r5900.gpr_regs[rt][0] = imm | sys.r5900.gpr_regs[rs][0];

        trace!("->  {} = {:#10X}", MIPS_GPR_NAMES[rt], sys.r5900.gpr_regs[rt][0]);
        sys.r5900.pc += 4;
    }

    fn op_xori(sys: &mut Ps2, instruction: u32) {}

    fn write_sign_extended_32_bit_reg(sys: &mut Ps2, gpr: usize, value: u32)
    {
        sys.r5900.gpr_regs[gpr][0] = value;
        if value & 0x80000000 == 0x80000000{
            sys.r5900.gpr_regs[gpr][1] = 0xFFFFFFFF;
        } else {
            sys.r5900.gpr_regs[gpr][1] = 0;
        }
    }

    fn op_lui(sys: &mut Ps2, instruction: u32) {
        let rt = ((instruction >> 16) & 0x1f) as usize;
        let imm = (instruction & 0xFFFF) << 16;

        trace!("LUI {}, {:#06X}", MIPS_GPR_NAMES[rt], instruction & 0xFFFF);

        Self::write_sign_extended_32_bit_reg(sys, rt, imm);
        sys.r5900.pc += 4;
    }

    fn op_cop0(sys: &mut Ps2, instruction: u32) {
        let rs = (instruction >> 21) & 0x1f;
        let rt = ((instruction >> 16) & 0x1f) as usize;
        let rd = ((instruction >> 11) & 0x1f) as usize;
        let function_no = instruction & 0x3f;
        match rs {
            0 => {
                match function_no {
                    0 => {
                        trace!("MFC0 {}, {}", MIPS_GPR_NAMES[rt], COP0_REGNAMES[rd]);
                        sys.r5900.gpr_regs[rt][0] = sys.r5900.cop0_regs[rd];
                    }
                    _ => {
                        trace!("MF0 - unknown function");
                    }
                }
                sys.r5900.pc += 4;
            }
            4 => {
                trace!("MT0");
                sys.r5900.pc += 4;
            }
            8 => {
                trace!("BC0");
            }
            0x10 => {
                trace!("C0");
                sys.r5900.pc += 4;
            }
            _ => (),
        }
    }

    fn op_cop1(sys: &mut Ps2, instruction: u32) {}

    fn op_cop2(sys: &mut Ps2, instruction: u32) {}

    fn op_illegal(sys: &mut Ps2, instruction: u32) {}

    fn op_beql(sys: &mut Ps2, instruction: u32) {
        let rs = ((instruction >> 21) & 0x1f) as usize;
        let rt = ((instruction >> 16) & 0x1f) as usize;
        let offset = (instruction & 0xFFFF) as i16;

        trace_opdis!("BEQL {}, {}, {:#06X}", MIPS_GPR_NAMES[rs], MIPS_GPR_NAMES[rt], offset);

        if sys.r5900.gpr_regs[rs][0] == sys.r5900.gpr_regs[rt][0] {
            Self::schedule_branch(sys, offset);
            sys.r5900.pc += 4;
        } else {
            trace!("-> not taken, skip delay slot");
            sys.r5900.pc += 8;
        }
    }

    fn op_bnel(sys: &mut Ps2, instruction: u32) {
        let rs = ((instruction >> 21) & 0x1f) as usize;
        let rt = ((instruction >> 16) & 0x1f) as usize;
        let offset = (instruction & 0xFFFF) as i16;

        trace_opdis!("BNEL {}, {}, {:#06X}", MIPS_GPR_NAMES[rs], MIPS_GPR_NAMES[rt], offset);

        if sys.r5900.gpr_regs[rs][0] != sys.r5900.gpr_regs[rt][0] {
            Self::schedule_branch(sys, offset);
            sys.r5900.pc += 4;
        } else {
            trace!("-> not taken, skip delay slot");
            sys.r5900.pc += 8;
        }
    }

    fn op_bne(sys: &mut Ps2, instruction: u32) {
        let rs = ((instruction >> 21) & 0x1f) as usize;
        let rt = ((instruction >> 16) & 0x1f) as usize;
        let offset = (instruction & 0xffff) as i16;

        trace!("BNE {}, {}, {:#06X}", MIPS_GPR_NAMES[rs], MIPS_GPR_NAMES[rt], offset);

        if sys.r5900.gpr_regs[rs][0] != sys.r5900.gpr_regs[rt][0] {
            Self::schedule_branch(sys, offset);
        }

        sys.r5900.pc += 4;
    }

    fn op_blez(sys: &mut Ps2, instruction: u32) {}

    fn op_bgtz(sys: &mut Ps2, instruction: u32) {}

    fn op_bltz(sys: &mut Ps2, instruction: u32) {
    }

    fn op_bgez(sys: &mut Ps2, instruction: u32) {
        let rs = ((instruction >> 21) & 0x1f) as usize;
        let offset = (instruction & 0xffff) as i16;

        trace_opdis!("BGEZ {}, {}", MIPS_GPR_NAMES[rs], offset);

        trace!("{} >= 0", sys.r5900.gpr_regs[rs][0] as i32);

        if sys.r5900.gpr_regs[rs][0] as i32 >= 0 {
            Self::schedule_branch(sys, offset);
        }

        sys.r5900.pc += 4;
    }

    fn op_blezl(sys: &mut Ps2, instruction: u32) {}

    fn op_bgtzl(sys: &mut Ps2, instruction: u32) {}

    fn op_daddi(sys: &mut Ps2, instruction: u32) {}

    fn op_daddiu(sys: &mut Ps2, instruction: u32) {}

    fn op_ldl(sys: &mut Ps2, instruction: u32) {}

    fn op_ldr(sys: &mut Ps2, instruction: u32) {}

    fn op_special2(sys: &mut Ps2, instruction: u32) {}

    fn op_lb(sys: &mut Ps2, instruction: u32) {
        let base = ((instruction >> 21) & 0x1f) as usize;
        let rt = ((instruction >> 16) & 0x1f) as usize;
        let offset = (instruction & 0xFFFF) as i16;

        trace_opdis!("LB {}, {:#06X}({})", MIPS_GPR_NAMES[rt], offset, MIPS_GPR_NAMES[base]);

        let addr = (sys.r5900.gpr_regs[base][0] as i32) + offset as i32;
        let bval = sys.read_ee_i8(addr as u32) as i32;
        sys.r5900.gpr_regs[rt][0] = bval as u32;
        if bval < 0{
            sys.r5900.gpr_regs[rt][1] = 0xFFFF_FFFF;
        } else {
            sys.r5900.gpr_regs[rt][1] = 0x0;
        }
        trace!("{} = {:#010X}_{:010X}", MIPS_GPR_NAMES[rt], sys.r5900.gpr_regs[rt][1], sys.r5900.gpr_regs[rt][0]);
        sys.r5900.pc += 4;
    }

    fn op_lh(sys: &mut Ps2, instruction: u32) {}

    fn op_lwl(sys: &mut Ps2, instruction: u32) {}

    fn op_lw(sys: &mut Ps2, instruction: u32) {}

    fn op_lbu(sys: &mut Ps2, instruction: u32) {}

    fn op_lhu(sys: &mut Ps2, instruction: u32) {}

    fn op_lwr(sys: &mut Ps2, instruction: u32) {}

    fn op_lwu(sys: &mut Ps2, instruction: u32) {}

    fn op_sb(sys: &mut Ps2, instruction: u32) {}

    fn op_sh(sys: &mut Ps2, instruction: u32) {}

    fn op_swl(sys: &mut Ps2, instruction: u32) {}

    fn op_sw(sys: &mut Ps2, instruction: u32) {
        let base = ((instruction >> 21) & 0x1f) as usize;
        let rt = ((instruction >> 16) & 0x1f) as usize;
        let imm = (instruction & 0xFFFF) as i16;

        trace_opdis!("SW {}, {:#06X}({})", MIPS_GPR_NAMES[rt], imm, MIPS_GPR_NAMES[base]);

        let addr = sys.r5900.gpr_regs[base][0] as i32 + i32::from(imm);
        sys.write_ee_u32(addr as u32, sys.r5900.gpr_regs[rt][0]);

        sys.r5900.pc += 4;
    }

    fn op_sdl(sys: &mut Ps2, instruction: u32) {}

    fn op_sdr(sys: &mut Ps2, instruction: u32) {}

    fn op_swr(sys: &mut Ps2, instruction: u32) {}

    fn op_cache(sys: &mut Ps2, instruction: u32) {}

    fn op_lwc1(sys: &mut Ps2, instruction: u32) {}

    fn op_pref(sys: &mut Ps2, instruction: u32) {}

    fn op_ldc2(sys: &mut Ps2, instruction: u32) {}

    fn op_ld(sys: &mut Ps2, instruction: u32) {}

    fn op_swc1(sys: &mut Ps2, instruction: u32) {
        let base = ((instruction >> 21) & 0x1f) as usize;
        let ft = ((instruction >> 16) & 0x1f) as usize;
        let offset = (instruction & 0xFFFF) as i16;

        trace_opdis!("SWC1 {}, {:#06X}({})", MIPS_FPR_NAMES[ft], offset, MIPS_GPR_NAMES[base]);

        let addr = sys.r5900.gpr_regs[base][0] as i32 + i32::from(offset);
        sys.write_ee_u32(addr as u32, sys.r5900.fpr_regs[ft] as u32);

        sys.r5900.pc += 4;
    }

    fn op_sdc2(sys: &mut Ps2, instruction: u32) {}

    fn op_sd(sys: &mut Ps2, instruction: u32) {
        let base = ((instruction >> 21) & 0x1f) as usize;
        let rt = ((instruction >> 16) & 0x1f) as usize;
        let imm = (instruction & 0xFFFF) as i16;

        trace!("SD {}, {:#06X}({})", MIPS_GPR_NAMES[rt], imm, MIPS_GPR_NAMES[base]);

        let addr = (sys.r5900.gpr_regs[rt][0] as i32 + i32::from(imm)) as u32;
        sys.write_ee_u32(addr, sys.r5900.gpr_regs[rt][0]);
        sys.write_ee_u32(addr+4, sys.r5900.gpr_regs[rt][1]);

        sys.r5900.pc += 4;
    }

    fn op_sll(sys: &mut Ps2, instruction: u32) {
        if instruction == 0 {
            trace!("NOP");
        } else {
            let rt = ((instruction >> 16) & 0x1f) as usize;
            let rd = ((instruction >> 11) & 0x1f) as usize;
            let sa = ((instruction >> 6) & 0x1f) as usize;
            trace_opdis!("SLL {}, {}, {}", MIPS_GPR_NAMES[rd], MIPS_GPR_NAMES[rt], sa);

            let rt_val = sys.r5900.gpr_regs[rt][0] << sa;
            Self::write_sign_extended_32_bit_reg(sys, rd, rt_val);

            trace!("{} = {:#010X}", MIPS_GPR_NAMES[rd], rt_val);
        }
        sys.r5900.pc += 4;
    }

    fn op_srl(sys: &mut Ps2, instruction: u32) {
        trace!("SRL");
        sys.r5900.pc += 4;
    }

    fn op_sra(sys: &mut Ps2, instruction: u32) {
        trace!("SRA");
        sys.r5900.pc += 4;
    }

    fn op_sllv(sys: &mut Ps2, instruction: u32) {
        trace!("SLLV");
        sys.r5900.pc += 4;
    }

    fn op_srlv(sys: &mut Ps2, instruction: u32) {
        trace!("SRLV");
        sys.r5900.pc += 4;
    }

    fn op_srav(sys: &mut Ps2, instruction: u32) {
        trace!("SRAV");
        sys.r5900.pc += 4;
    }

    fn op_movz(sys: &mut Ps2, instruction: u32) {
        trace!("MOVZ");
        sys.r5900.pc += 4;
    }

    fn op_movn(sys: &mut Ps2, instruction: u32) {
        trace!("MOVN");
        sys.r5900.pc += 4;
    }

    fn op_syscall(sys: &mut Ps2, instruction: u32) {
        trace!("SYSCALL");
        sys.r5900.pc += 4;
    }

    fn op_break(sys: &mut Ps2, instruction: u32) {
        trace!("BREAK");
        sys.r5900.pc += 4;
    }

    fn op_sync(sys: &mut Ps2, instruction: u32) {
        trace!("SYNC");
        sys.r5900.pc += 4;
    }

    fn op_mfhi(sys: &mut Ps2, instruction: u32) {
        let rd = ((instruction >> 11) & 0x1f) as usize;
        trace!("MFHI {}", MIPS_GPR_NAMES[rd]);
        sys.r5900.gpr_regs[rd][0] = sys.r5900.hi;
        sys.r5900.pc += 4;
    }

    fn op_mflo(sys: &mut Ps2, instruction: u32) {
        let rd = ((instruction >> 11) & 0x1f) as usize;
        trace!("MFLO {}", MIPS_GPR_NAMES[rd]);
        sys.r5900.gpr_regs[rd][0] = sys.r5900.lo;
        sys.r5900.pc += 4;
    }

    fn op_mthi(sys: &mut Ps2, instruction: u32) {
        trace!("MTHI");
        sys.r5900.pc += 4;
    }

    fn op_mtlo(sys: &mut Ps2, instruction: u32) {
        trace!("MTLO");
        sys.r5900.pc += 4;
    }

    fn op_dsllv(sys: &mut Ps2, instruction: u32) {
        trace!("DSLLV");
        sys.r5900.pc += 4;
    }

    fn op_dsrlv(sys: &mut Ps2, instruction: u32) {
        trace!("DSRLV");
        sys.r5900.pc += 4;
    }

    fn op_dsrav(sys: &mut Ps2, instruction: u32) {
        trace!("DSRAV");
        sys.r5900.pc += 4;
    }

    fn op_mult(sys: &mut Ps2, instruction: u32) {
        let rs = ((instruction >> 21) & 0x1f) as usize;
        let rt = ((instruction >> 16) & 0x1f) as usize;
        trace_opdis!("MULT {}, {}", MIPS_GPR_NAMES[rs], MIPS_GPR_NAMES[rt]);

        let rs_val = sys.r5900.gpr_regs[rs][0] as i32;
        let rt_val = sys.r5900.gpr_regs[rt][0] as i32;

        let result: i64 = (rs_val as i64) * (rt_val as i64);
        sys.r5900.lo = result as u32;
        sys.r5900.hi = (result >> 32) as u32;

        trace!("->  HI={:#X}, LO={:#X}", sys.r5900.hi, sys.r5900.lo);

        sys.r5900.pc += 4;
    }

    fn op_multu(sys: &mut Ps2, instruction: u32) {
        trace!("MULTU");
        sys.r5900.pc += 4;
    }

    fn op_div(sys: &mut Ps2, instruction: u32) {
        trace!("DIV");
        sys.r5900.pc += 4;
    }

    fn op_divu(sys: &mut Ps2, instruction: u32) {
        let rs = ((instruction >> 21) & 0x1f) as usize;
        let rt = ((instruction >> 16) & 0x1f) as usize;
        trace_opdis!("DIVU {}, {}", MIPS_GPR_NAMES[rs], MIPS_GPR_NAMES[rt]);

        let rs_val = sys.r5900.gpr_regs[rs][0];
        let rt_val = sys.r5900.gpr_regs[rt][0];

        if rt_val != 0{
            sys.r5900.lo = rs_val / rt_val;
            sys.r5900.hi = rs_val % rt_val;
        } else {
            trace!(" ** DIVIDE BY ZERO ***");
        }
        trace!("->  HI={:#X}, LO={:#X}", sys.r5900.hi, sys.r5900.lo);
        sys.r5900.pc += 4;
    }

    fn op_add(sys: &mut Ps2, instruction: u32) {
        trace!("ADD");
        sys.r5900.pc += 4;
    }

    fn op_addu(sys: &mut Ps2, instruction: u32) {
        trace!("ADDU");
        sys.r5900.pc += 4;
    }

    fn op_sub(sys: &mut Ps2, instruction: u32) {
        trace!("SUB");
        sys.r5900.pc += 4;
    }

    fn op_subu(sys: &mut Ps2, instruction: u32) {
        trace!("SUBU");
        sys.r5900.pc += 4;
    }

    fn op_and(sys: &mut Ps2, instruction: u32) {
        trace!("AND");
        sys.r5900.pc += 4;
    }

    fn op_or(sys: &mut Ps2, instruction: u32) {
        let rs = ((instruction >> 21) & 0x1f) as usize;
        let rt = ((instruction >> 16) & 0x1f) as usize;
        let rd = ((instruction >> 11) & 0x1f) as usize;

        trace_opdis!("OR {}, {}, {}", MIPS_GPR_NAMES[rd], MIPS_GPR_NAMES[rs], MIPS_GPR_NAMES[rt]);

        sys.r5900.gpr_regs[rd][0] = sys.r5900.gpr_regs[rs][0] | sys.r5900.gpr_regs[rt][0];
        sys.r5900.gpr_regs[rd][1] = sys.r5900.gpr_regs[rs][1] | sys.r5900.gpr_regs[rt][1];

        trace!("->  {} = {:#010X}_{:010X}", MIPS_GPR_NAMES[rd], sys.r5900.gpr_regs[rd][1], sys.r5900.gpr_regs[rd][0]);
        sys.r5900.pc += 4;
        sys.r5900.pc += 4;
    }

    fn op_xor(sys: &mut Ps2, instruction: u32) {
        trace!("XOR");
        sys.r5900.pc += 4;
    }

    fn op_nor(sys: &mut Ps2, instruction: u32) {
        trace!("NOR");
        sys.r5900.pc += 4;
    }

    fn op_mfsa(sys: &mut Ps2, instruction: u32) {
        trace!("MFSA");
        sys.r5900.pc += 4;
    }

    fn op_mtsa(sys: &mut Ps2, instruction: u32) {
        trace!("MTSA");
        sys.r5900.pc += 4;
    }

    fn op_slt(sys: &mut Ps2, instruction: u32) {
        trace!("SLT");
        sys.r5900.pc += 4;
    }

    fn op_sltu(sys: &mut Ps2, instruction: u32) {
        trace!("SLTU");
        sys.r5900.pc += 4;
    }

    fn op_dadd(sys: &mut Ps2, instruction: u32) {
        trace!("DADD");
        sys.r5900.pc += 4;
    }

    fn op_daddu(sys: &mut Ps2, instruction: u32) {
        let rs = ((instruction >> 21) & 0x1f) as usize;
        let rt = ((instruction >> 16) & 0x1f) as usize;
        let rd = ((instruction >> 11) & 0x1f) as usize;
        trace_opdis!("DADDU {}, {}, {}", MIPS_GPR_NAMES[rd], MIPS_GPR_NAMES[rs], MIPS_GPR_NAMES[rt]);
        if rs == 0 {
            // DADDU x, zero, y is used as an assignment
            sys.r5900.gpr_regs[rd][0] = sys.r5900.gpr_regs[rt][0];
            sys.r5900.gpr_regs[rd][1] = sys.r5900.gpr_regs[rt][1];
        } else {
            let sval = Self::get_64_bit_reg(sys, rs);
            let tval = Self::get_64_bit_reg(sys, rt);
            let total = sval + tval;
            Self::set_64_bit_reg(sys, rd, total);
        }
        trace!("{} = {:#X}", MIPS_GPR_NAMES[rd], Self::get_64_bit_reg(sys, rd));
        sys.r5900.pc += 4;
    }

    fn op_dsub(sys: &mut Ps2, instruction: u32) {
        trace!("DSUB");
        sys.r5900.pc += 4;
    }

    fn op_dsubu(sys: &mut Ps2, instruction: u32) {
        trace!("DSUBU");
        sys.r5900.pc += 4;
    }

    fn op_tge(sys: &mut Ps2, instruction: u32) {
        trace!("TGE");
        sys.r5900.pc += 4;
    }

    fn op_tgeu(sys: &mut Ps2, instruction: u32) {
        trace!("TGEU");
        sys.r5900.pc += 4;
    }

    fn op_tlt(sys: &mut Ps2, instruction: u32) {
        trace!("TLT");
        sys.r5900.pc += 4;
    }

    fn op_tltu(sys: &mut Ps2, instruction: u32) {
        trace!("TLTU");
        sys.r5900.pc += 4;
    }

    fn op_teq(sys: &mut Ps2, instruction: u32) {
        trace!("TEQ");
        sys.r5900.pc += 4;
    }

    fn op_tne(sys: &mut Ps2, instruction: u32) {
        trace!("TNE");
        sys.r5900.pc += 4;
    }

    fn op_dsll(sys: &mut Ps2, instruction: u32) {
        trace!("DSLL");
        sys.r5900.pc += 4;
    }

    fn op_dsrl(sys: &mut Ps2, instruction: u32) {
        trace!("DSRL");
        sys.r5900.pc += 4;
    }

    fn op_dsra(sys: &mut Ps2, instruction: u32) {
        trace!("DSRA");
        sys.r5900.pc += 4;
    }

    fn op_dsll32(sys: &mut Ps2, instruction: u32) {
        trace!("DSLL32");
        sys.r5900.pc += 4;
    }

    fn op_dsrl32(sys: &mut Ps2, instruction: u32) {
        trace!("DSRL32");
        sys.r5900.pc += 4;
    }

    fn op_dsra32(sys: &mut Ps2, instruction: u32) {
        trace!("DSRA32");
        sys.r5900.pc += 4;
    }

    const OPCODE_HANDLERS: [fn(&mut Ps2, u32); 0x40] = [
    /* 0x00 */ Self::op_special,
    Self::op_regimm,
    Self::op_j,
    Self::op_jal,
    Self::op_beq,
    Self::op_bne,
    Self::op_blez,
    Self::op_bgtz,
    /* 0x08 */ Self::op_addi,
    Self::op_addiu,
    Self::op_slti,
    Self::op_sltiu,
    Self::op_andi,
    Self::op_ori,
    Self::op_xori,
    Self::op_lui,
    /* 0x10 */ Self::op_cop0,
    Self::op_cop1,
    Self::op_cop2,
    Self::op_illegal,
    Self::op_beql,
    Self::op_bnel,
    Self::op_blezl,
    Self::op_bgtzl,
    /* 0x18 */ Self::op_daddi,
    Self::op_daddiu,
    Self::op_ldl,
    Self::op_ldr,
    Self::op_special2,
    Self::op_illegal,
    Self::op_illegal,
    Self::op_illegal,
    /* 0x20 */ Self::op_lb,
    Self::op_lh,
    Self::op_lwl,
    Self::op_lw,
    Self::op_lbu,
    Self::op_lhu,
    Self::op_lwr,
    Self::op_lwu,
    /* 0x28 */ Self::op_sb,
    Self::op_sh,
    Self::op_swl,
    Self::op_sw,
    Self::op_sdl,
    Self::op_sdr,
    Self::op_swr,
    Self::op_cache,
    /* 0x30 */ Self::op_illegal,
    Self::op_lwc1,
    Self::op_illegal,
    Self::op_pref,
    Self::op_illegal,
    Self::op_illegal,
    Self::op_ldc2,
    Self::op_ld,
    /* 0x38 */ Self::op_illegal,
    Self::op_swc1,
    Self::op_illegal,
    Self::op_illegal,
    Self::op_illegal,
    Self::op_illegal,
    Self::op_sdc2,
    Self::op_sd,
];

}



const SPECIAL_HANDLERS: [fn(&mut Ps2, u32); 0x40] = [
    /* 0x00 */ R5900::op_sll,
    R5900::op_illegal,
    R5900::op_srl,
    R5900::op_sra,
    R5900::op_sllv,
    R5900::op_illegal,
    R5900::op_srlv,
    R5900::op_srav,
    /* 0x08 */ R5900::op_jr,
    R5900::op_jalr,
    R5900::op_movz,
    R5900::op_movn,
    R5900::op_syscall,
    R5900::op_break,
    R5900::op_illegal,
    R5900::op_sync,
    /* 0x10 */ R5900::op_mfhi,
    R5900::op_mthi,
    R5900::op_mflo,
    R5900::op_mtlo,
    R5900::op_dsllv,
    R5900::op_illegal,
    R5900::op_dsrlv,
    R5900::op_dsrav,
    /* 0x18 */ R5900::op_mult,
    R5900::op_multu,
    R5900::op_div,
    R5900::op_divu,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    /* 0x20 */ R5900::op_add,
    R5900::op_addu,
    R5900::op_sub,
    R5900::op_subu,
    R5900::op_and,
    R5900::op_or,
    R5900::op_xor,
    R5900::op_nor,
    /* 0x28 */ R5900::op_mfsa,
    R5900::op_mtsa,
    R5900::op_slt,
    R5900::op_sltu,
    R5900::op_dadd,
    R5900::op_daddu,
    R5900::op_dsub,
    R5900::op_dsubu,
    /* 0x30 */ R5900::op_tge,
    R5900::op_tgeu,
    R5900::op_tlt,
    R5900::op_tltu,
    R5900::op_teq,
    R5900::op_illegal,
    R5900::op_tne,
    R5900::op_illegal,
    /* 0x38 */ R5900::op_dsll,
    R5900::op_illegal,
    R5900::op_dsrl,
    R5900::op_dsra,
    R5900::op_dsll32,
    R5900::op_illegal,
    R5900::op_dsrl32,
    R5900::op_dsra32,
];

const REGIMM_HANDLERS: [fn(&mut Ps2, u32); 0x20] = [
    /* 0x00 */ R5900::op_bltz,
    R5900::op_bgez,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    /* 0x08 */ R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    /* 0x10 */ R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    /* 0x18 */ R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal
];

const COP0_REGNAMES: [&str; 32] = 
[
	"Index",
	"Random",
	"EntryLo0",
	"EntryLo1",
	"Context",
	"PageMask",
	"Wired",
	"RESERVED",
	"BadVAddr",
	"Count",
	"EntryHi",
	"Compare",
	"Status",
	"Cause",
	"EPC",
	"PRId",
	"Config",
	"RESERVED",
	"RESERVED",
	"RESERVED",
	"RESERVED",
	"RESERVED",
	"RESERVED",
	"BadPAddr",
	"Debug",
	"Perf",
	"RESERVED",
	"RESERVED",
	"TagLo",
	"TagHi",
	"ErrorEPC",
	"RESERVED"
];

const MIPS_GPR_NAMES: [&str; 32] = 
[
    "Zero", "AT", "V0", "V1", "A0", "A1", "A2", "A3",
    "T0", "T1", "T2", "T3", "T4", "T5", "T6", "T7",
    "S0", "S1", "S2", "S3", "S4", "S5", "S6", "S7",
    "T8", "T9", "K0", "K1", "GP", "SP", "FP", "RA"
];

const MIPS_FPR_NAMES: [&str; 32] = 
[
    "F0", "F1", "F2", "F3", "F4", "F5", "F6", "F7",
    "F8", "F9", "F10", "F11", "F12", "F13", "F14", "F15",
    "F16", "F17", "F18", "F19", "F20", "F21", "F22", "F23",
    "F24", "F25", "F26", "F27", "F28", "F29", "F30", "F31"
];