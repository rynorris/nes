use emulator::io::Graphics;

pub struct DummyGraphics;

impl Graphics for DummyGraphics {
    fn draw_screen(&mut self, _pixel_data: &[u8]) {
    }
}
