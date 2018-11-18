use std::cell::RefCell;
use std::rc::Rc;

use emulator::io::palette;
use emulator::memory::Reader;
use emulator::ppu::flags;
use emulator::ppu::{Colour, PPU};

pub struct PPUDebug {
    ppu: Rc<RefCell<PPU>>,
}

impl PPUDebug {
    pub const PATTERN_WIDTH: usize = 256;
    pub const PATTERN_HEIGHT: usize = 128;
    pub const NAMETABLE_WIDTH: usize = 256 * 2;
    pub const NAMETABLE_HEIGHT: usize = 240 * 2;
    pub const SPRITE_WIDTH: usize = 256;
    pub const SPRITE_HEIGHT: usize = 32;
    pub const PALETTE_WIDTH: usize = 256;
    pub const PALETTE_HEIGHT: usize = 32;

    pub fn new(ppu: Rc<RefCell<PPU>>) -> PPUDebug {
        PPUDebug {
            ppu,
        }
    }

    pub fn do_render<F, G, H, I>(&mut self, render_pattern: F, render_nametable: G, render_sprites: H, render_palette: I) where
        F : FnOnce(&[u8]) -> (),
        G : FnOnce(&[u8]) -> (),
        H : FnOnce(&[u8]) -> (),
        I : FnOnce(&[u8]) -> () {

        let mut pattern_tables = [0; 0x2000];
        self.hydrate_pattern_tables(&mut pattern_tables);

        let mut pattern_buffer = [0; PPUDebug::PATTERN_WIDTH * PPUDebug::PATTERN_HEIGHT * 3];
        let mut nametable_buffer = [0; PPUDebug::NAMETABLE_WIDTH * PPUDebug::NAMETABLE_HEIGHT * 3];
        let mut sprite_buffer = [0; PPUDebug::SPRITE_WIDTH * PPUDebug::SPRITE_HEIGHT * 3];
        let mut palette_buffer = [0; PPUDebug::PALETTE_WIDTH * PPUDebug::PALETTE_HEIGHT * 3];

        PPUDebug::fill_pattern_buffer(&mut pattern_buffer, &pattern_tables);
        PPUDebug::fill_nametable_buffer(self.ppu.clone(), &mut nametable_buffer, &pattern_tables);
        PPUDebug::fill_sprite_buffer(self.ppu.clone(), &mut sprite_buffer, &pattern_tables);
        PPUDebug::fill_palette_buffer(self.ppu.clone(), &mut palette_buffer);
        render_pattern(&pattern_buffer);
        render_nametable(&nametable_buffer);
        render_sprites(&sprite_buffer);
        render_palette(&palette_buffer);
    }

    fn hydrate_pattern_tables(&mut self, target: &mut[u8]) {
        let mut ppu = self.ppu.borrow_mut();
        for ix in 0 .. 0x2000 {
            target[ix] = ppu.memory.read(ix as u16);
        }
    }

    fn fill_pattern_buffer(buffer: &mut[u8], pattern_tables: &[u8]) {
        for side in 0 .. 2 {
            for row in 0 .. 16 {
                for column in 0 .. 16 {
                    PPUDebug::copy_tile(
                        0x1000 * side,
                        row,
                        column,
                        column * 8 + side * 128,
                        row * 8,
                        pattern_tables,
                        buffer,
                        PPUDebug::PATTERN_WIDTH as u16,
                        0,
                    );
                }
            }
        }
    }

    fn fill_nametable_buffer(ppu_cell: Rc<RefCell<PPU>>, buffer: &mut[u8], pattern_tables: &[u8]) {
        let mut ppu = ppu_cell.borrow_mut();
        let side = if ppu.ppuctrl.is_set(flags::PPUCTRL::B) { 1 } else { 0 };
        for table in 0 .. 4 {
            for row in 0 .. 30 {
                for column in 0 .. 32 {
                    let nt_addr = 0x2000 | (table << 10) | (row << 5) | column;
                    let nt_byte = ppu.memory.read(nt_addr);
                    let attribute_addr = 0x23C0 | (table << 10) | ((row >> 2) << 3) | (column >> 2);
                    let attribute_byte = ppu.memory.read(attribute_addr);
                    let attr_shift = ((row << 1) & 0x4) | (column & 0x2);
                    let palette_ix = (attribute_byte >> attr_shift) & 0x3;
                    PPUDebug::copy_tile(
                        0x1000 * side,
                        (nt_byte >> 4) as u16,
                        (nt_byte & 0xF) as u16,
                        ((table % 2) * 256) + column * 8,
                        ((table / 2) * 240) + row * 8,
                        pattern_tables,
                        buffer,
                        PPUDebug::NAMETABLE_WIDTH as u16,
                        palette_ix,
                    );
                }
            }
        }
    }

