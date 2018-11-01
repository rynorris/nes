use emulator::memory;

pub struct NROM {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
}

impl NROM {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> NROM {
        NROM { prg_rom, chr_rom }
    }

    #[inline]
    fn map_prg_address(address: u16, prg_size: u16) -> u16 {
        (address - 0x8000) % prg_size
    }
}

impl memory::Mapper for NROM {
    fn read_chr(&mut self, address: u16) -> u8 {
        self.chr_rom[address as usize]
    }

    fn write_chr(&mut self, _address: u16, _byte: u8) {
        // Can't write to ROM.
    }

    fn read_prg(&mut self, address: u16) -> u8 {
        let mapped_address = NROM::map_prg_address(address, self.prg_rom.len() as u16);
        self.prg_rom[mapped_address as usize]
    }

    fn write_prg(&mut self, _address: u16, _byte: u8) {
        // Can't write to ROM.
    }
}
