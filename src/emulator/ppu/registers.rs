use emulator::components::latch;
use emulator::ppu::PPU;
use emulator::memory::Reader;
use emulator::memory::Writer;

impl PPU {
    fn ppuaddr_increment(&self) -> u16 {
        // Increment controlled by bit 2 of PPUCTRL.
        // 0 -> increment by 1
        // 1 -> incrmement by 32
        (((self.ppuctrl & 0b100) >> 2) * 32) as u16
    }
}

impl Reader for PPU {
    fn read(&mut self, address: u16) -> u8 {
        // PPU gets mounted between 0x2000 and 0x3FFF.
        // There are only 8 registers, mirrorred every 8 bytes, so we only care about the 3 low
        // bits of the address.
        // TODO: Reads of write-only registers should return the contents of an internal latch.
        match address & 0xb111 {
            // PPUCTRL - write-only
            0 => 0,

            // PPUMASK - write-only
            1 => 0,

            // PPUSTATUS
            // Only top 3 bits contain data.
            // TODO: Bottom 5 bits should be filled from internal latch.
            2 => {
                let byte = self.ppustatus & 0b1110_0000;

                // After reading PPUSTATUS, vblank flag is cleared.
                self.ppustatus &= 0b0111_1111;
                byte
            },

            // OAMADDR - write-only
            3 => 0,

            // OAMDATA
            4 => {
                // Reads during vblank read from OAM but do not increment OAMADDR.
                if self.is_vblanking() {
                    let addr = self.oamaddr as u16;
                    self.oam.read(addr)
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
                self.v.wrapping_add(inc);
                byte
            },

            _ => panic!("Unexpected PPU register address: {}", address),
        }
    }
}

impl Writer for PPU {
    fn write(&mut self, address: u16, byte: u8) {
        match address & 0xb111 {
            // PPUCTRL
            0 => self.ppuctrl = byte,

            // PPUMASK
            1 => self.ppumask = byte,

            // PPUSTATUS - read-only
            2 => (),

            // OAMADDR
            3 => self.oamaddr = byte,

            // OAMDATA
            // Writes to OAM and increments OAMADDR.
            // Ignore during rendering.
            4 => {
                if !self.is_rendering() {
                    let addr = self.oamaddr as u16;
                    self.oam.write(addr, byte);
                    self.oamaddr.wrapping_add(1);
                }
            },

            // PPUSCROLL
            // Write 2 bytes sequentially, controlled by a latch.
            5 => {
                match self.ppuscroll_latch.get() {
                    latch::State::OFF => {
                        // First write is to X scroll.
                        // High 5 bits go to coarse X in temporary VRAM address.
                        // Low 3 bits go to fine X.
                        self.t &= 0xFFE0;
                        self.t |= (byte >> 3) as u16;
                        self.fine_x = byte & 0x03;
                    },
                    latch::State::ON => {
                        // Second write is to Y scroll.
                        // High 5 bits go to coarse Y in temporary VRAM address.
                        // Low 3 bits go to fine Y in temporary VRAM address.
                        self.t &= 0x1C1F;
                        self.t |= ((byte >> 3) as u16) << 5;
                        self.t |= ((byte & 0x03) as u16) << 12;
                    }
                }

                // Flip the latch.
                self.ppuscroll_latch.toggle();
            },

            // PPUADDR
            // Write 2 bytes sequentially to specify a 16bit address.
            // Upper byte first.
            6 => {
                match self.ppuaddr_latch.get() {
                    latch::State::OFF => {
                        // First write is the high byte.
                        // Addresses above 0x3FFF are mirrored down, so clear the top two bits
                        // always.
                        self.v &= 0xFF00;
                        self.v |= (byte as u16) << 8;
                        self.v &= 0x3FFF;
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
                self.ppuaddr_latch.toggle();
            }

            // PPUDATA
            7 => {
                // Write byte and increment VRAM address.
                self.memory.write(self.v, byte);

                // Amount to increment by is determined by PPUCTRL.
                let inc = self.ppuaddr_increment();
                self.v.wrapping_add(inc);
            },

            _ => panic!("Unexpected PPU register address: {}", address),
        }
    }
}
