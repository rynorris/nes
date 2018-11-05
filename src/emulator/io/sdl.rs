extern crate sdl2;

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use self::sdl2::pixels;
use self::sdl2::surface;

use emulator::io::SimpleVideoOut;

pub struct ImageCapture {
    output: Rc<RefCell<SimpleVideoOut>>,
}

impl ImageCapture {
    pub fn new(output: Rc<RefCell<SimpleVideoOut>>) -> ImageCapture {
        let _ = sdl2::init().unwrap();
        ImageCapture {
            output,
        }
    }

    pub fn save_bmp(&mut self, path: &Path) {
        self.output.borrow().do_render(|buffer| {
            // Make a copy of the data so it doesn't need to be mutable.
            let mut copy = Vec::from(buffer);
            let surface = surface::Surface::from_data(
                copy.as_mut_slice(),
                256,
                240,
                256 * 3,
                pixels::PixelFormatEnum::RGB24,
            );

            let result = match surface {
                Err(cause) => panic!("Failed to create surface: {}", cause),
                Ok(s) => s.save_bmp(path),
            };

            match result {
                Err(cause) => panic!("Failed to save bmp image: {}", cause),
                Ok(_) => (),
            };
        });
    }
}
