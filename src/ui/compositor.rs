use std::cell::RefCell;
use std::rc::Rc;

use emulator::io::SimpleVideoOut;
use emulator::ppu::debug::PPUDebug;
use ui::sdl2::{pixels, rect, render, video};

const SCALE: u8 = 4;

pub struct Compositor {
    canvas: render::Canvas<video::Window>,
    nes_texture: render::Texture,
    debug_canvas: render::Canvas<video::Window>,
    pattern_texture: render::Texture,
    nametable_texture: render::Texture,
    sprite_texture: render::Texture,

    nes_output: Rc<RefCell<SimpleVideoOut>>,
    ppu_debug: PPUDebug,
    debug_is_on: bool,
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
                256 * 2 as u32,
                432 * 2 as u32)
            .opengl()
            .hidden()
            .build()
            .unwrap();

        let mut debug_canvas = debug_window.into_canvas()
            .accelerated()
            .build()
            .unwrap();

        debug_canvas.set_scale(2.0, 2.0).unwrap();

        let debug_texture_creator = debug_canvas.texture_creator();

        let pattern_texture = match debug_texture_creator.create_texture_static(Some(pixels::PixelFormatEnum::RGB24), 256, 128) {
            Err(cause) => panic!("Failed to create texture: {}", cause),
            Ok(t) => t,
        };

        let nametable_texture = match debug_texture_creator.create_texture_static(Some(pixels::PixelFormatEnum::RGB24), 512, 480) {
            Err(cause) => panic!("Failed to create texture: {}", cause),
            Ok(t) => t,
        };

        let sprite_texture = match debug_texture_creator.create_texture_static(Some(pixels::PixelFormatEnum::RGB24), 256, 32) {
            Err(cause) => panic!("Failed to create texture: {}", cause),
            Ok(t) => t,
        };

        Compositor {
            canvas,
            nes_texture,
            debug_canvas,
            pattern_texture,
            nametable_texture,
            sprite_texture,
            nes_output,
            ppu_debug,
            debug_is_on: false,
        }
    }

    pub fn render(&mut self) {
        self.render_main();

        if self.debug_is_on {
            self.render_debug();
        }
    }

    pub fn set_debug(&mut self, on: bool) {
        self.debug_is_on = on;
        if self.debug_is_on {
            self.debug_canvas.window_mut().show();
        } else {
            self.debug_canvas.window_mut().hide();
        }
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
        let pattern_texture = &mut self.pattern_texture;
        let nametable_texture = &mut self.nametable_texture;
        let sprite_texture = &mut self.sprite_texture;

        self.ppu_debug.do_render(
            |patterns| pattern_texture.update(None, patterns, PPUDebug::PATTERN_WIDTH * 3).unwrap(),
            |nametables| nametable_texture.update(None, nametables, PPUDebug::NAMETABLE_WIDTH * 3).unwrap(),
            |sprites| sprite_texture.update(None, sprites, PPUDebug::SPRITE_WIDTH * 3).unwrap(),
        );

        let _ = self.debug_canvas.copy(&pattern_texture, None, rect::Rect::new(0, 0, 256, 128));
        let _ = self.debug_canvas.copy(&nametable_texture, None, rect::Rect::new(0, 136, 256, 256));
        let _ = self.debug_canvas.copy(&sprite_texture, None, rect::Rect::new(0, 400, 256, 32));
        self.debug_canvas.present();
    }
}
