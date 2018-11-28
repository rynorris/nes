use emulator::memory::{Mapper, Memory};
use emulator::ppu::MirrorMode;
use emulator::state::{MapperState, SaveState};

// iNES Mapper 0: NROM
// Non-switchable PRG ROM, mirrorred to fill the space.
// Non-switchable CHR ROM.
pub struct NROM {
    prg_rom: Vec<u8>,
    chr_rom: Memory,
    mirror_mode: MirrorMode,
}

impl NROM {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Memory, mirror_mode: MirrorMode) -> NROM {
        NROM { prg_rom, chr_rom, mirror_mode }
    }
}

impl Mapper for NROM {
    fn read_chr(&mut self, address: u16) -> u8 {
        self.chr_rom.get(address as usize)
    }

    fn write_chr(&mut self, address: u16, byte: u8) {
        self.chr_rom.put(address as usize, byte);
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

impl <'de> SaveState<'de, MapperState> for NROM {
    fn freeze(&mut self) -> MapperState {
        MapperState::NROM
    }

    fn hydrate(&mut self, state: MapperState) {
        match state {
            MapperState::NROM => (),
            _ => panic!("Incompatible mapper state for NROM mapper: {:?}", state),
        }
    }
}
