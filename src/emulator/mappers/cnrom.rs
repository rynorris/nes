use emulator::memory;
use emulator::ppu::MirrorMode;

// iNES Mapper 3: CNROM
// Non-switchable PRG ROM, mirrorred to fill the space.
// Up to 4 switchable 2kb CHR ROM banks.
pub struct CNROM {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mirror_mode: MirrorMode,
    chr_bank: u8,
}

impl CNROM {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirror_mode: MirrorMode) -> CNROM {
        CNROM {
            prg_rom,
            chr_rom,
            mirror_mode,
            chr_bank: 0,
        }
    }
}

impl memory::Mapper for CNROM {
    fn read_chr(&mut self, address: u16) -> u8 {
        let base = (self.chr_bank as u16) << 13;
        self.chr_rom[(base | address) as usize]
    }

    fn write_chr(&mut self, _address: u16, _byte: u8) {
        // Can't write to ROM.
    }

    fn read_prg(&mut self, address: u16) -> u8 {
        self.prg_rom[((address - 0x8000) % self.prg_rom.len() as u16) as usize]
    }

    fn write_prg(&mut self, _address: u16, byte: u8) {
        self.chr_bank = byte & 0x03;
    }

    fn mirror_mode(&self) -> MirrorMode {
        self.mirror_mode
    }
}

