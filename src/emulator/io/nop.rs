use emulator::ppu;

pub struct Graphics;

impl ppu::VideoOut for Graphics {
    fn emit(&mut self, _: ppu::Colour) {
    }
}

impl Graphics {
    pub fn new() -> Graphics {
        Graphics {}
    }
}
