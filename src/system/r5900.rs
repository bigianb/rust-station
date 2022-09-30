use super::ps2::Ps2;

// allows us to quickly turn off tracing
macro_rules! trace {
    ($fmt:expr) => (print!($fmt));
    ($fmt:expr, $($arg:tt)*) => (print!($fmt, $($arg)*));
}

pub struct R5900State {
    pub pc: u32,

    pub gpr_regs: [[u32; 4]; 32],

    pub cop0_regs: [u32; 32]
}

const COP0_PRID: usize = 0x0f;

impl R5900State {
    pub fn new() -> R5900State {
        let mut it = R5900State { pc: 0xBFC0_0000, gpr_regs: [[0;4]; 32], cop0_regs: [0; 32] };
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
        OPCODE_HANDLERS[op_code](sys, instruction);
        trace!("\n");
    }

    fn op_special(sys: &mut Ps2, instruction: u32) {
        let function_no: usize = (instruction & 0x3f).try_into().unwrap();
        SPECIAL_HANDLERS[function_no](sys, instruction);
    }

    fn op_regimm(sys: &mut Ps2, instruction: u32) {}

    fn op_j(sys: &mut Ps2, instruction: u32) {}

    fn op_jal(sys: &mut Ps2, instruction: u32) {}

    fn op_beq(sys: &mut Ps2, instruction: u32) {
        trace!("BEQ");
        sys.r5900.pc += 4;
    }

    fn op_bne(sys: &mut Ps2, instruction: u32) {
        trace!("BNE");
        sys.r5900.pc += 4;
    }

    fn op_blez(sys: &mut Ps2, instruction: u32) {}

    fn op_bgtz(sys: &mut Ps2, instruction: u32) {}

    fn op_addi(sys: &mut Ps2, instruction: u32) {}

    fn op_addiu(sys: &mut Ps2, instruction: u32) {}

    fn op_slti(sys: &mut Ps2, instruction: u32) {
        trace!("SLTI");
        sys.r5900.pc += 4;
    }

    fn op_sltiu(sys: &mut Ps2, instruction: u32) {}

    fn op_andi(sys: &mut Ps2, instruction: u32) {}

    fn op_ori(sys: &mut Ps2, instruction: u32) {}

    fn op_xori(sys: &mut Ps2, instruction: u32) {}

    fn op_lui(sys: &mut Ps2, instruction: u32) {}

    fn op_cop0(sys: &mut Ps2, instruction: u32) {
        let rs = (instruction >> 21) & 0x1f;
        let rt = (instruction >> 16) & 0x1f;
        let rd = (instruction >> 11) & 0x1f;
        let function_no = instruction & 0x3f;
        match rs {
            0 => {
                match function_no {
                    0 => {
                        trace!("MFC0 {}, {}", MIPS_GPR_NAMES[(rt as usize)], COP0_REGNAMES[(rd as usize)]);
                    }
                    _ => {
                        trace!("MF0 - unknown function");
                    }
                }
                sys.r5900.pc += 4;
            }
            4 => {
                trace!("MT0");
            }
            8 => {
                trace!("BC0");
            }
            0x10 => {
                trace!("C0");
            }
            _ => (),
        }
    }

    fn op_cop1(sys: &mut Ps2, instruction: u32) {}

    fn op_cop2(sys: &mut Ps2, instruction: u32) {}

    fn op_illegal(sys: &mut Ps2, instruction: u32) {}

    fn op_beql(sys: &mut Ps2, instruction: u32) {}

    fn op_bnel(sys: &mut Ps2, instruction: u32) {}

    fn op_blezl(sys: &mut Ps2, instruction: u32) {}

    fn op_bgtzl(sys: &mut Ps2, instruction: u32) {}

    fn op_daddi(sys: &mut Ps2, instruction: u32) {}

    fn op_daddiu(sys: &mut Ps2, instruction: u32) {}

    fn op_ldl(sys: &mut Ps2, instruction: u32) {}

