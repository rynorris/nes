use crate::emulator::clock::Ticker;
use crate::emulator::memory::Writer;
use crate::emulator::ppu::test::ImageCapture;
use crate::emulator::ppu::test::data;
use crate::emulator::ppu::test::load_data_into_vram;
use crate::emulator::ppu::test::new_ppu;

#[test]
fn test_render_simple_background() {
    // We're going to render a background tiled with a single tile.

    // Firstly, create the PPU.
    let image = ImageCapture::new();
    let mut ppu = new_ppu(Box::new(image));

    // Load the X tile into the first position in pattern table 1.
    load_data_into_vram(&mut ppu, 0x0000, &data::TILE_X);

    // Fill the first nametable with this tile.
    // Actually don't need to do anything, since 0 is the correct value.

    // PPUMASK.  Enable background only.
    ppu.write(0x2001, 0b0000_1010);

    // Tick forward the PPU 89342 cycles (one frame).
    for _ in 0..40000 {
        ppu.tick();
    }
}
