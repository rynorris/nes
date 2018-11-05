use std::cell::RefCell;
use std::rc::Rc;

use emulator::io::SimpleVideoOut;
use ui::sdl2::{pixels, render, video};

const SCALE: u8 = 4;

pub struct Compositor {
    canvas: render::Canvas<video::Window>,
    nes_texture: render::Texture,
    nes_output: Rc<RefCell<SimpleVideoOut>>,
}

impl Compositor {
    pub fn new(
        video: sdl2::VideoSubsystem,
        nes_output: Rc<RefCell<SimpleVideoOut>>,
    ) -> Compositor {
        let mut window = video.window("NES", 256 * SCALE as u32, 240 * SCALE as u32)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        window.raise();

        let canvas = window.into_canvas()
            .accelerated()
            .build()
            .unwrap();

        let texture_creator = canvas.texture_creator();
        let nes_texture = match texture_creator.create_texture_static(Some(pixels::PixelFormatEnum::RGB24), 256, 240) {
            Err(cause) => panic!("Failed to create texture: {}", cause),
            Ok(t) => t,
        };

        Compositor {
            canvas,
            nes_texture,
            nes_output,
        }
    }

    pub fn render(&mut self) {
        self.canvas.clear();
        let texture = &mut self.nes_texture;
        self.nes_output.borrow().do_render(|data| {
            let _ = texture.update(None, data, 256 * 3);
        });
        let _ = self.canvas.copy(&texture, None, None);
        self.canvas.present();
    }
}