    fn op_ldr(sys: &mut Ps2, instruction: u32) {}

    fn op_special2(sys: &mut Ps2, instruction: u32) {}

    fn op_lb(sys: &mut Ps2, instruction: u32) {}

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

    fn op_sw(sys: &mut Ps2, instruction: u32) {}

    fn op_sdl(sys: &mut Ps2, instruction: u32) {}

    fn op_sdr(sys: &mut Ps2, instruction: u32) {}

    fn op_swr(sys: &mut Ps2, instruction: u32) {}

    fn op_cache(sys: &mut Ps2, instruction: u32) {}

    fn op_lwc1(sys: &mut Ps2, instruction: u32) {}

    fn op_pref(sys: &mut Ps2, instruction: u32) {}

    fn op_ldc2(sys: &mut Ps2, instruction: u32) {}

    fn op_ld(sys: &mut Ps2, instruction: u32) {}

    fn op_swc1(sys: &mut Ps2, instruction: u32) {}

    fn op_sdc2(sys: &mut Ps2, instruction: u32) {}

    fn op_sd(sys: &mut Ps2, instruction: u32) {}

    fn op_sll(sys: &mut Ps2, instruction: u32) {
        trace!("SLL");
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

    fn op_jr(sys: &mut Ps2, instruction: u32) {
        trace!("JR");
        sys.r5900.pc += 4;
    }

    fn op_jalr(sys: &mut Ps2, instruction: u32) {
        trace!("JALR");
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
        trace!("MFHI");
        sys.r5900.pc += 4;
    }

    fn op_mflo(sys: &mut Ps2, instruction: u32) {
        trace!("MFLO");
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
        trace!("MULT");
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
        trace!("DIVU");
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
        trace!("OR");
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
        trace!("DADDU");
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
}

const OPCODE_HANDLERS: [fn(&mut Ps2, u32); 0x40] = [
    /* 0x00 */ R5900::op_special,
    R5900::op_regimm,
    R5900::op_j,
    R5900::op_jal,
    R5900::op_beq,
    R5900::op_bne,
    R5900::op_blez,
    R5900::op_bgtz,
    /* 0x08 */ R5900::op_addi,
    R5900::op_addiu,
    R5900::op_slti,
    R5900::op_sltiu,
    R5900::op_andi,
    R5900::op_ori,
    R5900::op_xori,
    R5900::op_lui,
    /* 0x10 */ R5900::op_cop0,
    R5900::op_cop1,
    R5900::op_cop2,
    R5900::op_illegal,
    R5900::op_beql,
    R5900::op_bnel,
    R5900::op_blezl,
    R5900::op_bgtzl,
    /* 0x18 */ R5900::op_daddi,
    R5900::op_daddiu,
    R5900::op_ldl,
    R5900::op_ldr,
    R5900::op_special2,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    /* 0x20 */ R5900::op_lb,
    R5900::op_lh,
    R5900::op_lwl,
    R5900::op_lw,
    R5900::op_lbu,
    R5900::op_lhu,
    R5900::op_lwr,
    R5900::op_lwu,
    /* 0x28 */ R5900::op_sb,
    R5900::op_sh,
    R5900::op_swl,
    R5900::op_sw,
    R5900::op_sdl,
    R5900::op_sdr,
    R5900::op_swr,
    R5900::op_cache,
    /* 0x30 */ R5900::op_illegal,
    R5900::op_lwc1,
    R5900::op_illegal,
    R5900::op_pref,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_ldc2,
    R5900::op_ld,
    /* 0x38 */ R5900::op_illegal,
    R5900::op_swc1,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_illegal,
    R5900::op_sdc2,
    R5900::op_sd,
];

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
    "R0", "AT", "V0", "V1", "A0", "A1", "A2", "A3",
    "T0", "T1", "T2", "T3", "T4", "T5", "T6", "T7",
    "S0", "S1", "S2", "S3", "S4", "S5", "S6", "S7",
    "T8", "T9", "K0", "K1", "GP", "SP", "FP", "RA"
];
