use crate::emulator::memory::{Mapper, Memory};
use crate::emulator::ppu::MirrorMode;
use crate::emulator::state::{ColorDreamsState, MapperState, SaveState};

// iNES Mapper 11: Color Dreams
// Up to 4 switchable 32kb PRG ROM banks.
// Up to 16 switchable 8kb CHR banks.
pub struct ColorDreams {
    prg_rom: Memory,
    chr_mem: Memory,
    prg_bank: u8,
    chr_bank: u8,
    mirror_mode: MirrorMode,
}

impl ColorDreams {
    pub fn new(prg_rom: Memory, chr_mem: Memory, mirror_mode: MirrorMode) -> ColorDreams {
        ColorDreams {
            prg_rom,
            chr_mem,
            prg_bank: 0,
            chr_bank: 0,
            mirror_mode,
        }
    }
}

impl Mapper for ColorDreams {
    fn read_chr(&mut self, address: u16) -> u8 {
        let base = (self.chr_bank as usize) << 13;
        let offset = address as usize;
        self.chr_mem.get((base | offset) as usize)
    }

    fn write_chr(&mut self, address: u16, byte: u8) {
        self.chr_mem.put(address as usize, byte);
    }

    fn read_prg(&mut self, address: u16) -> u8 {
        let base = (self.prg_bank as usize) << 15;
        let offset = (address & 0x7FFF) as usize;
        self.prg_rom.get(base | offset)
    }

    fn write_prg(&mut self, _address: u16, byte: u8) {
        self.prg_bank = byte & 0x3;
        self.chr_bank = (byte & 0xF0) >> 4;
    }

    fn mirror_mode(&self) -> MirrorMode {
        self.mirror_mode
    }
}

impl <'de> SaveState<'de, MapperState> for ColorDreams {
    fn freeze(&mut self) -> MapperState {
        MapperState::ColorDreams(ColorDreamsState {
            prg_bank: self.prg_bank,
            chr_bank: self.chr_bank,
            chr_mem: self.chr_mem.freeze(),
        })
    }

    fn hydrate(&mut self, state: MapperState) {
        match state {
            MapperState::ColorDreams(s) => {
                self.prg_bank = s.prg_bank;
                self.chr_bank = s.chr_bank;
                self.chr_mem.hydrate(s.chr_mem);
            },
            _ => panic!("Incompatible mapper state for ColorDreams mapper: {:?}", state),
        }
    }
}

