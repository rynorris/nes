use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::vec::Vec;

use emulator::ppu;

pub struct ROM {
    data: Vec<u8>,
}

impl ROM {
    pub fn load<P : AsRef<Path>>(path: P) -> ROM {
        let mut file = match File::open(path) {
            Err(cause) => panic!("Couldn't open file: {}", cause),
            Ok(file) => file,
        };

        let mut contents = vec![];
        match file.read_to_end(&mut contents) {
            Err(cause) => panic!("Couldn't read file: {}", cause),
            Ok(_) => (),
        };

        ROM {
            data: contents,
        }
    }

    pub fn mapper_number(&self) -> u8 {
        ((self.data[6] & 0xF0) >> 4) | (self.data[7] & 0xF0)
    }

    pub fn prg_rom(&self) -> &[u8] {
        let size = self.prg_rom_size_bytes();
        let start = 16 as usize;
        let end = start + size as usize;
        &self.data[start..end]
    }

    pub fn prg_rom_size_bytes(&self) -> u32 {
        (self.data[4] as u32) * 16384
    }

    pub fn chr_rom(&self) -> &[u8] {
        let prg_size = self.prg_rom_size_bytes();
        let size = self.chr_rom_size_bytes();

        if size == 0 {
            // Cartridge uses chr_ram.
            &[0; 0x2000]
        } else {
            let start = (16 + prg_size) as usize;
            let end = start + size as usize;
            &self.data[start..end]
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
}
