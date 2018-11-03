extern crate sdl2;

use std::rc::Rc;
use std::cell::RefCell;

use self::sdl2::event;
use self::sdl2::pixels;
use self::sdl2::render;
use self::sdl2::video;

use emulator::clock;
use emulator::io::palette::PALETTE;
use emulator::ppu;

const SCALE: u8 = 4;

pub struct IO {
    sdl_context: sdl2::Sdl,
    video: sdl2::VideoSubsystem,
    canvas: render::Canvas<video::Window>,
    screen_texture: render::Texture,

    event_pump: sdl2::EventPump,
    event_handlers: Vec<Box<dyn EventHandler>>,
}

impl IO {
    pub fn new() -> IO {
        let sdl_context = sdl2::init().unwrap();
        let video = sdl_context.video().unwrap();
        let mut window = video.window("NES", 256 * SCALE as u32, 240 * SCALE as u32)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        window.raise();

        let mut canvas = window.into_canvas()
            .accelerated()
            .build()
            .unwrap();

        let texture_creator = canvas.texture_creator();
        let screen_texture = match texture_creator.create_texture_static(Some(pixels::PixelFormatEnum::RGB24), 256, 240) {
            Err(cause) => panic!("Failed to create texture: {}", cause),
            Ok(t) => t,
        };

        let _ = canvas.set_scale(SCALE as f32, SCALE as f32);
        println!("Using SDL_Renderer \"{}\"", canvas.info().name);

        let event_pump = match sdl_context.event_pump() {
            Err(cause) => panic!("Failed to create event pump: {}", cause),
            Ok(p) => p,
        };

        IO {
            sdl_context,
            video,
            canvas,
            screen_texture,
            event_pump,
            event_handlers: vec![],
        }
    }

    pub fn flip(&mut self) {
        self.canvas.present();
    }

    pub fn draw_screen(&mut self, pixel_data: &[u8]) {
        let _ = self.screen_texture.update(None, pixel_data, 256 * 3);
        let _ = self.canvas.copy(&self.screen_texture, None, None);
    }

    pub fn process_event(&mut self, event: event::Event) {
        for mut handler in self.event_handlers.iter_mut() {
            handler.handle_event(&event);
        }
    }
}

impl clock::Ticker for IO {
    fn tick(&mut self) -> u32 {
        self.flip();
        while let Some(e) = self.event_pump.poll_event() {
            self.process_event(e);
        }
        400_000 // Shrug?  One frame ~= 100k PPU clocks ~= 400k master clock.
    }
}

pub struct Graphics {
    io: Rc<RefCell<IO>>,
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
    pub fn new(io: Rc<RefCell<IO>>) -> Graphics {
        Graphics {
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

    fn convert_colour(c: ppu::Colour) -> pixels::Color {
        let (r, g, b) = match PALETTE.get(c.as_byte() as usize) {
            None => (0, 0, 0),
            Some(colour) => *colour,
        };
        pixels::Color::RGB(r, g, b)
    }
}

pub trait EventHandler {
    fn handle_event(&mut self, &event::Event);
}
