//use std::mem; // Needed if useing transmute
use self::ARM7Mode::*;

use std::fmt;
use gba_cpu::RType;
use gba_cpu::IType;

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
pub const CPSR:     i8 = 16;
pub const R8_FIQ:   i8 = 17;
pub const R9_FIQ:   i8 = 18;
pub const R10_FIQ:  i8 = 19;
pub const R11_FIQ:  i8 = 20;
pub const R12_FIQ:  i8 = 21;
pub const R13_FIQ:  i8 = 22;
pub const R14_FIQ:  i8 = 23;
pub const SPSR_FIQ: i8 = 24;
pub const R13_SV:   i8 = 25;
pub const R14_SV:   i8 = 26;
pub const SPSR_SV:  i8 = 27;
pub const R13_ABT:  i8 = 28;
pub const R14_ABT:  i8 = 29;
pub const SPSR_ABT: i8 = 30;
pub const R13_IRQ:  i8 = 31;
pub const R14_IRQ:  i8 = 32;
pub const SPSR_IRQ: i8 = 33;
pub const R13_UND:  i8 = 34;
pub const R14_UND:  i8 = 35;
pub const SPSR_UND: i8 = 36;
pub const NUM_REGS: usize = 37;

pub const PC: i8 = R15;

#[derive(Copy, Clone, Debug, Default)]
pub struct Register(RType);

impl Register {
    pub fn read(&self) -> RType {
        self.0
    }

    // pub fn read_u32(&self) -> u32 {
    //     self.read_RType()
    // }

    // pub fn read_i32(&self) -> i32 {
    //     unsafe {
    //         mem::transmute::<RType, i32>(self.0)
    //     }
    // }

    // pub fn read_f32(&self) -> f32 {
    //     unsafe {
    //         mem::transmute::<RType, f32>(self.0)
    //     }
    // }

    pub fn write(&mut self, val: RType) {
        self.0 = val
    }

    // pub fn write_i32(&self, val: i32) -> i32 {
    //     unsafe {
    //         mem::transmute::<RType, i32>(self.0)
    //     }
    // }

    // pub fn write_f32(&mut self, val: f32) {
    //     unsafe {
    //         mem::transmute::<RType, f32>(self.0)
    //     }
    // }

    pub fn read_masked(&self, mask: RType) -> RType {
        self.0 & mask
    }

    pub fn set(&mut self, mask: RType, val: RType) {
        self.0 |= val & mask
    }

    pub fn reset(&mut self, mask: RType, val: RType) {
        self.0 &= !(val & mask)
    }

    pub fn toggle(&mut self, mask: RType, val: RType) {
        self.0 ^= val & mask
    }
}

// Modes of execution for ARM7TDMI
// TODO: Consider creating a typed state machine if performance is an issue
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ARM7Mode {
    User       = USER_MODE as isize,
    FIQ        = FIQ_MODE as isize,
    IRQ        = IRQ_MODE as isize,
    Supervisor = SV_MODE as isize,
    Abort      = ABRT_MODE as isize,
    Undefined  = UDEF_MODE as isize,
    System     = SYS_MODE as isize,
}

impl fmt::Display for ARM7Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mode_word = match *self {
            User => "User",
            FIQ => "FIQ",
            IRQ => "IRQ",
            Supervisor => "Supervisor",
            Abort => "Abort",
            Undefined => "Undefined",
            System => "System",
        };
        write![f, "{}", mode_word]
    }
}

// Registers from:
// http://www.atmel.com/Images/DDI0029G_7TDMI_R3_trm.pdf
// section 2.6, page 2-8
pub struct ARM7 {
    regs: [Register; NUM_REGS],
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

    fn reg(&self, reg_num: i8) -> &Register {
        &self.regs[reg_num as usize]
    }

    fn reg_mut(&mut self, reg_num: i8) -> &mut Register {
        &mut self.regs[reg_num as usize]
    }

    fn reg_map(&self, reg_num: i8) -> Option<&Register> {
        match self.reg_map_index(reg_num) {
            Some(x) => Some(self.reg(x)),
            None => None,
        }
    }

    fn reg_map_mut(&mut self, reg_num: i8) -> Option<&mut Register> {
        match self.reg_map_index(reg_num) {
            Some(x) => Some(self.reg_mut(x)),
            None => None,
        }
    }

    // PC register
    pub fn pc(&self) -> RType {
        self.reg(PC).read()
    }

    pub fn inc_pc(&mut self) {
        let pc_val = self.reg(PC).read();
        if self.is_thumb() {
            self.reg_mut(PC).write((pc_val + 0b10) & 0xFFFFFFFE);
        }
        else {
            self.reg_mut(PC).write((pc_val + 0b100) & 0xFFFFFFFC)
        }
    }

