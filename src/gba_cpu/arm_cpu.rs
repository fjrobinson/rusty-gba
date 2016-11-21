//use std::mem; // Needed if useing transmute
use self::ARM7Mode::*;

use std::fmt;
use gba_cpu::RType;
use gba_cpu::register::Register;

// Important PSR bits from:
// http://www.atmel.com/Images/DDI0029G_7TDMI_R3_trm.pdf
// section 2.7, page 2-13
// Condition code flag bits (28-31)
const COND_MASK: RType = 0xF0000000;
const N_MASK: RType = 0x80000000; // Negative or less than (31)
const Z_MASK: RType = 0x40000000; // Zero (30)
const C_MASK: RType = 0x20000000; // Carry or borrow or extend (29)
const V_MASK: RType = 0x10000000; // Overflow (28)
// Control bits (5-7)
const I_MASK: RType = 0x80; // IRQ Disable (7)
const F_MASK: RType = 0x40; // FRQ Disable (6)
const T_MASK: RType = 0x20; // Thumb State (5)
const M_MASK: RType = 0x1F; // Mode State (4-0)

// PSR mode bits from:
// http://www.atmel.com/Images/DDI0029G_7TDMI_R3_trm.pdf
// section 2.7.2, page 2-15
// Valid mode states
const USER_MODE: RType = 0b10000;
const FIQ_MODE:  RType = 0b10001;
const IRQ_MODE:  RType = 0b10010;
const SV_MODE:   RType = 0b10011; // Supervisor mode
const ABRT_MODE: RType = 0b10111; // Abort mode
const UDEF_MODE: RType = 0b11011; // Undefined instruction mode (Apply reset!)
const SYS_MODE:  RType = 0b11111; // System mode

// Register indices (Not verified after reg 15)
// TODO: Find real register names if important
pub const R0:       i8 = 0;
pub const R1:       i8 = 1;
pub const R2:       i8 = 2;
pub const R3:       i8 = 3;
pub const R4:       i8 = 4;
pub const R5:       i8 = 5;
pub const R6:       i8 = 6;
pub const R7:       i8 = 7;
pub const R8:       i8 = 8;
pub const R9:       i8 = 9;
pub const R10:      i8 = 10;
pub const R11:      i8 = 11;
pub const R12:      i8 = 12;
pub const R13:      i8 = 13;
pub const R14:      i8 = 14;
pub const R15:      i8 = 15;
pub const R8_FIQ:   i8 = 16;
pub const R9_FIQ:   i8 = 17;
pub const R10_FIQ:  i8 = 18;
pub const R11_FIQ:  i8 = 19;
pub const R12_FIQ:  i8 = 20;
pub const R13_FIQ:  i8 = 21;
pub const R14_FIQ:  i8 = 22;
pub const R13_SV:   i8 = 23;
pub const R14_SV:   i8 = 24;
pub const R13_ABT:  i8 = 25;
pub const R14_ABT:  i8 = 26;
pub const R13_IRQ:  i8 = 27;
pub const R14_IRQ:  i8 = 28;
pub const R13_UND:  i8 = 29;
pub const R14_UND:  i8 = 30;
pub const NUM_REGS: usize = 31;

// Saved status register indices
pub const SPSR_FIQ: i8 = 0;
pub const SPSR_SV:  i8 = 1;
pub const SPSR_ABT: i8 = 2;
pub const SPSR_IRQ: i8 = 3;
pub const SPSR_UND: i8 = 4;
pub const NUM_STATUS_REGS: usize = 6;

// Register alias
pub const SP:   i8 = R13;
pub const LINK: i8 = R14;
pub const PC:   i8 = R15;

// Modes of execution for ARM7TDMI
// TODO: Consider creating a typed state machine if performance is an issue: SEE
// BOTTOM OF THIS FILE
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ARM7Mode {
    User       = USER_MODE as isize,
    FIQ        = FIQ_MODE  as isize,
    IRQ        = IRQ_MODE  as isize,
    Supervisor = SV_MODE   as isize,
    Abort      = ABRT_MODE as isize,
    Undefined  = UDEF_MODE as isize,
    System     = SYS_MODE  as isize,
}

impl fmt::Display for ARM7Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mode_word = match *self {
            User       => "User",
            FIQ        => "FIQ",
            IRQ        => "IRQ",
            Supervisor => "Supervisor",
            Abort      => "Abort",
            Undefined  => "Undefined",
            System     => "System",
        };
        write![f, "{}", mode_word]
    }
}

