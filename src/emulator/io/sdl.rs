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
            canvas,
        }
    }

    pub fn flip(&mut self) {
        self.canvas.present();
    }

    pub fn draw_screen(&mut self, pixel_data: &[u8]) {
        let texture_creator = self.canvas.texture_creator();
        let mut texture = match texture_creator.create_texture_static(Some(pixels::PixelFormatEnum::RGB24), 256, 240) {
            Err(cause) => panic!("Failed to create texture: {}", cause),
            Ok(t) => t,
        };

        let _ = texture.update(None, pixel_data, 256 * 3);
        let _ = self.canvas.copy(&texture, None, None);
    }
}

pub struct Graphics {
    io: IO,
    scanline: u32,
    dot: u32,
    screen_buffer: [u8; 256 * 240 * 3],
    render_tile_grid: bool,
}

impl ppu::VideoOut for Graphics {
    fn emit(&mut self, c: ppu::Colour) {
        let x = self.dot;
        let y = self.scanline;

        let colour = if  self.render_tile_grid && (x % 8 == 0 || y % 8 == 0) {
            pixels::Color::RGB(255, 0, 0)
        } else {
            Graphics::convert_colour(c)
        };

        self.screen_buffer[((x + y * 256) * 3) as usize] = colour.r;
        self.screen_buffer[((x + y * 256) * 3 + 1) as usize] = colour.g;
        self.screen_buffer[((x + y * 256) * 3 + 2) as usize] = colour.b;

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
            screen_buffer: [0; 256 * 240 * 3],
            render_tile_grid: false,
        }
    }

    fn render(&mut self) {
        self.io.draw_screen(&self.screen_buffer);
        self.io.flip();
    }

    fn convert_colour(c: ppu::Colour) -> pixels::Color {
        let (r, g, b) = PALETTE[c.as_byte() as usize];
        pixels::Color::RGB(r, g, b)
    }
}