    // CPSR Register access
    // TODO: Do we need mutators for this?
    pub fn cpsr(&self) -> RType {
        self.reg(CPSR).read()
    }

    // Negative or less than
    pub fn is_neg_lt(&self) -> bool { self.reg(CPSR).read_masked(N_MASK) != 0 }
    pub fn set_neg_lt(&mut self)    { self.reg_mut(CPSR).set(N_MASK, N_MASK); }
    pub fn reset_neg_lt(&mut self)  { self.reg_mut(CPSR).reset(N_MASK, N_MASK); }

    // Zero
    pub fn is_zero(&self) -> bool { self.reg(CPSR).read_masked(Z_MASK) != 0 }
    pub fn set_zero(&mut self)    { self.reg_mut(CPSR).set(Z_MASK, Z_MASK); }
    pub fn reset_zero(&mut self)  { self.reg_mut(CPSR).reset(Z_MASK, Z_MASK); }

    // Carry, borrow, or extend
    pub fn is_carry(&self) -> bool { self.reg(CPSR).read_masked(C_MASK) != 0 }
    pub fn set_carry(&mut self)    { self.reg_mut(CPSR).set(C_MASK, C_MASK); }
    pub fn reset_carry(&mut self)  { self.reg_mut(CPSR).reset(C_MASK, C_MASK); }

    // Overflow
    pub fn is_overflow(&self) -> bool { self.reg(CPSR).read_masked(V_MASK) != 0 }
    pub fn set_overflow(&mut self)    { self.reg_mut(CPSR).set(V_MASK, V_MASK); }
    pub fn reset_overflow(&mut self)  { self.reg_mut(CPSR).reset(V_MASK, V_MASK); }

    // Reset condition bits
    pub fn reset_cond(&mut self) { self.reg_mut(CPSR).reset(COND_MASK, COND_MASK); }

    // IRQ disable
    pub fn is_irq_disable(&self) -> bool { self.reg(CPSR).read_masked(I_MASK) != 0 }
    pub fn set_irq_disable(&mut self)    { self.reg_mut(CPSR).set(I_MASK, I_MASK); }
    pub fn reset_irq_disable(&mut self)  { self.reg_mut(CPSR).reset(I_MASK, I_MASK); }

    // FRQ disable
    pub fn is_fiq_disable(&self) -> bool { self.reg(CPSR).read_masked(F_MASK) != 0 }
    pub fn set_fiq_disable(&mut self)    { self.reg_mut(CPSR).set(F_MASK, F_MASK); }
    pub fn reset_fiq_disable(&mut self)  { self.reg_mut(CPSR).reset(F_MASK, F_MASK); }

    // Thumb mode
    pub fn is_thumb(&self) -> bool { self.reg(CPSR).read_masked(T_MASK) != 0 }
    pub fn set_thumb(&mut self)    { self.reg_mut(CPSR).set(T_MASK, T_MASK); }
    pub fn reset_thumb(&mut self)  { self.reg_mut(CPSR).reset(T_MASK, T_MASK); }

    pub fn mode(&self) -> ARM7Mode {
        match self.reg(CPSR).read_masked(M_MASK) {
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
}

impl fmt::Debug for ARM7 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for ARM7 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write![f, "ARM7TDMI State:\n"]);
        try!(write![f, "\tMode:        {}\n", self.mode()]);
        try!(write![f, "\tThumb Mode:  {}\n", self.is_thumb()]);
        try!(write![f, "\tIRQ Disable: {}\n", self.is_irq_disable()]);
        try!(write![f, "\tFIQ Disable: {}\n", self.is_fiq_disable()]);
        try!(write![f, "\tNeg/Less:    {}\n", self.is_neg_lt()]);
        try!(write![f, "\tZero:        {}\n", self.is_zero()]);
        try!(write![f, "\tCarry:       {}\n", self.is_carry()]);
        try!(write![f, "\tOverflow:    {}\n", self.is_overflow()]);
        try!(write![f, "\tRegisters:\n"]);
        write![f, "\t\tR1: {}({:#x})", 3, 3]
    }
}

// Register availability map based on mode in ARM state from:
// http://www.atmel.com/Images/DDI0029G_7TDMI_R3_trm.pdf
// section 2.6.1, page 2-9


// Register availability map based on mode in THUMB state from:
// http://www.atmel.com/Images/DDI0029G_7TDMI_R3_trm.pdf
// section 2.6.2, page 2-10
