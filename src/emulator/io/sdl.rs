extern crate sdl2;

use self::sdl2::event;
use self::sdl2::keyboard::Keycode;
use self::sdl2::pixels;
use self::sdl2::render;
use self::sdl2::video;

use emulator::clock;
use emulator::io::{Graphics, Input};
use emulator::io::event::{Event, EventHandler, Key};

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
        self.canvas.clear();
        let _ = self.canvas.copy(&self.screen_texture, None, None);
        self.canvas.present();
    }

    pub fn process_event(&mut self, event: event::Event) {
        let internal_event = convert_sdl_event_to_internal(event);

        if let Some(e) = internal_event {
            for mut handler in self.event_handlers.iter_mut() {
                handler.handle_event(e);
            }
        }
    }

}

impl Graphics for IO {
    fn draw_screen(&mut self, pixel_data: &[u8]) {
        let _ = self.screen_texture.update(None, pixel_data, 256 * 3);
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

fn convert_sdl_event_to_internal(event: event::Event) -> Option<Event> {
    match event {
        event::Event::KeyDown { keycode, .. } => keycode
            .and_then(|k| convert_sdl_keycode_to_internal(k))
            .map(|k| Event::KeyDown(k)),
        event::Event::KeyUp { keycode, .. } => keycode
            .and_then(|k| convert_sdl_keycode_to_internal(k))
            .map(|k| Event::KeyUp(k)),
        _ => None,
    }
}

fn convert_sdl_keycode_to_internal(keycode: Keycode) -> Option<Key> {
    match keycode {
        Keycode::A => Some(Key::A),
        Keycode::B => Some(Key::B),
        Keycode::C => Some(Key::C),
        Keycode::D => Some(Key::D),
        Keycode::E => Some(Key::E),
        Keycode::F => Some(Key::F),
        Keycode::G => Some(Key::G),
        Keycode::H => Some(Key::H),
        Keycode::I => Some(Key::I),
        Keycode::J => Some(Key::J),
        Keycode::K => Some(Key::K),
        Keycode::L => Some(Key::L),
        Keycode::M => Some(Key::M),
        Keycode::N => Some(Key::N),
        Keycode::O => Some(Key::O),
        Keycode::P => Some(Key::P),
        Keycode::Q => Some(Key::Q),
        Keycode::S => Some(Key::S),
        Keycode::T => Some(Key::T),
        Keycode::U => Some(Key::U),
        Keycode::V => Some(Key::V),
        Keycode::W => Some(Key::W),
        Keycode::X => Some(Key::X),
        Keycode::Y => Some(Key::Y),
        Keycode::Z => Some(Key::Z),

        Keycode::Num0 => Some(Key::Num0),
        Keycode::Num1 => Some(Key::Num1),
        Keycode::Num2 => Some(Key::Num2),
        Keycode::Num3 => Some(Key::Num3),
        Keycode::Num4 => Some(Key::Num4),
        Keycode::Num5 => Some(Key::Num5),
        Keycode::Num6 => Some(Key::Num6),
        Keycode::Num7 => Some(Key::Num7),
        Keycode::Num8 => Some(Key::Num8),
        Keycode::Num9 => Some(Key::Num9),
        Keycode::Minus => Some(Key::Minus),
        Keycode::Equals => Some(Key::Equals),

        Keycode::Up => Some(Key::Up),
        Keycode::Down => Some(Key::Down),
        Keycode::Left => Some(Key::Left),
        Keycode::Right => Some(Key::Right),

        Keycode::Escape => Some(Key::Escape),
        Keycode::Return => Some(Key::Return),
        Keycode::Tab => Some(Key::Tab),
        Keycode::Space => Some(Key::Space),

        _ => None
    }
}
