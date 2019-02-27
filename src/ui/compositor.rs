use emulator::components::portal::Portal;
use emulator::apu::debug::APUDebug;
use emulator::ppu::debug::PPUDebug;
use sdl2::{pixels, rect, render, video};

const SCALE: u8 = 4;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum DebugMode {
    OFF,
    PPU,
    APU,
}

pub struct Compositor {
    canvas: render::Canvas<video::Window>,
    nes_texture: render::Texture,
    debug_canvas: render::Canvas<video::Window>,
    pattern_texture: render::Texture,
    nametable_texture: render::Texture,
    sprite_texture: render::Texture,
    palette_texture: render::Texture,
    waveform_texture: render::Texture,

    nes_output: Portal<Box<[u8]>>,
    ppu_debug: PPUDebug,
    apu_debug: APUDebug,
    debug_mode: DebugMode,
}

impl Compositor {
    pub fn new(
        video: sdl2::VideoSubsystem,
        nes_output: Portal<Box<[u8]>>,
        ppu_debug: PPUDebug,
        apu_debug: APUDebug,
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
                472 * 2 as u32)
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

        let palette_texture = match debug_texture_creator.create_texture_static(Some(pixels::PixelFormatEnum::RGB24), 256, 32) {
            Err(cause) => panic!("Failed to create texture: {}", cause),
            Ok(t) => t,
        };

        let waveform_texture = match debug_texture_creator.create_texture_static(Some(pixels::PixelFormatEnum::RGB24), 256, 160) {
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
            palette_texture,
            waveform_texture,
            nes_output,
            ppu_debug,
            apu_debug,
            debug_mode: DebugMode::OFF,
        }
    }

    pub fn render(&mut self) {
        self.render_main();

        match self.debug_mode {
            DebugMode::PPU => self.render_ppu_debug(),
            DebugMode::APU => self.render_apu_debug(),
            _ => (),
        }
    }

    pub fn set_window_title(&mut self, title: &str) {
        match self.canvas.window_mut().set_title(title) {
            Err(cause) => panic!("failed to set window title: {}", cause),
            Ok(_) => (),
        };
    }

    pub fn set_debug(&mut self, mode: DebugMode) {
        if mode == self.debug_mode {
            return;
        }

        self.debug_mode = mode;
        match self.debug_mode {
            DebugMode::PPU | DebugMode::APU => self.debug_canvas.window_mut().show(),
            _ => self.debug_canvas.window_mut().hide(),
        }
    }

    fn render_main(&mut self) {
        self.canvas.clear();
        let texture = &mut self.nes_texture;
        self.nes_output.consume(|data| {
            let _ = texture.update(None, data, 256 * 3);
        });
        let _ = self.canvas.copy(&texture, None, None);
        self.canvas.present();
    }

    fn render_ppu_debug(&mut self) {
        self.debug_canvas.clear();
        let pattern_texture = &mut self.pattern_texture;
        let nametable_texture = &mut self.nametable_texture;
        let sprite_texture = &mut self.sprite_texture;
        let palette_texture = &mut self.palette_texture;

        self.ppu_debug.do_render(|buffers| {
            pattern_texture.update(None, &buffers.patterns, PPUDebug::PATTERN_WIDTH * 3).unwrap();
            nametable_texture.update(None, &buffers.nametables, PPUDebug::NAMETABLE_WIDTH * 3).unwrap();
            sprite_texture.update(None, &buffers.sprites, PPUDebug::SPRITE_WIDTH * 3).unwrap();
            palette_texture.update(None, &buffers.palettes, PPUDebug::PALETTE_WIDTH * 3).unwrap();
        });

        let _ = self.debug_canvas.copy(&pattern_texture, None, rect::Rect::new(0, 0, 256, 128));
        let _ = self.debug_canvas.copy(&nametable_texture, None, rect::Rect::new(0, 136, 256, 256));
        let _ = self.debug_canvas.copy(&sprite_texture, None, rect::Rect::new(0, 400, 256, 32));
        let _ = self.debug_canvas.copy(&palette_texture, None, rect::Rect::new(0, 440, 256, 32));
        self.debug_canvas.present();
    }

    fn render_apu_debug(&mut self) {
        self.debug_canvas.clear();
        let waveform_texture = &mut self.waveform_texture;

        self.apu_debug.do_render(
            |waveforms| waveform_texture.update(None, waveforms, APUDebug::WAVEFORM_WIDTH * 3).unwrap(),
        );

        let _ = self.debug_canvas.copy(&waveform_texture, None, rect::Rect::new(0, 0, 256, 160));
        self.debug_canvas.present();
    }
}
