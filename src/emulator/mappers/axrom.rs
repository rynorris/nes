use emulator::memory;
use emulator::ppu::MirrorMode;
use emulator::state::{AXROMState, MapperState, SaveState};

// iNES Mapper 7: AXROM
// 32kb switchable PRG ROM.
// 8kb CHR RAM.
// Selectable, sindle-screen mirroring.
pub struct AXROM {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mirror_mode: MirrorMode,
    prg_bank: u8,
}

impl AXROM {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> AXROM {
        AXROM {
            prg_rom,
            chr_rom,
            mirror_mode: MirrorMode::SingleLower,
            prg_bank: 0,
        }
    }
}

impl memory::Mapper for AXROM {
    fn read_chr(&mut self, address: u16) -> u8 {
        self.chr_rom[address as usize]
    }

    fn write_chr(&mut self, address: u16, byte: u8) {
        self.chr_rom[address as usize] = byte;
    }

    fn read_prg(&mut self, address: u16) -> u8 {
        let base = (self.prg_bank as usize) << 15;
        let rel = (address & 0x7FFF) as usize;
        self.prg_rom[(base | rel) % self.prg_rom.len()]
    }

    fn write_prg(&mut self, _address: u16, byte: u8) {
        self.prg_bank = byte & 0x7;
        self.mirror_mode = if byte & 0x10 == 0 {
            MirrorMode::SingleLower
        } else {
            MirrorMode::SingleUpper
        };
    }

    fn mirror_mode(&self) -> MirrorMode {
        self.mirror_mode
    }
}

impl <'de> SaveState<'de, MapperState> for AXROM {
    fn freeze(&mut self) -> MapperState {
        MapperState::AXROM(AXROMState {
            mirror_mode: self.mirror_mode,
            prg_bank: self.prg_bank,
            chr_ram: self.chr_rom.to_vec(),
        })
    }

    fn hydrate(&mut self, state: MapperState) {
        match state {
            MapperState::AXROM(s) => {
                self.mirror_mode = s.mirror_mode;
                self.prg_bank = s.prg_bank;
                self.chr_rom.copy_from_slice(s.chr_ram.as_slice());
            },
            _ => panic!("Incompatible mapper state for AXROM mapper: {:?}", state),
        }
    }
}
