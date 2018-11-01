mod background;
mod data;

use emulator::memory;
use emulator::memory::Writer;
use emulator::ppu::Colour;
use emulator::ppu::PPU;
use emulator::ppu::VideoOut;

fn new_ppu(output: Box<VideoOut>) -> PPU {
    PPU::new(memory::RAM::new(), output)
}

fn load_data_into_vram(ppu: &mut PPU, addr: u16, bytes: &[u8]) {
    for (ix, byte) in bytes.iter().enumerate() {
        ppu.memory.write(addr + (ix as u16), *byte);
    }
}

struct ImageCapture {
    dot: u64,
}

impl ImageCapture {
    pub fn new() -> ImageCapture {
        ImageCapture { dot: 0 }
    }
}

impl VideoOut for ImageCapture {
    fn emit(&mut self, c: Colour) {
        if self.dot % 256 < 40 {
            print!("{:03} ", c.byte);
        }
        self.dot += 1;
        if self.dot % 256 == 0 {
            println!();
        }
    }
}
