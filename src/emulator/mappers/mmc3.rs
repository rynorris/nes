use emulator::memory::Mapper;
use emulator::ppu::MirrorMode;

// 1x 8kb PRG RAM - right now we have this sram outside the mappers, so ignored here.
// 4x 8kb switchable PRG ROM
// 2x 2kb switchable CHR ROM (we will treat this as 4x 1kb)
// 4x 1kb switchable CHR ROM
// Capable of generating IRQs.
pub struct MMC3 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,

    // 8 registers for banks R0-R7, plus 2 slots which always point to the 2nd last and last PRG
    // banks.
    bank_registers: [usize; 10],
    bank_select: usize,
    prg_inversion: bool,
    chr_inversion: bool,

    irq_flag: bool,
    irq_counter: u8,
    irq_reload_flag: bool,
    irq_counter_reload: u8,
    irq_enabled: bool,

    ppu_a12: bool,
    ppu_a12_low_counter: u8,

    mirror_mode: MirrorMode,
}

impl MMC3 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> MMC3 {
        let mut m = MMC3 {
            prg_rom,
            chr_rom,
            bank_registers: [0; 10],
            bank_select: 0,
            prg_inversion: false,
            chr_inversion: false,
            irq_flag: false,
            irq_counter: 0,
            irq_reload_flag: false,
            irq_counter_reload: 0,
            irq_enabled: false,
            ppu_a12: false,
            ppu_a12_low_counter: 0,
            mirror_mode: MirrorMode::Horizontal,
        };
        let num_banks = m.prg_rom.len() / 0x2000;
        m.bank_registers[8] = ((num_banks - 2) * 0x2000) as usize;
        m.bank_registers[9] = ((num_banks - 1) * 0x2000) as usize;
        m
    }

    fn clock_irq(&mut self) {
        if self.irq_counter == 0 || self.irq_reload_flag {
            self.irq_flag = self.irq_enabled;
            self.irq_counter = self.irq_counter_reload;
            self.irq_reload_flag = false;
        } else {
            self.irq_counter = self.irq_counter.saturating_sub(1);
        }
    }
}

impl Mapper for MMC3 {
    fn read_chr(&mut self, address: u16) -> u8 {
        let (bank_ix, bank_size) = match address {
            // CHR banks.
            0x0000 ... 0x03FF => if self.chr_inversion { (2, 0x400) } else { (0, 0x800) },
            0x0400 ... 0x07FF => if self.chr_inversion { (3, 0x400) } else { (0, 0x800) },
            0x0800 ... 0x0BFF => if self.chr_inversion { (4, 0x400) } else { (1, 0x800) },
            0x0C00 ... 0x0FFF => if self.chr_inversion { (5, 0x400) } else { (1, 0x800) },
            0x1000 ... 0x13FF => if self.chr_inversion { (0, 0x800) } else { (2, 0x400) },
            0x1400 ... 0x17FF => if self.chr_inversion { (0, 0x800) } else { (3, 0x400) },
            0x1800 ... 0x1BFF => if self.chr_inversion { (1, 0x800) } else { (4, 0x400) },
            0x1C00 ... 0x1FFF => if self.chr_inversion { (1, 0x800) } else { (5, 0x400) },
            _ => panic!("Unexpected address: ${:X}", address),
        };

        let base = self.bank_registers[bank_ix];
        let offset = (address % bank_size) as usize;

        // Update A12 and clock IRQ.
        let a12 = address & 0x1000 == 0x1000;
        if a12 && !self.ppu_a12 && self.ppu_a12_low_counter > 12 {
            self.clock_irq();
        } else if !a12 && !self.ppu_a12 {
            self.ppu_a12_low_counter += 1;
        } else if a12 {
            self.ppu_a12_low_counter = 0;
        }
        self.ppu_a12 = a12;
        self.chr_rom[base + offset]
    }

    fn write_chr(&mut self, _address: u16, _byte: u8) {
        // CHR ROM not writeable.
    }

    fn read_prg(&mut self, address: u16) -> u8 {
        let (bank_ix, bank_size) = match address {
            // PRG banks.
            0x8000 ... 0x9FFF => if self.prg_inversion { (8, 0x2000) } else { (6, 0x2000) },
            0xA000 ... 0xBFFF => if self.prg_inversion { (7, 0x2000) } else { (7, 0x2000) },
            0xC000 ... 0xDFFF => if self.prg_inversion { (6, 0x2000) } else { (8, 0x2000) },
            0xE000 ... 0xFFFF => if self.prg_inversion { (9, 0x2000) } else { (9, 0x2000) },
            _ => panic!("Unexpected address: ${:X}", address),
        };

        let base = self.bank_registers[bank_ix];
        let offset = (address % bank_size) as usize;
        self.prg_rom[base + offset]
    }

    fn write_prg(&mut self, address: u16, byte: u8) {
        // The MMC3 has 4 pairs of registers at $8000-$9FFF, $A000-$BFFF, $C000-$DFFF, and $E000-$FFFF
        //   - even addresses ($8000, $8002, etc.) select the low register 
        //   - odd addresses ($8001, $8003, etc.) select the high register in each pair.
        // These can be broken into two independent functional units:
        //   - memory mapping ($8000, $8001, $A000, $A001)
        //   - scanline counting ($C000, $C001, $E000, $E001).
        println!("${:X} = 0x{:X}", address, byte);
        match address & 0xE000 {
            0x8000 => {
                if address & 0x1 == 0 {
                    // 0x8000, even => Bank select
                    self.bank_select = (byte & 0x0F) as usize;
                    self.prg_inversion = byte & 0x40 == 0x40;
                    self.chr_inversion = byte & 0x80 == 0x80;
                } else {
                    // 0x8000, odd => Bank data
                    // Handle PRG and CHR separately.
                    if self.bank_select >= 6 {
                        // PRG, 8kb banks, ignores top 2 bits.
                        self.bank_registers[self.bank_select] = ((byte & 0x7F) as usize) << 13;
                    } else if self.bank_select <= 1 {
                        // 2kb CHR banks can only select even banks.
                        self.bank_registers[self.bank_select] = ((byte & 0xFE) as usize) << 10;
                    } else {
                        self.bank_registers[self.bank_select] = (byte as usize) << 10;
                    }
                }
            },
            0xA000 => {
                self.mirror_mode = match byte & 0x1 == 0 {
                    true => MirrorMode::Vertical,
                    false => MirrorMode::Horizontal,
                }
            },
            0xC000 => {
                if address & 0x1 == 0 {
                    // 0xC000, even => IRQ Latch
                    self.irq_counter_reload = byte;
                } else {
                    // 0xC000, odd => IRQ Reload
                    self.irq_reload_flag = true;
                }
            },
            0xE000 => {
                if address & 0x1 == 0 {
                    // 0xE000, even => IRQ disable
                    self.irq_enabled = false;
                    self.irq_flag = false;
                } else {
                    // 0xE000, odd => IRQ enable
                    self.irq_enabled = true;
                }
            },

            _ => panic!("Unexpected address: ${:X}", address),
        }
    }

    fn mirror_mode(&self) -> MirrorMode {
        self.mirror_mode
    }

    fn irq_triggered(&mut self) -> bool {
        let flag = self.irq_flag;
        self.irq_flag = false;
        flag
    }
}
