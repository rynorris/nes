mod background;
mod data;

use crate::emulator::memory;
use crate::emulator::memory::Writer;
use crate::emulator::ppu::{Colour, Mirrorer, MirrorMode, PPU, VideoOut};

fn new_ppu(output: Box<VideoOut>) -> PPU {
    let ppu_memory = memory::PPUMemory::new(
        Box::new(memory::Memory::new_ram(0x2000)),
        Box::new(DummyMirrorer{}),
        Box::new(memory::Memory::new_ram(0x2000)),
    );
    PPU::new(ppu_memory, output)
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

struct DummyMirrorer;

impl Mirrorer for DummyMirrorer {
    fn mirror_mode(&self) -> MirrorMode {
        MirrorMode::Horizontal
    }
}
