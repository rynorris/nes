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
        let window = video.window("NES", 256 * SCALE as u32, 240 * SCALE as u32)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas()
            .accelerated()
            .build()
            .unwrap();

        let _ = canvas.set_scale(SCALE as f32, SCALE as f32);
        println!("Using SDL_Renderer \"{}\"", canvas.info().name);

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
    screen_buffer: [pixels::Color; 256 * 240],
}

impl ppu::VideoOut for Graphics {
    fn emit(&mut self, c: ppu::Colour) {
        let colour = Graphics::convert_colour(c);
        let x = self.dot;
        let y = self.scanline;
        self.screen_buffer[(x + y * 256) as usize] = colour;

        self.dot = (self.dot + 1) % 256;
        if self.dot == 0 {
            self.scanline = (self.scanline + 1) % 240;
            if self.scanline == 0 {
                self.render();
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
            screen_buffer: [pixels::Color::RGB(0, 0, 0); 256 * 240],
        }
    }

    fn render(&mut self) {
        for y in 0..240u16 {
            for x in 0..256u16 {
                let colour = self.screen_buffer[(x + y * 256) as usize];
                let point = rect::Point::new(x as i32, y as i32);
                self.io.draw_point(point, colour);
            }
        }

        self.io.flip();
    }

    fn convert_colour(c: ppu::Colour) -> pixels::Color {
        let (r, g, b) = PALETTE[c.as_byte() as usize];
        pixels::Color::RGB(r, g, b)
    }
}
