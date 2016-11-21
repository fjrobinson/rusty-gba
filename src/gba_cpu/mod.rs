pub mod arm_cpu;
pub mod arm_instr;
pub mod register;

pub use gba_mem::Memory;
pub use gba_cpu::arm_cpu::ARM7;

pub type RType = u32;
pub type IType = u32;
pub type SIType = i32;
pub type TIType = u16;

// Common interface for executing and loading instructions
pub trait Instruction {
    type CPU;
    type Instr;

    fn decode(instr: Self::Instr) -> Self;
    fn execute(&self, cpu: &mut Self::CPU, mem: &mut Memory);
}
