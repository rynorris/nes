use emulator::memory::{Mapper, Memory};
use emulator::ppu::MirrorMode;
use emulator::state::{MapperState, MMC1State, SaveState};


// iNES Mapper 1: MMC1
// 2 switchable 16k PRG ROM banks.
// 2 switchable 4k CHR ROM banks.
// Non-switchable CHR ROM.
pub struct MMC1 {
    prg_rom: Memory,
    chr_mem: Memory,

    load_register: u8,
    write_index: u8,
    control: u8,

    prg_bank: u8,
    chr_bank_1: u8,
    chr_bank_2: u8,

    prg_offsets: [u32; 2],
    chr_offsets: [u32; 2],
}

impl MMC1 {
    pub fn new(prg_rom: Memory, chr_mem: Memory) -> MMC1 {
        let mut mapper = MMC1 {
            prg_rom,
            chr_mem,

            load_register: 0x10,
            write_index: 0,

            // 4bit0
            // -----
            // CPPMM
            // |||||
            // |||++- Mirroring (0: one-screen, lower bank; 1: one-screen, upper bank;
            // |||               2: vertical; 3: horizontal)
            // |++--- PRG ROM bank mode (0, 1: switch 32 KB at $8000, ignoring low bit of bank number;
            // |                         2: fix first bank at $8000 and switch 16 KB bank at $C000;
            // |                         3: fix last bank at $C000 and switch 16 KB bank at $8000)
            // +----- CHR ROM bank mode (0: switch 8 KB at a time; 1: switch two separate 4 KB banks)
            control: 0b0_1100,

            prg_bank: 0,
            chr_bank_1: 0,
            chr_bank_2: 0,
            prg_offsets: [0; 2],
            chr_offsets: [0; 2],
        };
        mapper.update_offsets();
        //mapper.prg_offsets[1] = mapper.prg_offset((mapper.prg_rom.len() as u32) / 0x4000 - 1);
        mapper
    }

    fn update_offsets(&mut self) {
        match (self.control & 0x0C) >> 2 {
            0 | 1 => {
                self.prg_offsets[0] = self.prg_offset((self.prg_bank as u32) & 0x0E);
                self.prg_offsets[1] = self.prg_offset(((self.prg_bank as u32) | 0x01) & 0x0F);
            },
            2 => {
                self.prg_offsets[0] = 0;
                self.prg_offsets[1] = self.prg_offset((self.prg_bank as u32) & 0x0F);
            },
            3 => {
                self.prg_offsets[0] = self.prg_offset((self.prg_bank as u32) & 0x0F);
                self.prg_offsets[1] = self.prg_offset((self.prg_rom.len() as u32) / 0x4000 - 1);
            },
            _ => panic!("Invalid prg control value: {:b}", self.control),
        }

        match (self.control & 0x10) >> 4 {
            0 => {
                self.chr_offsets[0] = self.chr_offset((self.chr_bank_1 as u32) & 0x1E);
                self.chr_offsets[1] = self.chr_offset((self.chr_bank_1 as u32) | 0x01);
            },
            1 => {
                self.chr_offsets[0] = self.chr_offset((self.chr_bank_1 as u32) & 0x1F);
                self.chr_offsets[1] = self.chr_offset((self.chr_bank_2 as u32) & 0x1F);
            },
            _ => panic!("Invalid chr control value: {:b}", self.control),
        }
    }

    fn prg_offset(&self, index: u32) -> u32 {
        (index % ((self.prg_rom.len() as u32) / 0x4000)) * 0x4000
    }

    fn chr_offset(&self, index: u32) -> u32 {
        (index % ((self.chr_mem.len() as u32) / 0x1000)) * 0x1000
    }
}

impl Mapper for MMC1 {
    fn read_chr(&mut self, address: u16) -> u8 {
        let rel = address;
        let bank = rel / 0x1000;
        let offset = rel % 0x1000;
        self.chr_mem.get((self.chr_offsets[bank as usize] + (offset as u32)) as usize)
    }

    fn write_chr(&mut self, address: u16, byte: u8) {
        let rel = address;
        let bank = rel / 0x1000;
        let offset = rel % 0x1000;
        self.chr_mem.put((self.chr_offsets[bank as usize] + (offset as u32)) as usize, byte);
    }

    fn read_prg(&mut self, address: u16) -> u8 {
        let rel = address - 0x8000;
        let bank = rel / 0x4000;
        let offset = rel % 0x4000;
        let final_addr = self.prg_offsets[bank as usize] + (offset as u32);
        self.prg_rom.get(final_addr as usize)
    }

    fn write_prg(&mut self, address: u16, byte: u8) {
        // If bit 7 is set, clear the register.
        if byte & 0x80 != 0 {
            self.load_register = 0;
            self.write_index = 0;
            return;
        }

        // Otherwise, shift bit into the register.
        self.load_register >>= 1;
        self.load_register |= (byte & 0x01) << 4;

        // On the 5th write, copy into the correct register.
        self.write_index += 1;
        if self.write_index == 5 {
            // Register is determined by bits 13 and 14 of the address written to.
            {
                let target_register = match address & 0xE000 {
                    0x8000 => &mut self.control,
                    0xA000 => &mut self.chr_bank_1,
                    0xC000 => &mut self.chr_bank_2,
                    0xE000 => &mut self.prg_bank,
                    _ => panic!("Unexpected address: ${:X}", address),
                };

                *target_register = self.load_register;
            }

            //println!("${:X} = 0b{:b}", address, self.load_register);
            self.load_register = 0;
            self.write_index = 0;
            self.update_offsets();
        }
    }

    fn mirror_mode(&self) -> MirrorMode {
        match self.control & 0x3 {
            0 => MirrorMode::SingleLower,
            1 => MirrorMode::SingleUpper,
            2 => MirrorMode::Vertical,
            3 => MirrorMode::Horizontal,
            _ => panic!("Unexpected mirror control: 0b{:b}", self.control),
        }
    }
}

impl <'de> SaveState<'de, MapperState> for MMC1 {
    fn freeze(&mut self) -> MapperState {
        MapperState::MMC1(MMC1State {
            load_register: self.load_register,
            write_index: self.write_index,
            control: self.control,
            prg_bank: self.prg_bank,
            chr_bank_1: self.chr_bank_1,
            chr_bank_2: self.chr_bank_2,
            prg_offsets: self.prg_offsets.to_vec(),
            chr_offsets: self.chr_offsets.to_vec(),
            chr_mem: self.chr_mem.freeze(),
        })
    }

    fn hydrate(&mut self, state: MapperState) {
        match state {
            MapperState::MMC1(s) => {
                self.load_register = s.load_register;
                self.write_index = s.write_index;
                self.control = s.control;
                self.prg_bank = s.prg_bank;
                self.chr_bank_1 = s.chr_bank_1;
                self.chr_bank_2 = s.chr_bank_2;
                self.prg_offsets.copy_from_slice(s.prg_offsets.as_slice());
                self.chr_offsets.copy_from_slice(s.chr_offsets.as_slice());
                self.chr_mem.hydrate(s.chr_mem);
            },
            _ => panic!("Incompatible mapper state for MMC1 mapper: {:?}", state),
        }
    }
}
