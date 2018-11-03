pub mod event;
pub mod nop;
pub mod palette;
pub mod sdl;

use std::cell::RefCell;
use std::rc::Rc;

use emulator::io::event::EventHandler;
use emulator::io::palette::PALETTE;
use emulator::ppu;

pub trait Input {
    fn register_event_handler(&mut self, handler: Box<dyn EventHandler>);
}

pub trait Graphics {
    fn draw_screen(&mut self, pixel_data: &[u8]);
}

pub struct SimpleVideoOut {
    io: Rc<RefCell<dyn Graphics>>,
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
            SimpleVideoOut::convert_colour(c)
        };

        self.screen_buffer[((x + y * 256) * 3) as usize] = r;
        self.screen_buffer[((x + y * 256) * 3 + 1) as usize] = g;
        self.screen_buffer[((x + y * 256) * 3 + 2) as usize] = b;

        self.dot = (self.dot + 1) % 256;
        if self.dot == 0 {
            self.scanline = (self.scanline + 1) % 240;
            if self.scanline == 0 {
                self.render();
            }
        }
    }
}

impl SimpleVideoOut {
    pub fn new(io: Rc<RefCell<Graphics>>) -> SimpleVideoOut {
        SimpleVideoOut {
            io,
            scanline: 0,
            dot: 0,
            screen_buffer: [0; 256 * 240 * 3],
            render_tile_grid: false,
        }
    }

    fn render(&mut self) {
        self.io.borrow_mut().draw_screen(&self.screen_buffer);
    }

    fn convert_colour(c: ppu::Colour) -> (u8, u8, u8) {
        let (r, g, b) = match PALETTE.get(c.as_byte() as usize) {
            None => (0, 0, 0),
            Some(colour) => *colour,
        };
        (r, g, b)
    }
}
