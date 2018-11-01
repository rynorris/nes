use std::fs::File;
use std::io::Read;
use std::vec::Vec;

pub struct ROM {
    data: Vec<u8>,
}

impl ROM {
    pub fn load(path: &str) -> ROM {
        let mut file = match File::open(path) {
            Err(cause) => panic!("Couldn't open {}: {}", path, cause),
            Ok(file) => file,
        };

        let mut contents = vec![];
        match file.read_to_end(&mut contents) {
            Err(cause) => panic!("Couldn't read {}: {}", path, cause),
            Ok(_) => (),
        };

        ROM {
            data: contents,
        }
    }

    pub fn prg_rom(&self) -> &[u8] {
        let size = self.prg_rom_size_bytes();
        let start = 16 as usize;
        let end = start + size as usize;
        &self.data[start..end]
    }

    pub fn prg_rom_size_bytes(&self) -> u16 {
        (self.data[4] as u16) * 16384
    }

    pub fn chr_rom(&self) -> &[u8] {
        let prg_size = self.prg_rom_size_bytes();
        let size = self.chr_rom_size_bytes();
        let start = (16 + prg_size) as usize;
        let end = start + size as usize;
        &self.data[start..end]
    }

    pub fn chr_rom_size_bytes(&self) -> u16 {
        (self.data[5] as u16) * 8192
    }
}
