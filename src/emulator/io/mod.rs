pub mod event;
pub mod nop;
pub mod palette;
pub mod sdl;

use emulator::ppu;

pub trait Graphics {
    fn draw_screen(&mut self, pixel_data: &[u8]);
}

pub struct SimpleVideoOut {
    scanline: u32,
    dot: u32,
    screen_buffer: [u8; 256 * 240 * 3],
    render_tile_grid: bool,
}

impl ppu::VideoOut for SimpleVideoOut {
    fn emit(&mut self, c: ppu::Colour) {
        let x = self.dot;
        let y = self.scanline;

        let (r, g, b) = if self.render_tile_grid && (x % 8 == 0 || y % 8 == 0) {
            (255, 0, 0)
        } else {
            palette::convert_colour(c)
        };

        self.screen_buffer[((x + y * 256) * 3) as usize] = r;
        self.screen_buffer[((x + y * 256) * 3 + 1) as usize] = g;
        self.screen_buffer[((x + y * 256) * 3 + 2) as usize] = b;

        self.dot = (self.dot + 1) % 256;
        if self.dot == 0 {
            self.scanline = (self.scanline + 1) % 240;
        }
    }
}

impl SimpleVideoOut {
    pub fn new() -> SimpleVideoOut {
        SimpleVideoOut {
            scanline: 0,
            dot: 0,
            screen_buffer: [0; 256 * 240 * 3],
            render_tile_grid: false,
        }
    }

    pub fn do_render<F : FnOnce(&[u8]) -> ()>(&self, render: F) {
        render(&self.screen_buffer);
    }
}
