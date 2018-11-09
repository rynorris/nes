use emulator::memory;
use emulator::ppu::MirrorMode;

// iNES Mapper 0: NROM
// Non-switchable PRG ROM, mirrorred to fill the space.
// Non-switchable CHR ROM.
pub struct NROM {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mirror_mode: MirrorMode,
}

impl NROM {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirror_mode: MirrorMode) -> NROM {
        NROM { prg_rom, chr_rom, mirror_mode }
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
        self.prg_rom[((address - 0x8000) % self.prg_rom.len() as u16) as usize]
    }

    fn write_prg(&mut self, _address: u16, _byte: u8) {
        // Can't write to ROM.
    }

    fn mirror_mode(&self) -> MirrorMode {
        self.mirror_mode
    }
}
