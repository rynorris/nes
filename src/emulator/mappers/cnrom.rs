use emulator::memory::{Mapper, Memory};
use emulator::ppu::MirrorMode;
use emulator::state::{CNROMState, MapperState, SaveState};

// iNES Mapper 3: CNROM
// Non-switchable PRG ROM, mirrorred to fill the space.
// Up to 4 switchable 2kb CHR ROM banks.
pub struct CNROM {
    prg_rom: Memory,
    chr_mem: Memory,
    mirror_mode: MirrorMode,
    chr_bank: u8,
}

impl CNROM {
    pub fn new(prg_rom: Memory, chr_mem: Memory, mirror_mode: MirrorMode) -> CNROM {
        CNROM {
            prg_rom,
            chr_mem,
            mirror_mode,
            chr_bank: 0,
        }
    }
}

impl Mapper for CNROM {
    fn read_chr(&mut self, address: u16) -> u8 {
        let base = (self.chr_bank as u16) << 13;
        self.chr_mem.get((base | address) as usize)
    }

    fn write_chr(&mut self, address: u16, byte: u8) {
        self.chr_mem.put(address as usize, byte);
    }

    fn read_prg(&mut self, address: u16) -> u8 {
        self.prg_rom.get(((address - 0x8000) % self.prg_rom.len() as u16) as usize)
    }

    fn write_prg(&mut self, _address: u16, byte: u8) {
        self.chr_bank = byte & 0x03;
    }

    fn mirror_mode(&self) -> MirrorMode {
        self.mirror_mode
    }
}

impl <'de> SaveState<'de, MapperState> for CNROM {
    fn freeze(&mut self) -> MapperState {
        MapperState::CNROM(CNROMState {
            chr_bank: self.chr_bank,
            chr_mem: self.chr_mem.freeze(),
        })
    }

    fn hydrate(&mut self, state: MapperState) {
        match state {
            MapperState::CNROM(s) => {
                self.chr_bank = s.chr_bank;
                self.chr_mem.hydrate(s.chr_mem);
            },
            _ => panic!("Incompatible mapper state for CNROM mapper: {:?}", state),
        }
    }
}
