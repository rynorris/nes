extern crate sdl2;

use self::sdl2::event;
use self::sdl2::pixels;
use self::sdl2::render;
use self::sdl2::video;

use emulator::clock;
use emulator::io::{EventHandler, Graphics, Input};

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

    pub fn process_event(&mut self, event: event::Event) {
        for mut handler in self.event_handlers.iter_mut() {
            handler.handle_event(&event);
        }
    }

}

impl Graphics for IO {
    fn draw_screen(&mut self, pixel_data: &[u8]) {
        let _ = self.screen_texture.update(None, pixel_data, 256 * 3);
        let _ = self.canvas.copy(&self.screen_texture, None, None);
    }
}

impl Input for IO {
    fn register_event_handler(&mut self, handler: Box<dyn EventHandler>) {
        self.event_handlers.push(handler);
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

