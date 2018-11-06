use std::cell::RefCell;
use std::rc::Rc;

use emulator::ppu::flags;
use emulator::ppu::PPU;

pub struct PPUDebug {
    ppu: Rc<RefCell<PPU>>,

    // Copy of pattern memory.
    pattern_tables: [u8; 0x2000],

    // Render pattern tables.
    // Layout as two 128x128 squares with a gap of 8 in the middle.
    // => 264 x 128
    pattern_buffer: [u8; PPUDebug::PATTERN_WIDTH * PPUDebug::PATTERN_HEIGHT * 3],
    nametable_buffer: [u8; PPUDebug::NAMETABLE_WIDTH * PPUDebug::NAMETABLE_HEIGHT * 3],
}

impl PPUDebug {
    pub const PATTERN_WIDTH: usize = 256;
    pub const PATTERN_HEIGHT: usize = 128;
    pub const NAMETABLE_WIDTH: usize = 256 * 2;
    pub const NAMETABLE_HEIGHT: usize = 240 * 2;

    pub fn new(ppu: Rc<RefCell<PPU>>) -> PPUDebug {
        PPUDebug {
            ppu,
            pattern_tables: [0; 0x2000],

            pattern_buffer: [0; PPUDebug::PATTERN_WIDTH * PPUDebug::PATTERN_HEIGHT * 3],
            nametable_buffer: [0; PPUDebug::NAMETABLE_WIDTH * PPUDebug::NAMETABLE_HEIGHT * 3],
        }
    }

    pub fn do_render_pattern_tables<F : FnOnce(&[u8]) -> ()>(&mut self, render: F) {
        self.fill_pattern_buffer();
        render(&self.pattern_buffer);
    }

    pub fn do_render_nametables<F : FnOnce(&[u8]) -> ()>(&mut self, render: F) {
        self.fill_nametable_buffer();
        render(&self.nametable_buffer);
    }

    pub fn hydrate_pattern_tables(&mut self) {
        let mut ppu = self.ppu.borrow_mut();
        for ix in 0 .. 0x2000 {
            self.pattern_tables[ix] = ppu.memory.read(ix as u16);
        }
    }

    fn fill_pattern_buffer(&mut self) {
        let source = &self.pattern_tables;
        let target = &mut self.pattern_buffer;
        for side in 0 .. 2 {
            for row in 0 .. 16 {
                for column in 0 .. 16 {
                    PPUDebug::copy_tile(
                        0x1000 * side,
                        row,
                        column,
                        column * 8 + side * 128,
                        row * 8,
                        source,
                        target,
                        PPUDebug::PATTERN_WIDTH as u16,
                    );
                }
            }
        }
    }

    fn fill_nametable_buffer(&mut self) {
        let mut ppu = self.ppu.borrow_mut();
        let source = &self.pattern_tables;
        let target = &mut self.nametable_buffer;
        let side = if ppu.ppuctrl.is_set(flags::PPUCTRL::B) { 1 } else { 0 };
        for table in 0 .. 4 {
            for row in 0 .. 30 {
                for column in 0 .. 32 {
                    let nt_addr = 0x2000 | (table << 10) | (row << 5) | column;
                    let nt_byte = ppu.memory.read(nt_addr);
                    PPUDebug::copy_tile(
                        0x1000 * side,
                        (nt_byte >> 4) as u16,
                        (nt_byte & 0xF) as u16,
                        ((table % 2) * 256) + column * 8,
                        ((table / 2) * 240) + row * 8,
                        source,
                        target,
                        PPUDebug::NAMETABLE_WIDTH as u16,
                    );
                }
            }
        }
    }

    fn copy_tile(base: u16, row: u16, column: u16, x: u16, y: u16,
                 source: &[u8], target: &mut [u8], pitch: u16) {
        for line in 0 .. 8 {
            let low = source[(base | (row << 8) | (column << 4) | line) as usize];
            let high = source[(base | (row << 8) | (column << 4) | 0x8 | line) as usize];
            for pixel in 0 .. 8 {
                let pixel_high = (high >> (7 - pixel)) & 0x1;
                let pixel_low = (low >> (7 - pixel)) & 0x1;

                // Just generate greyscale.
                let colour = (pixel_high << 7) | (pixel_low << 6);

                let pixel_x = (x + pixel) as usize;
                let pixel_y = (y + line) as usize;
                target[(pixel_y * (pitch as usize) + pixel_x) * 3] = colour;
                target[(pixel_y * (pitch as usize) + pixel_x) * 3 + 1] = colour;
                target[(pixel_y * (pitch as usize) + pixel_x) * 3 + 2] = colour;
            }
        }
    }
}
