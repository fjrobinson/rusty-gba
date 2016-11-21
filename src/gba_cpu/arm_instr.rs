use std::fmt;

use gba_cpu::{Instruction, IType, SIType, ARM7};
use gba_mem::{Address, Memory};

const COND_MASK: IType = 0xF0000000;

// Condition codes:
// https://www.scss.tcd.ie/~waldroj/3d1/arm_arm.pdf
// section A3.2.1
const COND_EQ: i8 = 0b0000; // Equal; Z set
const COND_NE: i8 = 0b0001; // Not equal; Z clear
const COND_CS: i8 = 0b0010; // Carry set; C set     (AKA: HS)
const COND_CC: i8 = 0b0011; // Carry clear; C clear (AKA: LO)
const COND_MI: i8 = 0b0100; // Minus/negative; N set
const COND_PL: i8 = 0b0101; // Plus/positive or zero; N clear
const COND_VS: i8 = 0b0110; // Overflow; V set
const COND_VC: i8 = 0b0111; // No overflow; V clear
const COND_HI: i8 = 0b1000; // Unsigned higher; C set and Z clear
const COND_LS: i8 = 0b1001; // Unsigned lower; C clear and Z set
const COND_GE: i8 = 0b1010; // Signed greater than or equal; N == V
const COND_LT: i8 = 0b1011; // Signed less than; N != V
const COND_GT: i8 = 0b1100; // Signed greater than; (Z == 0 && N == V)
const COND_LE: i8 = 0b1101; // Signed less than or equal; (Z == 1 || N != V)
const COND_AL: i8 = 0b1110; // Always

const COND_SHIFT: IType = 27;
const COND_EQ_MASKED: IType = 0b0000 << COND_SHIFT;
const COND_NE_MASKED: IType = 0b0001 << COND_SHIFT;
const COND_CS_MASKED: IType = 0b0010 << COND_SHIFT;
const COND_CC_MASKED: IType = 0b0011 << COND_SHIFT;
const COND_MI_MASKED: IType = 0b0100 << COND_SHIFT;
const COND_PL_MASKED: IType = 0b0101 << COND_SHIFT;
const COND_VS_MASKED: IType = 0b0110 << COND_SHIFT;
const COND_VC_MASKED: IType = 0b0111 << COND_SHIFT;
const COND_HI_MASKED: IType = 0b1000 << COND_SHIFT;
const COND_LS_MASKED: IType = 0b1001 << COND_SHIFT;
const COND_GE_MASKED: IType = 0b1010 << COND_SHIFT;
const COND_LT_MASKED: IType = 0b1011 << COND_SHIFT;
const COND_GT_MASKED: IType = 0b1100 << COND_SHIFT;
const COND_LE_MASKED: IType = 0b1101 << COND_SHIFT;
const COND_AL_MASKED: IType = 0b1110 << COND_SHIFT;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cond {
    EQ = COND_EQ_MASKED as isize,
    NE = COND_NE_MASKED as isize,
    CS = COND_CS_MASKED as isize,
    CC = COND_CC_MASKED as isize,
    MI = COND_MI_MASKED as isize,
    PL = COND_PL_MASKED as isize,
    VS = COND_VS_MASKED as isize,
    VC = COND_VC_MASKED as isize,
    HI = COND_HI_MASKED as isize,
    LS = COND_LS_MASKED as isize,
    GE = COND_GE_MASKED as isize,
    LT = COND_LT_MASKED as isize,
    GT = COND_GT_MASKED as isize,
    LE = COND_LE_MASKED as isize,
    AL = COND_AL_MASKED as isize,
}

impl Cond {
    fn decode(instr: IType) -> Cond {
        match instr & COND_MASK {
            COND_EQ_MASKED => Cond::EQ,
            COND_NE_MASKED => Cond::NE,
            COND_CS_MASKED => Cond::CS,
            COND_CC_MASKED => Cond::CC,
            COND_MI_MASKED => Cond::MI,
            COND_PL_MASKED => Cond::PL,
            COND_VS_MASKED => Cond::VS,
            COND_VC_MASKED => Cond::VC,
            COND_HI_MASKED => Cond::HI,
            COND_LS_MASKED => Cond::LS,
            COND_GE_MASKED => Cond::GE,
            COND_LT_MASKED => Cond::LT,
            COND_GT_MASKED => Cond::GT,
            COND_LE_MASKED => Cond::LE,
            COND_AL_MASKED => Cond::AL,
            _ => unreachable!(),
        }
    }

