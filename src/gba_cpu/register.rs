use std::fmt;
use gba_cpu::RType;

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

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write![f, "{}", self.0]
    }
}

impl fmt::UpperHex for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !f.alternate() {
            write![f, "{:X}", self.0]
        }
        else {
            write![f, "{:#X}", self.0]
        }
    }
}

impl fmt::LowerHex for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !f.alternate() {
            write![f, "{:x}", self.0]
        }
        else {
            write![f, "{:#x}", self.0]
        }
    }
}

impl fmt::Binary for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !f.alternate() {
            write![f, "{:032b}", self.0]
        }
        else {
            write![f, "{:#032b}", self.0]
        }
    }
}

impl fmt::Pointer for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write![f, "{:#010x}", self.0]
    }
}
