use std::fmt;
use std::fmt::Debug;
use std::io::{Cursor, Read, Write};
use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};

use gba_mem::Address;

pub const BYTE_WIDTH: u16 = 8;

#[derive(Clone, Copy, Debug)]
pub enum BusWidth {
    BW16,
    BW32,
}

impl BusWidth {
    #[inline]
    pub fn to_bytes(&self) -> u16 {
        match *self {
            BusWidth::BW16 => 2,
            BusWidth::BW32 => 4,
        }
    }

    #[inline]
    pub fn to_bits(&self) -> u16 {
        self.to_bytes() * BYTE_WIDTH
    }
}

pub trait MemoryRegion {
    fn lo() -> Address;
    fn hi() -> Address;
    fn bus_width() -> BusWidth;

    #[inline]
    fn len() -> usize {
        (Self::hi() - Self::lo())/BYTE_WIDTH as Address + 1
    }

    #[inline]
    fn contains_cmp(addr: Address) -> isize {
        if addr < Self::lo() {
            -1
        }
        else if addr > Self::hi() {
            1
        }
        else {
            0
        }
    }

    #[inline]
    fn contains(addr: Address) -> bool {
        match Self::contains_cmp(addr) {
            0 => true,
            _ => false,
        }
    }
}

pub trait MemRead<T> {
    fn read(&self, addr: Address) -> T;
}

pub trait MemWrite<T> {
    fn write(&mut self, addr: Address, val: T);
}

macro_rules! new_mem_region {
    ($name:ident, $lo:expr, $hi:expr, $bus:expr) => {
        pub struct $name {
            mem: Vec<u8>,//Box<[u8; (($hi - $lo) as usize)/(BYTE_WIDTH as usize) + 1]>,
        }

        impl $name {
            pub fn create_from_array(array: &[u8]) -> $name {
                let mut ret = $name {
                    mem: vec![0; (($hi - $lo) as usize)/(BYTE_WIDTH as usize) + 1],
                };

                println!("{:x}\n{:x}", ret.mem.len(), array.len());
                ret.mem.copy_from_slice(array);

                ret
            }

            pub fn create_from_file(file_path: &str) -> io::Result<$name> {
                let file_path = Path::new(file_path);
                let mut file = try!(File::open(file_path));
                let file_len = try!(file.metadata()).len() as usize;
                let mem_len = $name::len();

                if file_len > mem_len {
                    let errmsg = match file_path.to_str() {
                        Some(f) => format!("File {} ({} Bytes) is too big for the {} memory region ({} Bytes).", f, file_len, stringify!($name), mem_len),
                        None => format!("File is too big for the {} memory region.", stringify!($name)),
                    };

                    Err(io::Error::new(io::ErrorKind::Other, errmsg))
                }
                else {
                    let mut ret = $name {
                        mem: vec![0; (($hi - $lo) as usize)/(BYTE_WIDTH as usize) + 1],
                    };

                    try!(file.read(ret.mem.as_mut_slice()));

                    Ok(ret)
                }
            }

            pub fn to_file(&self, file_path: &str) {
                let file_path = Path::new(file_path);
                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(file_path).unwrap();

                file.write_all(self.mem.as_ref()).unwrap();
            }
        }

        impl Default for $name {
            fn default() -> Self {
                $name {
                    mem: vec![0; (($hi - $lo) as usize)/(BYTE_WIDTH as usize) + 1],
                }
            }
        }

        impl Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}{{ lo:{:#x}, lo:{:#x}, bus_width:{} }}",
                       stringify!($name), $name::lo(), $name::hi(),
                       $name::bus_width().to_bits())
            }
        }

        impl MemoryRegion for $name {
            #[inline]
            fn lo() -> Address { $lo }

            #[inline]
            fn hi() -> Address { $hi }

            #[inline]
            fn bus_width() -> BusWidth { $bus }
        }
    };
}

