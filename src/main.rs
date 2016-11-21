// #![cfg_attr(feature = "dev", allow(unstable_features))]
// #![cfg_attr(feature = "dev", feature(plugin))]
// #![cfg_attr(feature = "dev", plugin(clippy))]

// #![deny(missing_docs,
//         missing_debug_implementations, missing_copy_implementations,
//         trivial_casts, trivial_numeric_casts,
//         unsafe_code,
//         unstable_features,
//         unused_import_braces, unused_qualifications)]
#![warn(missing_docs,
        missing_debug_implementations, missing_copy_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code, unstable_features,
        unused_import_braces, unused_qualifications)]

extern crate byteorder;

pub mod gba_mem;
pub mod gba_cpu;

use std::env;
use std::fs::File;

pub use gba_cpu::arm_cpu::ARM7;
pub use gba_mem::Memory;

fn main() {
    let pak_rom_filename = env::args()
        .nth(1)
        .expect("PAK ROM argument not specified");

    let mut m = Memory::new(pak_rom_filename.as_str()).unwrap();

    m.write32::<u32>(0x02000000, 0xdeadbeef);

    println!("{:#x}", m.read::<u8>(0x02000000));

    let cpu = ARM7::default();
    println!("{}", cpu);
}