    fn fill_sprite_buffer(ppu_cell: Rc<RefCell<PPU>>, buffer: &mut[u8], pattern_tables: &[u8]) {
        let ppu = ppu_cell.borrow_mut();
        for sprite_ix in 0 .. 64 {
            let tile_byte = ppu.oam[(sprite_ix + 1) as usize];
            let tall_sprites = ppu.ppuctrl.is_set(flags::PPUCTRL::H);
            let (base, tile_ix) = match tall_sprites {
                // Tall sprites.
                true => (((tile_byte as u16) & 1) << 12, tile_byte & 0xFE),
                
                // Normal sprites.
                false => (if ppu.ppuctrl.is_set(flags::PPUCTRL::S) { 0x1000 } else { 0x0000 }, tile_byte),
            };
            PPUDebug::copy_tile(
                base,
                (tile_ix >> 4) as u16,
                (tile_ix & 0xF) as u16,
                (sprite_ix % 32) * 8,
                (sprite_ix / 32) * 16,
                pattern_tables,
                buffer,
                PPUDebug::SPRITE_WIDTH as u16,
                0
            );

            if tall_sprites {
                PPUDebug::copy_tile(
                    base,
                    (tile_ix >> 4) as u16,
                    (tile_ix & 0xF) as u16 + 1,
                    (sprite_ix % 32) * 8,
                    (sprite_ix / 32) * 16 + 8,
                    pattern_tables,
                    buffer,
                    PPUDebug::SPRITE_WIDTH as u16,
                    0
                );
            }
        }
    }

    fn fill_palette_buffer(ppu_cell: Rc<RefCell<PPU>>, buffer: &mut[u8]) {
        let mut ppu = ppu_cell.borrow_mut();
        for palette_ix in 0 .. 8 {
            for colour_ix in 0 .. 4 {
                let addr = 0x3F00 | (palette_ix << 2) | colour_ix;
                let colour = Colour {
                    byte: ppu.memory.read(addr),
                    em_r: false,
                    em_g: false,
                    em_b: false,
                };
                PPUDebug::fill_rect(
                    buffer,
                    256,
                    (((palette_ix * 4) + colour_ix) % 16) * 16,
                    (((palette_ix * 4) + colour_ix) / 16) * 16,
                    16,
                    16,
                    colour
                );
            }
        }
    }

    fn copy_tile(base: u16, row: u16, column: u16, x: u16, y: u16,
                 source: &[u8], target: &mut [u8], pitch: u16,
                 palette_ix: u8) {
        for line in 0 .. 8 {
            let low = source[(base | (row << 8) | (column << 4) | line) as usize];
            let high = source[(base | (row << 8) | (column << 4) | 0x8 | line) as usize];
            for pixel in 0 .. 8 {
                let pixel_high = (high >> (7 - pixel)) & 0x1;
                let pixel_low = (low >> (7 - pixel)) & 0x1;

                // Palette 0 = greyscale, 1 = R, 2 = G, 3 = B.
                let colour = (pixel_high << 7) | (pixel_low << 6);
                let r = if palette_ix == 0 || palette_ix == 1 { colour } else { 0 };
                let g = if palette_ix == 0 || palette_ix == 2 { colour } else { 0 };
                let b = if palette_ix == 0 || palette_ix == 3 { colour } else { 0 };

                let pixel_x = (x + pixel) as usize;
                let pixel_y = (y + line) as usize;
                target[(pixel_y * (pitch as usize) + pixel_x) * 3] = r;
                target[(pixel_y * (pitch as usize) + pixel_x) * 3 + 1] = g;
                target[(pixel_y * (pitch as usize) + pixel_x) * 3 + 2] = b;
            }
        }
    }

    fn fill_rect(target: &mut [u8], pitch: u16, x: u16, y: u16, width: u16, height: u16, colour: Colour) {
        let (r, g, b) = palette::convert_colour(colour);
        for row in 0 .. height {
            for col in 0 .. width {
                target[(((y + row) * pitch + (x + col)) * 3) as usize] = r;
                target[((((y + row) * pitch + (x + col)) * 3) + 1) as usize] = g;
                target[((((y + row) * pitch + (x + col)) * 3) + 2) as usize] = b;
            }
        }
    }
}
