extern crate sdl2;

use self::sdl2::pixels;
use self::sdl2::rect;
use self::sdl2::render;
use self::sdl2::video;

use emulator::io::palette::PALETTE;
use emulator::ppu;

const SCALE: u8 = 4;

pub struct IO {
    sdl_context: sdl2::Sdl,
    video: sdl2::VideoSubsystem,
    canvas: render::Canvas<video::Window>,
}

impl IO {
    pub fn new() -> IO {
        let sdl_context = sdl2::init().unwrap();
        let video = sdl_context.video().unwrap();
        let window = video.window("NES", 256 * (SCALE as u32), 240 * (SCALE as u32))
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();

        IO {
            sdl_context,
            video,
            canvas: canvas,
        }
    }

    pub fn flip(&mut self) {
        self.canvas.present();
    }

    pub fn draw_point(&mut self, point: rect::Point, colour: pixels::Color) {
        self.canvas.set_draw_color(colour);
        match self.canvas.draw_point(point) {
            Ok(_) => (),
            Err(message) => panic!("Failed to draw pixel: {}", message),
        }
    }

    pub fn draw_rect(&mut self, rect: rect::Rect, colour: pixels::Color) {
        self.canvas.set_draw_color(colour);
        match self.canvas.fill_rect(rect) {
            Ok(_) => (),
            Err(message) => panic!("Failed to draw rect: {}", message),
        }
    }
}

pub struct Graphics {
    io: IO,
    scanline: u16,
    dot: u16,
}

impl ppu::VideoOut for Graphics {
    fn emit(&mut self, c: ppu::Colour) {
        let colour = Graphics::convert_colour(c);
        let x = (self.dot as i32) * (SCALE as i32);
        let y = (self.scanline as i32) * (SCALE as i32);
        let rect = rect::Rect::new(x, y, SCALE as u32, SCALE as u32);
        self.io.draw_rect(rect, colour);

        self.dot = (self.dot + 1) % 256;
        if self.dot == 0 {
            self.scanline = (self.scanline + 1) % 240;
            if self.scanline == 0 {
                self.io.flip();
            }
        }
    }
}

impl Graphics {
    pub fn new(io: IO) -> Graphics {
        Graphics {
            io,
            scanline: 0,
            dot: 0,
        }
    }

    fn convert_colour(c: ppu::Colour) -> pixels::Color {
        let (r, g, b) = PALETTE[c.as_byte() as usize];
        pixels::Color::RGB(r, g, b)
    }
}
