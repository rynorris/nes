use emulator::components::latch;
use emulator::ppu::flags;
use emulator::ppu::PPU;
use emulator::memory::Reader;
use emulator::memory::Writer;

impl PPU {
    fn ppuaddr_increment(&self) -> u16 {
        // Increment controlled by bit 2 of PPUCTRL.
        // 0 -> increment by 1
        // 1 -> incrmement by 32
        if self.ppuctrl.is_set(flags::PPUCTRL::I) { 32 } else { 1 }
    }
}

impl Reader for PPU {
    fn read(&mut self, address: u16) -> u8 {
        // PPU gets mounted between 0x2000 and 0x3FFF.
        // There are only 8 registers, mirrorred every 8 bytes, so we only care about the 3 low
        // bits of the address.
        // TODO: Reads of write-only registers should return the contents of an internal latch.
        match address % 8 {
            // PPUCTRL - write-only
            0 => 0,

            // PPUMASK - write-only
            1 => 0,

            // PPUSTATUS
            // Only top 3 bits contain data.
            // TODO: Bottom 5 bits should be filled from internal latch.
            2 => {
                let byte = self.ppustatus.as_byte() & 0b1110_0000;

                // After reading PPUSTATUS, vblank flag is cleared.
                self.ppustatus.clear(flags::PPUSTATUS::V);
                byte
            },

            // OAMADDR - write-only
            3 => 0,

            // OAMDATA
            4 => {
                // Reads during vblank read from OAM but do not increment OAMADDR.
                if self.is_vblanking() {
                    let addr = self.oamaddr;
                    self.oam[addr as usize]
                } else {
                    0
                }
            },

            // PPUSCROLL - write-only
            5 => 0,

            // PPUADDR - write-only
            6 => 0,

            // PPUDATA
            7 => {
                // Note that 
                // Read from ppu memory and increment v.
                let addr = self.v;
                let byte = self.memory.read(addr);

                // Amount to increment by is determined by PPUCTRL.
                let inc = self.ppuaddr_increment();
                self.v = self.v.wrapping_add(inc);

                if addr < 0x3F00 {
                    // Reading from before palettes, buffer the read.
                    let byte_to_return = self.ppudata_read_buffer;
                    self.ppudata_read_buffer = byte;
                    byte_to_return
                } else {
                    // Reading from palettes, return immediately, but grab the nametable byte
                    // "behind" the palettes into the buffer.
                    self.ppudata_read_buffer = self.memory.read(addr & 0x2FFF);
                    byte
                }
            },

            _ => panic!("Unexpected PPU register address: {}", address),
        }
    }
}

impl Writer for PPU {
    fn write(&mut self, address: u16, byte: u8) {
        match address % 8 {
            // PPUCTRL
            0 => {
                // Load ppuctrl and also set base nametable bits in t.
                self.ppuctrl.load_byte(byte);
                self.t &= 0xF3FF;
                self.t |= ((byte & 0b11) as u16) << 10;
            },

            // PPUMASK
            1 => self.ppumask.load_byte(byte),

            // PPUSTATUS - read-only
            2 => (),

            // OAMADDR
            3 => self.oamaddr = byte,

            // OAMDATA
            // Writes to OAM and increments OAMADDR.
            // Ignore during rendering.
            4 => {
                if !self.is_rendering() {
                    let addr = self.oamaddr;
                    self.oam[addr as usize] = byte;
                    self.oamaddr = self.oamaddr.wrapping_add(1);
                }
            },

            // PPUSCROLL
            // Write 2 bytes sequentially, controlled by a latch.
            5 => {
                match self.write_latch.get() {
                    latch::State::OFF => {
                        // First write is to X scroll.
                        // High 5 bits go to coarse X in temporary VRAM address.
                        // Low 3 bits go to fine X.
                        self.t &= 0xFFE0;
                        self.t |= (byte >> 3) as u16;
                        self.fine_x = byte & 0x07;
                    },
                    latch::State::ON => {
                        // Second write is to Y scroll.
                        // High 5 bits go to coarse Y in temporary VRAM address.
                        // Low 3 bits go to fine Y in temporary VRAM address.
                        self.t &= 0x1C1F;
                        self.t |= ((byte >> 3) as u16) << 5;
                        self.t |= ((byte & 0x07) as u16) << 12;
                    }
                }

                // Flip the latch.
                self.write_latch.toggle();
            },

            // PPUADDR
            // Write 2 bytes sequentially to specify a 16bit address.
            // Upper byte first.
            6 => {
                match self.write_latch.get() {
                    latch::State::OFF => {
                        // First write is the high byte.
                        // Addresses above 0x3FFF are mirrored down, so clear the top two bits
                        // always.
                        self.t &= 0x00FF;
                        self.t |= (byte as u16) << 8;
                        self.t &= 0x3FFF;
                    },
                    latch::State::ON => {
                        // Second write is the low byte.
                        self.t &= 0xFF00;
                        self.t |= byte as u16;

                        // After the second write, t is copied to v.
                        self.v = self.t;
                    }
                }

                // Flip the latch.
                self.write_latch.toggle();
            }

            // PPUDATA
            7 => {
                // Write byte and increment VRAM address.
                self.memory.write(self.v, byte);

                // Amount to increment by is determined by PPUCTRL.
                let inc = self.ppuaddr_increment();
                self.v = self.v.wrapping_add(inc);
            },

            _ => panic!("Unexpected PPU register address: {}", address),
        }
    }
}