macro_rules! def_mem_region_ops {
    ($name:ty) => {};

    (mem_read_as_self: $name:ty, $func:ident, $ty:ty) => {
        #[allow(trivial_numeric_casts)]
        impl MemRead<$ty> for $name {
            fn read(&self, addr: Address) -> $ty {
                self.mem[(addr - Self::lo()) as usize] as $ty
            }
        }
    };

    (mem_read_as_other: $name:ty, $func:ident, $ty:ty) => {
        impl MemRead<$ty> for $name {
            fn read(&self, addr: Address) -> $ty {
                let loc = (addr - Self::lo()) as u64;
                let mut rdr = Cursor::new((*self.mem).as_ref());
                rdr.set_position(loc);
                rdr.$func::<LittleEndian>().unwrap()
            }
        }
    };

    (mem_write_as_self: $name:ty, $func:ident, $ty:ty) => {
        #[allow(trivial_numeric_casts)]
        impl MemWrite<$ty> for $name {
            fn write(&mut self, addr: Address, val: $ty) {
                self.mem[addr - Self::lo()] = val as u8;
            }
        }
    };

    (mem_write_as_other: $name:ty, $func:ident, $ty:ty) => {
        impl MemWrite<$ty> for $name {
            fn write(&mut self, addr: Address, val: $ty) {
                let loc = (addr - Self::lo()) as u64;
                let mut wtr = Cursor::new((*self.mem).as_mut());
                wtr.set_position(loc);
                wtr.$func::<LittleEndian>(val).unwrap()
            }
        }
    };


    (read: $name:ty, 8)  => {
        def_mem_region_ops!(mem_read_as_self:   $name, read_i8,   i8 );
        def_mem_region_ops!(mem_read_as_self:   $name, read_u8,   u8 );
    };
    (read: $name:ty, 16) => {
        def_mem_region_ops!(mem_read_as_other:  $name, read_i16,  i16);
        def_mem_region_ops!(mem_read_as_other:  $name, read_u16,  u16);
    };
    (read: $name:ty, 32) => {
        def_mem_region_ops!(mem_read_as_other:  $name, read_i32,  i32);
        def_mem_region_ops!(mem_read_as_other:  $name, read_u32,  u32);
        def_mem_region_ops!(mem_read_as_other:  $name, read_f32,  f32);
    };

    (write: $name:ty, 8)  => {
        def_mem_region_ops!(mem_write_as_self:  $name, write_i8,  i8 );
        def_mem_region_ops!(mem_write_as_self:  $name, write_u8,  u8 );
    };
    (write: $name:ty, 16) => {
        def_mem_region_ops!(mem_write_as_other: $name, write_i16, i16);
        def_mem_region_ops!(mem_write_as_other: $name, write_u16, u16);
    };
    (write: $name:ty, 32) => {
        def_mem_region_ops!(mem_write_as_other: $name, write_i32, i32);
        def_mem_region_ops!(mem_write_as_other: $name, write_u32, u32);
        def_mem_region_ops!(mem_write_as_other: $name, write_f32, f32);
    };

    ($name:ty, r, $tok:tt) => { def_mem_region_ops!(read:  $name, $tok); };
    ($name:ty, w, $tok:tt) => { def_mem_region_ops!(write: $name, $tok); };
    ($name:ty, rw, $tok:tt) => {
        def_mem_region_ops!(read:  $name, $tok);
        def_mem_region_ops!(write: $name, $tok);
    };
    ($name:ty, wr, $tok:tt) => { def_mem_region_ops!($name, rw, $tok); };

    ($name:ty, $op:tt[]) => {};
    ($name:ty, $op:tt[ $tok:tt ]) => { def_mem_region_ops!($name, $op, $tok); };
    ($name:ty, $op:tt[ $tok:tt, $($toks:tt),* ]) => {
        def_mem_region_ops!($name, $op, $tok);
        def_mem_region_ops!($name, $op[ $($toks),* ]);
    };

    ($name:ty, $op:tt[ $($toks:tt),* ], $( $r_op:tt[ $($r_size:tt),* ] ),*) => {
        def_mem_region_ops!($name, $op[ $($toks),* ]);
        def_mem_region_ops!($name, $( $r_op[ $($r_size),* ] ),*);
    };
}

// Declare memory regions
new_mem_region!(SystemRom, 0x00000000, 0x0001FFFF, BusWidth::BW32);
new_mem_region!(ExternRam, 0x02000000, 0x0203FFFF, BusWidth::BW32);
new_mem_region!(InternRam, 0x03000000, 0x03007FFF, BusWidth::BW32);
new_mem_region!(PalettRam, 0x05000000, 0x050003FF, BusWidth::BW32);
new_mem_region!(VisualRam, 0x06000000, 0x06017FFF, BusWidth::BW16);
new_mem_region!(OAM,       0x07000000, 0x070003FF, BusWidth::BW32);
new_mem_region!(PakRom,    0x08000000, 0x0FFFFFFF, BusWidth::BW16);

// Implement read and write operations
def_mem_region_ops!(SystemRom, r[8, 16, 32]);
def_mem_region_ops!(ExternRam, rw[8, 16, 32]);
def_mem_region_ops!(InternRam, rw[8, 16, 32]);
def_mem_region_ops!(PalettRam, r[8, 16, 32], w[16, 32]);
def_mem_region_ops!(VisualRam, r[8, 16, 32], w[16, 32]);
def_mem_region_ops!(OAM,       r[8, 16, 32], w[16, 32]);
def_mem_region_ops!(PakRom,    rw[8, 16, 32]);