    fn is_satisfied(&self, cpu: &ARM7) -> bool {
        // Check ensure correct shift amount at compile time
        assert!(0xF << COND_SHIFT == COND_MASK);

        match *self as IType & COND_MASK {
            COND_EQ_MASKED =>  cpu.is_zero(),
            COND_NE_MASKED => !cpu.is_zero(),
            COND_CS_MASKED =>  cpu.is_carry(),
            COND_CC_MASKED => !cpu.is_carry(),
            COND_MI_MASKED =>  cpu.is_neg_lt(),
            COND_PL_MASKED => !cpu.is_neg_lt(),
            COND_VS_MASKED =>  cpu.is_overflow(),
            COND_VC_MASKED => !cpu.is_overflow(),
            COND_HI_MASKED =>  cpu.is_carry() && !cpu.is_zero(),
            COND_LS_MASKED => !cpu.is_carry() &&  cpu.is_zero(),
            COND_GE_MASKED =>  cpu.is_neg_lt() == cpu.is_overflow(),
            COND_LT_MASKED =>  cpu.is_neg_lt() != cpu.is_overflow(),
            COND_GT_MASKED =>  cpu.is_zero() && cpu.is_neg_lt() == cpu.is_overflow(),
            COND_LE_MASKED => !cpu.is_zero() || cpu.is_neg_lt() != cpu.is_overflow(),
            COND_AL_MASKED =>  true,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for Cond {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = match *self {
            Cond::EQ => "eq",
            Cond::NE => "ne",
            Cond::CS => "cs",
            Cond::CC => "cc",
            Cond::MI => "mi",
            Cond::PL => "pl",
            Cond::VS => "vs",
            Cond::VC => "vc",
            Cond::HI => "hi",
            Cond::LS => "ls",
            Cond::GE => "ge",
            Cond::LT => "lt",
            Cond::GT => "gt",
            Cond::LE => "le",
            Cond::AL => "",
        };

        write!(f, "{}", c)
    }
}

const DATA_OPCODE_MASK: IType = 0x01E00000;

// pub enum ARM7Instruction {
//     Branch(Branch),
//     Unknown,
// }

// impl ARM7Instruction {
//     fn fetch(pc: Address, mem: &Memory) -> IType {
//         mem.read::<IType>(pc)
//     }

//     fn decode(instr: IType) -> ARM7Instruction
// }

// The ARM7TDMI uses the ARMv4T architecture
// Instuction encodings from:
// https://www.scss.tcd.ie/~waldroj/3d1/arm_arm.pdf
// section A3.1
// pub enum ARMInstruction {
//     DataPIS,
//     DataPRS,
//     Misc1,
//     Misc2,
//     Multiplies,
//     Branch{link: bool, off: IType},
// }

// Implementation of branch instruction
// Instruction description from:
// https://www.scss.tcd.ie/~waldroj/3d1/arm_arm.pdf
// section A4.1.5; page A4-10 to A4-11
const BRANCH_MASK:  IType = 0x0E000000;
const BRANCH_IDENT: IType = 0x0A000000;
const BRANCH_LINK:  IType = 0x01000000;
const BRANCH_SIGN:  IType = 0x00800000;
const BRANCH_EXTEND:IType = 0xFF000000;

pub struct Branch {
    cond: Cond,
    link: bool,
    off: SIType,
}

impl Instruction for Branch {
    type CPU = ARM7;
    type Instr = IType;

    fn decode(instr: IType) -> Branch {
        Branch {
            cond: Cond::decode(instr),
            link: instr & BRANCH_LINK == BRANCH_LINK,
            off: (if instr & BRANCH_SIGN != 0 {
                instr | BRANCH_EXTEND
            }
            else {
                instr & !BRANCH_EXTEND
            } << 2) as SIType, // TODO: Does this cause an overflow check?
        }
    }

    fn execute(&self, cpu: &mut Self::CPU, mem: &mut Memory) {

    }
}

impl fmt::Display for Branch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let l = if self.link {
            "l"
        }
        else {
            ""
        };

        write!(f, "b{}{}\t{:#x}", l, self.cond, self.off)
    }
}

// TODO: Determine if this is necessary
fn decode(instr: IType) -> Branch {
    if instr & BRANCH_MASK == BRANCH_IDENT {
        return Branch::decode(instr)
    }
    unimplemented!()
}

// ARM and THUMB instruction definitions can be found at:
// https://www.scss.tcd.ie/~waldroj/3d1/arm_arm.pdf

// Data processing instructions
