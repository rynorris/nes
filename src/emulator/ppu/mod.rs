

use emulator::memory;

pub struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

pub trait VideoOut {
    fn emit(&mut self, p: Pixel);
}

pub struct PPU {
    // Device to output rendered pixels to.
    output: Box<VideoOut>,

    // PPU memory is laid out like so:
    // $0000-$0FFF = pattern table 0
    // $1000-$1FFF = pattern table 1
    // $2000-$23FF = name table 0
    // $2400-$27FF = name table 0
    // $2800-$2BFF = name table 0
    // $2C00-$2FFF = name table 0
    // $3000-$3EFF = mirrors of $2000-$2EFF
    // $3F00-$3F1F = palette RAM indexes
    // $3F20-$3FFF = mirrors of $3F00-$3F1F
    memory: memory::Manager,

    // -- Background State --

    // VRAM address.
    v: u16,

    // Temporary VRAM address.
    t: u16,

    // Fine X Scroll.
    fine_x: u8,

    // First/second write toggle.
    is_first_write: bool,

    // Two 16-bit shift registers containing bitmap data for 2 tiles.
    // Every 8 cycles the data for the next tile is loaded into the upper 8 bits of the register,
    // meanwhile the pixel to render is fetched from the lower 8 bits.
    tile_register_1: u16,
    tile_register_2: u16,

    // Two 8-bit shift registers containing the palette attributes for the lower 8 pixels of the
    // 16-bit register.
    // These registers are fed by a latch which contains the palette attribute for the next tile.
    // Every 8 cycles the latch is loaded with the attribute for the next tile.
    attribute_register_1: u8,
    attribute_register_2: u8,

    // -- Sprite State --

    // In addition to its main memory, the PPU has 256 bytes of memory known as OAM which determines how sprites are
    // rendered.
    // $00-$0C = Sprite Y coordinate
    // $01-$0D = Sprite tile #
    // $02-$0E = Sprite attribute
    // $03-$0F = Sprite X coordinate
    // TODO: What does this actually mean?
    oam: memory::Manager,

    // Secondary OAM holds 8 sprites to be rendered on the current scanline.
    secondary_oam: memory::Manager,

    // Eight pairs of 8-bit shift registers to hold the bitmap data for 8 sprites to be rendered on
    // the current scanline.

    // Eight latches containing the attribute bytes for the 8 sprites.

    // Eight counters containing the X positions for the 8 sprites.

    // --- Counters for tracking the current rendering stage.

    // There are 262 scanlines in total. 0-239 are visible, 240-260 occur durng vblank, and 261 is
    // idle.
    scanline: u16,

    // Each scanline takes 341 cycles to render.
    cycle: u16,
}

impl PPU {
    pub fn tick(&mut self) {
        match self.scanline {
            0 ... 239 | 261 => self.tick_render(),
            240 => self.tick_idle_scanline(),
            241 => self.tick_vblank(),
            _ => panic!("Scanline index should never exceed 261.  Got {}.", self.scanline),
        }

        self.cycle = (self.cycle + 1) % 341;
        if self.cycle == 0 {
            self.scanline = (self.scanline + 1) % 262;
        }
    }

    fn tick_render(&mut self) {
        match self.cycle {
            // Cycle 0 is an idle cycle.
            0 => self.tick_idle_cycle(),

            // The data for each tile is fetched durnig this phase.
            // This where the actual pixels for the scanline are output.
            1 ... 256 => self.tick_render_cycle(),

            // The tile data for the sprites on the next scanline are fetched during this phase.
            257 ... 320 => self.tick_sprite_fetch_cycle(),

            // This is where the first two tiles of the next scanline are fetched and loaded into
            // the shift registers.
            321 ... 336 => self.tick_prefetch_tiles_cycle(),

            // Finally, here two bytes are fetched, but the purpose is unknown.
            337 ... 340 => self.tick_unknown_fetch(),

            _ => panic!("PPU cycle index should never exceed 341.  Got {}.", self.cycle),
        }
    }

    fn tick_idle_scanline(&mut self) {
        // PPU does nothing on the idle scanline.
    }

    fn tick_vblank(&mut self) {
        if self.scanline == 241 && self.cycle == 1 {
            // TODO: Set VBlank flag.
        }
        // Otherwise idle.
    }

    fn tick_idle_cycle(&mut self) {
        // PPU does nothing during idle cycle.
    }

    fn tick_render_cycle(&mut self) {
    }

    fn tick_sprite_fetch_cycle(&mut self) {
    }

    fn tick_prefetch_tiles_cycle(&mut self) {
    }

    fn tick_unknown_fetch(&mut self) {
    }
}
