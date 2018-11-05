use std::cell::RefCell;
use std::rc::Rc;

use emulator::ppu::PPU;

pub struct PPUDebug {
    ppu: Rc<RefCell<PPU>>,

    // Render pattern tables.
    // Layout as two 128x128 squares with a gap of 8 in the middle.
    // => 264 x 128
    pattern_buffer: [u8; PPUDebug::PATTERN_WIDTH * PPUDebug::PATTERN_HEIGHT * 3],
}

impl PPUDebug {
    pub const PATTERN_WIDTH: usize = 264;
    pub const PATTERN_HEIGHT: usize = 128;

    pub fn new(ppu: Rc<RefCell<PPU>>) -> PPUDebug {
        PPUDebug {
            ppu,
            pattern_buffer: [0; 264 * 128 * 3],
        }
    }

    pub fn do_render_pattern_tables<F : FnOnce(&[u8]) -> ()>(&mut self, render: F) {
        self.fill_pattern_buffer();
        render(&self.pattern_buffer);
    }

    fn fill_pattern_buffer(&mut self) {
        for side in 0 .. 2 {
            for row in 0 .. 16 {
                for column in 0 .. 16 {
                    self.copy_tile(
                        0x1000 * side,
                        row,
                        column,
                        column * 8 + side * 136,
                        row * 8,
                    );
                }
            }
        }
    }

    fn copy_tile(&mut self, base: u16, row: u16, column: u16, x: u16, y: u16) {
        let mut ppu = self.ppu.borrow_mut();
        for line in 0 .. 8 {
            let low = ppu.memory.read(base | (row << 8) | (column << 4) | line);
            let high = ppu.memory.read(base | (row << 8) | (column << 4) | 0x8 | line);
            for pixel in 0 .. 8 {
                let pixel_high = (high >> (7 - pixel)) & 0x1;
                let pixel_low = (low >> (7 - pixel)) & 0x1;

                // Just generate greyscale.
                let colour = (pixel_high << 7) | (pixel_low << 6);

                let pixel_x = (x + pixel) as usize;
                let pixel_y = (y + line) as usize;
                self.pattern_buffer[(pixel_y * PPUDebug::PATTERN_WIDTH + pixel_x) * 3] = colour;
                self.pattern_buffer[(pixel_y * PPUDebug::PATTERN_WIDTH + pixel_x) * 3 + 1] = colour;
                self.pattern_buffer[(pixel_y * PPUDebug::PATTERN_WIDTH + pixel_x) * 3 + 2] = colour;
            }
        }
    }
}
