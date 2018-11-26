use emulator::memory;
use emulator::ppu::MirrorMode;
use emulator::state::{MapperState, SaveState, UXROMState};

// iNES Mapper 2: UXROM
// 16k switchable + 16k fixed PRG ROM.
// 8kb CHR RAM.
pub struct UXROM {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mirror_mode: MirrorMode,
    prg_bank: u8,
}

impl UXROM {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirror_mode: MirrorMode) -> UXROM {
        UXROM {
            prg_rom,
            chr_rom,
            mirror_mode,
            prg_bank: 0,
        }
    }
}

impl memory::Mapper for UXROM {
    fn read_chr(&mut self, address: u16) -> u8 {
        self.chr_rom[address as usize]
    }

    fn write_chr(&mut self, address: u16, byte: u8) {
        self.chr_rom[address as usize] = byte;
    }

    fn read_prg(&mut self, address: u16) -> u8 {
        let base = if address & 0x4000 == 0 {
            (self.prg_bank as usize) << 14
        } else {
            (self.prg_rom.len() - 1) << 14
        };
        let rel = (address & 0x3FFF) as usize;
        self.prg_rom[(base | rel) % self.prg_rom.len()]
    }

    fn write_prg(&mut self, _address: u16, byte: u8) {
        self.prg_bank = byte;
    }

    fn mirror_mode(&self) -> MirrorMode {
        self.mirror_mode
    }
}

impl <'de> SaveState<'de, MapperState> for UXROM {
    fn freeze(&mut self) -> MapperState {
        MapperState::UXROM(UXROMState {
            prg_bank: self.prg_bank,
            chr_ram: self.chr_rom.to_vec(),
        })
    }

    fn hydrate(&mut self, state: MapperState) {
        match state {
            MapperState::UXROM(s) => {
                self.prg_bank = s.prg_bank;
                self.chr_rom.copy_from_slice(s.chr_ram.as_slice());
            },
            _ => panic!("Incompatible mapper state for UXROM mapper: {:?}", state),
        }
    }
}
