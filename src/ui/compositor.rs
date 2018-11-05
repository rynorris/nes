use std::cell::RefCell;
use std::rc::Rc;

use emulator::io::SimpleVideoOut;
use emulator::ppu::debug::PPUDebug;
use ui::sdl2::{pixels, render, video};

const SCALE: u8 = 4;

pub struct Compositor {
    canvas: render::Canvas<video::Window>,
    nes_texture: render::Texture,
    debug_canvas: render::Canvas<video::Window>,
    debug_texture: render::Texture,

    nes_output: Rc<RefCell<SimpleVideoOut>>,
    ppu_debug: PPUDebug,
}

impl Compositor {
    pub fn new(
        video: sdl2::VideoSubsystem,
        nes_output: Rc<RefCell<SimpleVideoOut>>,
        ppu_debug: PPUDebug,
    ) -> Compositor {
        let mut main_window = video.window("NES", 256 * SCALE as u32, 240 * SCALE as u32)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        main_window.raise();

        let canvas = main_window.into_canvas()
            .accelerated()
            .build()
            .unwrap();

        let texture_creator = canvas.texture_creator();
        let nes_texture = match texture_creator.create_texture_static(Some(pixels::PixelFormatEnum::RGB24), 256, 240) {
            Err(cause) => panic!("Failed to create texture: {}", cause),
            Ok(t) => t,
        };

        let debug_window = video.window(
                "NES (Debug)",
                (PPUDebug::PATTERN_WIDTH as u32) * SCALE as u32,
                (PPUDebug::PATTERN_HEIGHT as u32) * SCALE as u32)
            .opengl()
            .build()
            .unwrap();

        let debug_canvas = debug_window.into_canvas()
            .accelerated()
            .build()
            .unwrap();

        let debug_texture_creator = debug_canvas.texture_creator();
        let debug_texture = match debug_texture_creator.create_texture_static(Some(pixels::PixelFormatEnum::RGB24), 264, 128) {
            Err(cause) => panic!("Failed to create texture: {}", cause),
            Ok(t) => t,
        };

        Compositor {
            canvas,
            nes_texture,
            debug_canvas,
            debug_texture,
            nes_output,
            ppu_debug,
        }
    }

    pub fn render(&mut self) {
        self.render_main();
        self.render_debug();
    }

    fn render_main(&mut self) {
        self.canvas.clear();
        let texture = &mut self.nes_texture;
        self.nes_output.borrow().do_render(|data| {
            let _ = texture.update(None, data, 256 * 3);
        });
        let _ = self.canvas.copy(&texture, None, None);
        self.canvas.present();
    }

    fn render_debug(&mut self) {
        self.debug_canvas.clear();
        let texture = &mut self.debug_texture;
        self.ppu_debug.do_render_pattern_tables(|data| {
            let _ = texture.update(None, data, PPUDebug::PATTERN_WIDTH * 3);
        });
        let _ = self.debug_canvas.copy(&texture, None, None);
        self.debug_canvas.present();
    }
}