// Registers from:
// http://www.atmel.com/Images/DDI0029G_7TDMI_R3_trm.pdf
// section 2.6, page 2-8
#[allow(missing_copy_implementations)]
pub struct ARM7 {
    regs: [Register; NUM_REGS],
    cpsr: Register,
    spsr: [Register; NUM_STATUS_REGS],
}

impl Default for ARM7 {
    fn default() -> ARM7 {
        let mut cpu = ARM7 {
            regs: [Register::default(); NUM_REGS],
            cpsr: Register::default(),
            spsr: [Register::default(); NUM_STATUS_REGS],
        };

        cpu.set_mode(FIQ);
        cpu.set_irq_disable();
        cpu
    }
}

// Implementation of ARM7TDMI
impl ARM7 {
    fn reg_map_index(&self, reg_num: i8) -> Option<i8> {
        assert!(reg_num >= R0);
        assert!(reg_num <= R15);

        if !self.is_thumb() {
            if reg_num <= R7 || reg_num == PC {
                Some(reg_num)
            }
            else {
                match self.mode() {
                    User | System => Some(reg_num),
                    FIQ => Some(reg_num + R8_FIQ - R8),
                    _ if reg_num <= R12 => Some(reg_num),
                    _ => Some(match self.mode() {
                        IRQ => reg_num + R13_IRQ,
                        Supervisor => reg_num + R13_SV,
                        Abort => reg_num + R13_ABT,
                        Undefined => reg_num + R13_UND,
                        _ => unreachable!(),
                    } - R13),
                }
            }
        }
        else {
            if reg_num <= R7 {
                Some(reg_num)
            }
            else {
                None
            }
        }
    }

    fn unmapped_reg_op<F>(&mut self, reg_num: i8, op: F)
        where F: Fn(&mut Register) {
        op(&mut self.regs[reg_num as usize])
    }

    pub fn reg_op<F>(&mut self, reg_num: i8, op: F)
        where F: Fn(&mut Register) {
        match self.reg_map_index(reg_num) {
            Some(reg) => self.unmapped_reg_op(reg, op),
            None => unreachable!(),
        }
    }

    fn reg_raw(&self, reg_num: i8) -> &Register {
        &self.regs[reg_num as usize]
    }

    fn reg_raw_mut(&mut self, reg_num: i8) -> &mut Register {
        &mut self.regs[reg_num as usize]
    }

    pub fn reg(&self, reg_num: i8) -> Option<&Register> {
        match self.reg_map_index(reg_num) {
            Some(x) => Some(self.reg_raw(x)),
            None => None,
        }
    }

    pub fn reg_mut(&mut self, reg_num: i8) -> Option<&mut Register> {
        match self.reg_map_index(reg_num) {
            Some(x) => Some(self.reg_raw_mut(x)),
            None => None,
        }
    }

    // PC register
    pub fn pc(&self) -> RType {
        self.reg_raw(PC).read()
    }

    pub fn inc_pc(&mut self) {
        let pc_val = self.reg_raw(PC).read();
        if self.is_thumb() {
            self.reg_raw_mut(PC).write((pc_val + 0b10) & 0xFFFFFFFE);
        }
        else {
            self.reg_raw_mut(PC).write((pc_val + 0b100) & 0xFFFFFFFC)
        }
    }

    pub fn set_pc(&mut self, pc_val: RType) {
        self.reg_raw_mut(PC).write(pc_val);
    }

    // CPSR Register access
    // TODO: Do we need mutators for this?
    pub fn cpsr(&self) -> &Register {
        &self.cpsr
    }

    pub fn spsr(&self) -> Option<&Register> {
        match self.mode() {
            User       => None,
            FIQ        => Some(&self.spsr[SPSR_FIQ as usize]),
            IRQ        => Some(&self.spsr[SPSR_IRQ as usize]),
            Supervisor => Some(&self.spsr[SPSR_SV  as usize]),
            Abort      => Some(&self.spsr[SPSR_ABT as usize]),
            Undefined  => Some(&self.spsr[SPSR_UND as usize]),
            System     => None,
        }
    }

    // Negative or less than
    pub fn is_neg_lt(&self) -> bool { self.cpsr.read_masked(N_MASK) != 0 }
    pub fn set_neg_lt(&mut self)    { self.cpsr.set(N_MASK, N_MASK); }
    pub fn reset_neg_lt(&mut self)  { self.cpsr.reset(N_MASK, N_MASK); }

