mod mem_regions;

use gba_mem::mem_regions::{SystemRom, ExternRam, InternRam,
                           PalettRam, VisualRam, OAM, PakRom,
                           MemRead, MemWrite, MemoryRegion};
use std::io;

pub type Address = usize;

#[derive(Debug)]
pub struct Memory {
    sys_rom: SystemRom,
    ext_ram: ExternRam,
    int_ram: InternRam,
    pal_ram: PalettRam,
    vis_ram: VisualRam,
    oam:     OAM,
    pak_rom: PakRom,
}

impl Memory {
    pub fn new(pak_filename: &str) -> io::Result<Memory> {
        println!("WARNING: BIOS emulation not implemented. Please emulate bios rather than use a ROM.");
        Ok(Memory {
            sys_rom: SystemRom::create_from_array(include_bytes!("../../roms/gba.bin")),
            ext_ram: ExternRam::default(),
            int_ram: InternRam::default(),
            pal_ram: PalettRam::default(),
            vis_ram: VisualRam::default(),
            oam:     OAM::default(),
            pak_rom: try!(PakRom::create_from_file(pak_filename)),
        })
    }

    pub fn read<T>(&self, addr: Address) -> T
        where SystemRom: MemRead<T>,
              ExternRam: MemRead<T>,
              InternRam: MemRead<T>,
              PalettRam: MemRead<T>,
              VisualRam: MemRead<T>,
              OAM: MemRead<T>,
              PakRom: MemRead<T> {
        match addr {
            _ if addr >= SystemRom::lo() && addr <= SystemRom::hi() =>
                <SystemRom as MemRead<T>>::read(&self.sys_rom, addr),
            _ if addr >= ExternRam::lo() && addr <= ExternRam::hi() =>
                <ExternRam as MemRead<T>>::read(&self.ext_ram, addr),
            _ if addr >= InternRam::lo() && addr <= InternRam::hi() =>
                <InternRam as MemRead<T>>::read(&self.int_ram, addr),
            _ if addr >= PalettRam::lo() && addr <= PalettRam::hi() =>
                <PalettRam as MemRead<T>>::read(&self.pal_ram, addr),
            _ if addr >= VisualRam::lo() && addr <= VisualRam::hi() =>
                <VisualRam as MemRead<T>>::read(&self.vis_ram, addr),
            _ if addr >= OAM::lo() && addr <= OAM::hi() =>
                <OAM as MemRead<T>>::read(&self.oam, addr),
            _ if addr >= PakRom::lo() && addr <= PakRom::hi() =>
                <PakRom as MemRead<T>>::read(&self.pak_rom, addr),
            _ => unreachable!(),
        }
    }

    pub fn write8<T>(&mut self, addr: Address, val: T)
        where ExternRam: MemWrite<T>,
              InternRam: MemWrite<T>,
              PakRom: MemWrite<T> {
        match addr {
            _ if addr >= ExternRam::lo() && addr <= ExternRam::hi() =>
                <ExternRam as MemWrite<T>>::write(&mut self.ext_ram, addr, val),
            _ if addr >= InternRam::lo() && addr <= InternRam::hi() =>
                <InternRam as MemWrite<T>>::write(&mut self.int_ram, addr, val),
            _ if addr >= PakRom::lo() && addr <= PakRom::hi() =>
                <PakRom as MemWrite<T>>::write(&mut self.pak_rom, addr, val),
            _ => unreachable!(),
        }
    }

    pub fn write16<T>(&mut self, addr: Address, val: T)
        where ExternRam: MemWrite<T>,
              InternRam: MemWrite<T>,
              PalettRam: MemWrite<T>,
              VisualRam: MemWrite<T>,
              OAM: MemWrite<T>,
              PakRom: MemWrite<T> {
        match addr {
            _ if addr >= ExternRam::lo() && addr <= ExternRam::hi() =>
                <ExternRam as MemWrite<T>>::write(&mut self.ext_ram, addr, val),
            _ if addr >= InternRam::lo() && addr <= InternRam::hi() =>
                <InternRam as MemWrite<T>>::write(&mut self.int_ram, addr, val),
            _ if addr >= PalettRam::lo() && addr <= PalettRam::hi() =>
                <PalettRam as MemWrite<T>>::write(&mut self.pal_ram, addr, val),
            _ if addr >= VisualRam::lo() && addr <= VisualRam::hi() =>
                <VisualRam as MemWrite<T>>::write(&mut self.vis_ram, addr, val),
            _ if addr >= OAM::lo() && addr <= OAM::hi() =>
                <OAM as MemWrite<T>>::write(&mut self.oam, addr, val),
            _ if addr >= PakRom::lo() && addr <= PakRom::hi() =>
                <PakRom as MemWrite<T>>::write(&mut self.pak_rom, addr, val),
            _ => unreachable!(),
        }
    }

    pub fn write32<T>(&mut self, addr: Address, val: T)
        where ExternRam: MemWrite<T>,
              InternRam: MemWrite<T>,
              PalettRam: MemWrite<T>,
              VisualRam: MemWrite<T>,
              OAM: MemWrite<T>,
              PakRom: MemWrite<T> {
        self.write16::<T>(addr, val);
    }
}

// impl Mem {
//     fn new(pak_filename: String) -> Mem {
//         let mut pak_rom_file = File::open(pak_filename).expect("Failed to open PAK ROM file");
//         let mut pak_rom = Vec::<u8>::new();
//         pak_rom_file.read_to_end(&mut pak_rom).expect("Failed to read PAK ROM");
//         //let mut pak_rom = u8_slice_to_u16_slice(&pak_rom);

//         Mem {
//             system_rom: [0;SYSTEM_ROM_SIZE as usize],
//             extern_ram: [0;EXTERN_RAM_SIZE as usize],
//             intern_ram: [0;INTERN_RAM_SIZE as usize],
//             palett_ram: [0;PALETT_RAM_SIZE as usize],
//             vram:       [0;VRAM_SIZE as usize],
//             oam:        [0;OAM_SIZE as usize],
//             pak_rom:    pak_rom.into_boxed_slice(),
//         }
//     }
// }
