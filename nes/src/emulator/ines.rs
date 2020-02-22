use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;
use std::vec::Vec;

use crate::emulator::mappers;
use crate::emulator::memory::{Mapper, Memory};
use crate::emulator::ppu;

pub struct ROM {
    data: Vec<u8>,
}

impl ROM {
    pub fn load<P: AsRef<Path>>(path: P) -> ROM {
        let mut file = match File::open(path) {
            Err(cause) => panic!("Couldn't open file: {}", cause),
            Ok(file) => file,
        };

        let mut contents = vec![];
        match file.read_to_end(&mut contents) {
            Err(cause) => panic!("Couldn't read file: {}", cause),
            Ok(_) => (),
        };

        ROM::from_bytes(contents)
    }

    pub fn from_bytes(data: Vec<u8>) -> ROM {
        ROM { data }
    }

    pub fn mapper_number(&self) -> u8 {
        ((self.data[6] & 0xF0) >> 4) | (self.data[7] & 0xF0)
    }

    pub fn prg_rom(&self) -> Memory {
        let size = self.prg_rom_size_bytes();
        let start = 16 as usize;
        let end = start + size as usize;
        Memory::new_rom(self.data[start..end].to_vec())
    }

    pub fn prg_rom_size_bytes(&self) -> u32 {
        (self.data[4] as u32) * 16384
    }

    pub fn chr_mem(&self) -> Memory {
        let prg_size = self.prg_rom_size_bytes();
        let size = self.chr_rom_size_bytes();

        if size == 0 {
            // Cartridge uses chr_ram.
            Memory::new_ram(0x2000)
        } else {
            let start = (16 + prg_size) as usize;
            let end = start + size as usize;
            Memory::new_rom(self.data[start..end].to_vec())
        }
    }

    pub fn chr_rom_size_bytes(&self) -> u32 {
        (self.data[5] as u32) * 8192
    }

    pub fn mirror_mode(&self) -> ppu::MirrorMode {
        if self.data[6] & 0x1 == 0 {
            ppu::MirrorMode::Horizontal
        } else {
            ppu::MirrorMode::Vertical
        }
    }

    pub fn get_mapper(&self) -> Rc<RefCell<dyn Mapper>> {
        let prg_rom = self.prg_rom();
        let chr_mem = self.chr_mem();
        let mirror_mode = self.mirror_mode();

        match self.mapper_number() {
            0 => Rc::new(RefCell::new(mappers::NROM::new(
                prg_rom,
                chr_mem,
                mirror_mode,
            ))),
            1 => Rc::new(RefCell::new(mappers::MMC1::new(prg_rom, chr_mem))),
            2 => Rc::new(RefCell::new(mappers::UXROM::new(
                prg_rom,
                chr_mem,
                mirror_mode,
            ))),
            3 => Rc::new(RefCell::new(mappers::CNROM::new(
                prg_rom,
                chr_mem,
                mirror_mode,
            ))),
            4 => Rc::new(RefCell::new(mappers::MMC3::new(prg_rom, chr_mem))),
            7 => Rc::new(RefCell::new(mappers::AXROM::new(prg_rom, chr_mem))),
            11 => Rc::new(RefCell::new(mappers::ColorDreams::new(
                prg_rom,
                chr_mem,
                mirror_mode,
            ))),
            _ => panic!("Unknown mapper: {}", self.mapper_number()),
        }
    }
}