    // Zero
    pub fn is_zero(&self) -> bool { self.cpsr.read_masked(Z_MASK) != 0 }
    pub fn set_zero(&mut self)    { self.cpsr.set(Z_MASK, Z_MASK); }
    pub fn reset_zero(&mut self)  { self.cpsr.reset(Z_MASK, Z_MASK); }

    // Carry, borrow, or extend
    pub fn is_carry(&self) -> bool { self.cpsr.read_masked(C_MASK) != 0 }
    pub fn set_carry(&mut self)    { self.cpsr.set(C_MASK, C_MASK); }
    pub fn reset_carry(&mut self)  { self.cpsr.reset(C_MASK, C_MASK); }

    // Overflow
    pub fn is_overflow(&self) -> bool { self.cpsr.read_masked(V_MASK) != 0 }
    pub fn set_overflow(&mut self)    { self.cpsr.set(V_MASK, V_MASK); }
    pub fn reset_overflow(&mut self)  { self.cpsr.reset(V_MASK, V_MASK); }

    // Reset condition bits
    pub fn reset_cond(&mut self) { self.cpsr.reset(COND_MASK, COND_MASK); }

    // IRQ disable
    pub fn is_irq_disable(&self) -> bool { self.cpsr.read_masked(I_MASK) != 0 }
    pub fn set_irq_disable(&mut self)    { self.cpsr.set(I_MASK, I_MASK); }
    pub fn reset_irq_disable(&mut self)  { self.cpsr.reset(I_MASK, I_MASK); }

    // FRQ disable
    pub fn is_fiq_disable(&self) -> bool { self.cpsr.read_masked(F_MASK) != 0 }
    pub fn set_fiq_disable(&mut self)    { self.cpsr.set(F_MASK, F_MASK); }
    pub fn reset_fiq_disable(&mut self)  { self.cpsr.reset(F_MASK, F_MASK); }

    // Thumb mode
    pub fn is_thumb(&self) -> bool { self.cpsr.read_masked(T_MASK) != 0 }
    pub fn set_thumb(&mut self)    { self.cpsr.set(T_MASK, T_MASK); }
    pub fn reset_thumb(&mut self)  { self.cpsr.reset(T_MASK, T_MASK); }

    pub fn mode(&self) -> ARM7Mode {
        match self.cpsr.read_masked(M_MASK) {
            USER_MODE => User,
            FIQ_MODE  => FIQ,
            IRQ_MODE  => IRQ,
            SV_MODE   => Supervisor,
            ABRT_MODE => Abort,
            UDEF_MODE => Undefined,
            SYS_MODE  => System,
            _ => unreachable!(),
        }
    }

    pub fn set_mode(&mut self, new_mode: ARM7Mode) {
        self.cpsr.set(M_MASK, new_mode as u32)
    }
}

impl fmt::Debug for ARM7 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for ARM7 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write![f, "ARM7TDMI:\n"]?;
        for i in 0..R15 {
            let reg_idx = self.reg_map_index(i).unwrap_or(-1);
            let alt_reg = if reg_idx > PC || reg_idx < R0 { "*" } else { "" };
            let reg_val = *self.reg(i).unwrap_or(&Register::default());
            write![f, "\tR{:02}[{:2}]:\t{}({:p}){}\n",
                   i, reg_idx, reg_val, reg_val, alt_reg]?;
        }
        write![f, "\tR{:02}[{:2}]:\t{}({:#010x})\t(PC)\n",
               PC + 1, self.reg_map_index(PC).unwrap_or(-1),
               self.pc(), self.pc()]?;

        write![f, "\tCPSR:\t{:#032b}\n", self.cpsr()]?;

        //write![f, "ARM7TDMI State:\n"]?;
        write![f, "\tMode:        {}\n", self.mode()]?;
        write![f, "\tThumb Mode:  {}\n", self.is_thumb()]?;
        write![f, "\tIRQ Disable: {}\n", self.is_irq_disable()]?;
        write![f, "\tFIQ Disable: {}\n", self.is_fiq_disable()]?;
        write![f, "\tNeg/Less:    {}\n", self.is_neg_lt()]?;
        write![f, "\tZero:        {}\n", self.is_zero()]?;
        write![f, "\tCarry:       {}\n", self.is_carry()]?;
        write![f, "\tOverflow:    {}\n", self.is_overflow()]
    }
}

