use emulator::memory;
use emulator::memory::{Reader, Writer};

pub struct NROM {
    prg_size: u16,
    prg_rom: memory::RAM,
    chr_rom: memory::RAM,
}

impl NROM {
    pub fn new(prg_size: u16, prg_rom: memory::RAM, chr_rom: memory::RAM) -> NROM {
        NROM { prg_size, prg_rom, chr_rom }
    }

    #[inline]
    fn map_prg_address(address: u16, prg_size: u16) -> u16 {
        (address - 0x8000) % prg_size
    }
}

impl memory::Mapper for NROM {
    fn read_chr(&mut self, address: u16) -> u8 {
        self.chr_rom.read(address)
    }

    fn write_chr(&mut self, address: u16, byte: u8) {
        self.chr_rom.write(address, byte)
    }

    fn read_prg(&mut self, address: u16) -> u8 {
        let mapped_address = NROM::map_prg_address(address, self.prg_size);
        self.prg_rom.read(mapped_address)
    }

    fn write_prg(&mut self, address: u16, byte: u8) {
        let mapped_address = NROM::map_prg_address(address, self.prg_size);
        self.prg_rom.write(mapped_address, byte)
    }
}