// Register availability map based on mode in ARM state from:
// http://www.atmel.com/Images/DDI0029G_7TDMI_R3_trm.pdf
// section 2.6.1, page 2-9
// trait CPU {
//     fn gp_reg_op<F>(&mut self, reg_num: i8, op: F) where F: Fn(&mut Register);
//     fn gp_reg(&self, reg_num: i8) -> &Register;
//     fn gp_reg_mut(&mut self, reg_num: i8) -> &mut Register;
//     fn pc(&self) -> RType;

//     fn cpsr(&self) -> &Register;
//     fn cpsr_mut(&mut self) -> &mut Register;

//     fn spsr(&self) -> Option<&Register>;
//     fn spsr_mut(&self) -> Option<&mut Register>;

//     // Negative or less than
//     fn is_neg_lt(&self) -> bool { self.cpsr().read_masked(N_MASK) != 0 }
//     fn set_neg_lt(&mut self)    { self.cpsr_mut().set(N_MASK, N_MASK); }
//     fn reset_neg_lt(&mut self)  { self.cpsr_mut().reset(N_MASK, N_MASK); }

//     // Zero
//     fn is_zero(&self) -> bool { self.cpsr().read_masked(Z_MASK) != 0 }
//     fn set_zero(&mut self)    { self.cpsr_mut().set(Z_MASK, Z_MASK); }
//     fn reset_zero(&mut self)  { self.cpsr_mut().reset(Z_MASK, Z_MASK); }

//     // Carry, borrow, or extend
//     fn is_carry(&self) -> bool { self.cpsr().read_masked(C_MASK) != 0 }
//     fn set_carry(&mut self)    { self.cpsr_mut().set(C_MASK, C_MASK); }
//     fn reset_carry(&mut self)  { self.cpsr_mut().reset(C_MASK, C_MASK); }

//     // Overflow
//     fn is_overflow(&self) -> bool { self.cpsr().read_masked(V_MASK) != 0 }
//     fn set_overflow(&mut self)    { self.cpsr_mut().set(V_MASK, V_MASK); }
//     fn reset_overflow(&mut self)  { self.cpsr_mut().reset(V_MASK, V_MASK); }

//     // Reset condition bits
//     fn reset_cond(&mut self) { self.cpsr_mut().reset(COND_MASK, COND_MASK); }

//     // IRQ disable
//     fn is_irq_disable(&self) -> bool { self.cpsr().read_masked(I_MASK) != 0 }
//     fn set_irq_disable(&mut self)    { self.cpsr_mut().set(I_MASK, I_MASK); }
//     fn reset_irq_disable(&mut self)  { self.cpsr_mut().reset(I_MASK, I_MASK); }

//     // FRQ disable
//     fn is_fiq_disable(&self) -> bool { self.cpsr().read_masked(F_MASK) != 0 }
//     fn set_fiq_disable(&mut self)    { self.cpsr_mut().set(F_MASK, F_MASK); }
//     fn reset_fiq_disable(&mut self)  { self.cpsr_mut().reset(F_MASK, F_MASK); }

//     // Thumb mode
//     fn is_thumb(&self) -> bool { self.cpsr().read_masked(T_MASK) != 0 }
//     fn set_thumb(&mut self)    { self.cpsr_mut().set(T_MASK, T_MASK); }
//     fn reset_thumb(&mut self)  { self.cpsr_mut().reset(T_MASK, T_MASK); }
// }


// pub struct UserARM7<'a> {
//     cpu: &'a mut ARM7,
// }

// pub struct SystemARM7<'a> {
//     cpu: &'a mut ARM7,
// }

// pub struct IRQARM7<'a> {
//     cpu: &'a mut ARM7,
// }

// pub struct FIRARM7<'a> {
//     cpu: &'a mut ARM7,
// }

// pub struct SupervisorARM7<'a> {
//     cpu: &'a mut ARM7,
// }

// pub struct AbortARM7<'a> {
//     cpu: &'a mut ARM7,
// }

// pub struct UndefinedARM7<'a> {
//     cpu: &'a mut ARM7,
// }

// Register availability map based on mode in THUMB state from:
// http://www.atmel.com/Images/DDI0029G_7TDMI_R3_trm.pdf
// section 2.6.2, page 2-10
