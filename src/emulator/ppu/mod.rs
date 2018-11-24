pub mod debug;
mod flags;
mod registers;

#[cfg(test)]
mod test;

use std::cell::RefCell;
use std::rc::Rc;

use emulator::clock;
use emulator::components::bitfield::BitField;
use emulator::components::latch;
use emulator::memory::{PPUMemory, Reader};
use emulator::util;

// Colours represented as a single byte:
// 76543210
// ||||||||
// ||||++++- Hue (phase, determines NTSC/PAL chroma)
// ||++----- Value (voltage, determines NTSC/PAL luma)
// ++------- Unimplemented, reads back as 0
pub struct Colour {
    byte: u8,

    // Emphasis bits.
    pub em_r: bool,
    pub em_b: bool,
    pub em_g: bool,
}

impl Colour {
    pub fn hue(&self) -> u8 {
        self.byte & 0b1111
    }

    pub fn brightness(&self) -> u8 {
        (self.byte >> 4) & 0b11
    }

    pub fn as_byte(&self) -> u8 {
        self.byte
    }
}

pub trait VideoOut {
    fn emit(&mut self, c: Colour);
}

impl <V : VideoOut> VideoOut for Rc<RefCell<V>> {
    fn emit(&mut self, c: Colour) {
        self.borrow_mut().emit(c);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MirrorMode {
    SingleLower,
    SingleUpper,
    Vertical,
    Horizontal,
}

pub trait Mirrorer {
    fn mirror_mode(&self) -> MirrorMode;
}

pub struct PPU {
    // Device to output rendered pixels to.
    output: Box<VideoOut>,

    // --- Registers.

    // PPUCTRL
    // See note in flags.rs for detail on meaning of each bit.
    ppuctrl: BitField,

    // PPUMASK
    // See note in flags.rs for detail on meaning of each bit.
    ppumask: BitField,

    // PPUSTATUS
    // See note in flags.rs for detail on meaning of each bit.
    ppustatus: BitField,

    oamaddr: u8,
    write_latch: latch::Latch,

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
    memory: PPUMemory,

    // -- Background State --

    // VRAM address.
    v: u16,

    // Temporary VRAM address.
    t: u16,

    // Fine X Scroll.
    fine_x: u8,

    // Two 16-bit shift registers containing bitmap data for 2 tiles.
    // Every 8 cycles the data for the next tile is loaded into the upper 8 bits of the register,
    // meanwhile the pixel to render is fetched from the lower 8 bits.
    tile_register_low: u16,
    tile_register_high: u16,
    tile_latch_low: u8,
    tile_latch_high: u8,

    // Two 8-bit shift registers containing the palette attributes for the lower 8 pixels of the
    // 16-bit register.
    // These registers are fed by a latch which contains the palette attribute for the next tile.
    // Every 8 cycles the latch is loaded with the attribute for the next tile.
    attribute_register_1: u8,
    attribute_register_2: u8,
    attribute_latch_1: u8,
    attribute_latch_2: u8,

    // -- Sprite State --

    // In addition to its main memory, the PPU has 256 bytes of memory known as OAM which determines how sprites are
    // rendered.
    // $00-$0C = Sprite Y coordinate
    // $01-$0D = Sprite tile #
    // $02-$0E = Sprite attribute
    // $03-$0F = Sprite X coordinate
    oam: [u8; 256],

    // Secondary OAM holds 8 sprites to be rendered on the current scanline.
    secondary_oam: [u8; 32],

    // Eight pairs of 8-bit shift registers to hold the bitmap data for 8 sprites to be rendered on
    // the current scanline.
    sprites_tile_high: [u8; 8],
    sprites_tile_low: [u8; 8],

    // Eight latches containing the attribute bytes for the 8 sprites.
    sprites_attribute: [u8; 8],

    // Eight counters containing the X positions for the 8 sprites.
    sprites_x: [u8; 8],

    // --- Counters for tracking the current rendering stage.

    // There are 262 scanlines in total. 0-239 are visible, 240-260 occur durng vblank, and 261 is
    // idle.
    pub scanline: u16,

    // Each scanline takes 341 cycles to render.
    pub cycle: u16,

    // Every odd frame is one cycle short.
    is_odd_frame: bool,

    // -- Internal State --

    // Byte fetched from nametable indicating which tile to fetch from pattern table.
    tmp_pattern_coords: u8,

    // Byte fetched from attribute table for next tile.
    tmp_attribute_byte: u8,

    // Byte read from OAM.
    tmp_oam_byte: u8,

    // Counters.
    sprite_n: u8,
    sprite_m: u8,
    sprite_queued_copies: u8,
    sprites_copied: u8,
    sprite_eval_phase: u8,
    num_sprites: u8,
    sprite_0_next_line: bool,
    sprite_0_this_line: bool,

    // Bytes read from $2007 are delayed in this buffer.
    ppudata_read_buffer: u8,

    // Internal memory latch, causes reads from write-only registers to return the previously read
    // value.
    bus_latch: u8,
}

impl clock::Ticker for PPU {
    #[inline]
    fn tick(&mut self) -> u32 {
        self.tick_internal() as u32
    }
}

impl PPU {
    pub fn new(memory: PPUMemory, output: Box<VideoOut>) -> PPU {
        PPU {
            output: output,
            ppuctrl: BitField::new(),
            ppumask: BitField::new(),
            ppustatus: BitField::new(),
            oamaddr: 0,
            write_latch: latch::new(),
            memory,
            v: 0,
            t: 0,
            fine_x: 0,
            tile_register_low: 0,
            tile_register_high: 0,
            tile_latch_low: 0,
            tile_latch_high: 0,
            attribute_register_1: 0,
            attribute_register_2: 0,
            attribute_latch_1: 0,
            attribute_latch_2: 0,
            oam: [0; 256],
            secondary_oam: [0; 32],
            sprites_tile_high: [0; 8],
            sprites_tile_low: [0; 8],
            sprites_attribute: [0; 8],
            sprites_x: [0; 8],
            scanline: 261,
            cycle:  0,
            is_odd_frame: false,
            tmp_pattern_coords: 0,
            tmp_attribute_byte: 0,
            tmp_oam_byte: 0,
            sprite_n: 0,
            sprite_m: 0,
            sprite_queued_copies: 0,
            sprites_copied: 0,
            sprite_eval_phase: 0,
            num_sprites: 0,
            sprite_0_next_line: false,
            sprite_0_this_line: false,
            ppudata_read_buffer: 0,
            bus_latch: 0,
        }
    }

    pub fn nmi_triggered(&self) -> bool {
        self.ppustatus.is_set(flags::PPUSTATUS::V) && self.ppuctrl.is_set(flags::PPUCTRL::V)
    }

    // Returns how many PPU cycles the tick took.
    fn tick_internal(&mut self) -> u16 {
        let cycles = match self.scanline {
            0 ... 239 | 261 => self.tick_render_scanline(),
            240 => self.tick_idle_scanline(),
            241 ... 260 => self.tick_vblank_scanline(),
            _ => panic!("Scanline index should never exceed 261.  Got {}.", self.scanline),
        };

        self.cycle = self.cycle + cycles;

        if self.cycle > 341 {
            panic!("Cycle index should never exceed 341.  Got: {}.", self.cycle);
        }

        if self.cycle == 341 {
            self.cycle = 0;
            self.scanline = (self.scanline + 1) % 262;

            if self.scanline == 0 {
                self.is_odd_frame = !self.is_odd_frame;
                // Skip scanline 0, cycle 0 on odd frames, only if rendering is enabled.
                if self.is_odd_frame && self.rendering_is_enabled() {
                    self.cycle += 1;
                }
            }
        }

        cycles
    }

    fn tick_render_scanline(&mut self) -> u16 {
        // Rendering stages.
        let cycles = match self.cycle {
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
        };

        // Sprite evaluation.
        // Does not occur on the pre-render scanline or if rendering totally disabled.
        if self.scanline != 261 && self.rendering_is_enabled() {
            self.sprite_evaluation();
        }

        // Scrolling.
        self.handle_scrolling();

        // On dot 1 of the pre-render scanline, clear vblank flag and sprite overflow flag.
        if self.scanline == 261 && self.cycle == 1 {
            self.ppustatus.clear(flags::PPUSTATUS::V);
            self.ppustatus.clear(flags::PPUSTATUS::O);
            self.ppustatus.clear(flags::PPUSTATUS::S);
        }

        cycles
    }

    fn tick_idle_scanline(&mut self) -> u16 {
        // PPU does nothing on the idle scanline.
        // Just idle for 341 cycles.
        341
    }

    fn tick_vblank_scanline(&mut self) -> u16 {
        if self.scanline == 241 && self.cycle == 1 {
            // Set VBlank flag.
            self.ppustatus.set(flags::PPUSTATUS::V);
        }
        // Otherwise idle.
        if self.cycle == 0 { 1 } else { 340 }
    }

    fn tick_idle_cycle(&mut self) -> u16 {
        // PPU does nothing during idle cycle.
        1
    }

    fn tick_render_cycle(&mut self) -> u16 {
        // If cycle 1, 9, 17, ..., 257 then reload the shift registers from the latches.
        if self.cycle % 8 == 1 {
            self.reload_shift_registers();
        }

        self.fetch_tile_data();

        // Actually render and emit one pixel.
        // Unless this is scanline 261, which is just a dummy scanline.
        if self.scanline != 261 {
            let pixel = self.render_pixel();
            self.output.emit(pixel);
        }

        // Finally shift all the registers.
        self.shift_registers();
        self.shift_sprite_registers();
        1
    }

    fn tick_sprite_fetch_cycle(&mut self) -> u16 {
        // Background not doing much during these cycles.
        if self.cycle == 257 {
            self.reload_shift_registers();
        }
        1
    }

    fn tick_prefetch_tiles_cycle(&mut self) -> u16 {
        if self.cycle % 8 == 1 {
            self.reload_shift_registers();
        }

        self.fetch_tile_data();
        
        // Finally shift all the registers.
        self.shift_registers();
        1
    }

    fn tick_unknown_fetch(&mut self) -> u16 {
        // These cycles just read the next nametable byte for no reason.
        // This is used by one mapper to detect hblank, so have to include it.
        let addr = self.tile_address();
        self.tmp_pattern_coords = self.memory.read(addr);
        1
    }

    // --- FETCHING
    // Put all the memory fetching logic in one place.

    // Reload shift registers from their associated latches.
    fn reload_shift_registers(&mut self) {
        self.tile_register_low &= 0xFF00;
        self.tile_register_low |= self.tile_latch_low as u16;
        self.tile_register_high &= 0xFF00;
        self.tile_register_high |= self.tile_latch_high as u16;

        // Mux the correct bits and load into the bit-latches.
        self.attribute_latch_1 = self.tmp_attribute_byte & 1;
        self.attribute_latch_2 = (self.tmp_attribute_byte >> 1) & 1;
    }

    // Shift the registers.
    fn shift_registers(&mut self) {
        self.tile_register_low <<= 1;
        self.tile_register_high <<= 1;

        // Attribute registers pull in bits from the latch.
        self.attribute_register_1 <<= 1;
        self.attribute_register_2 <<= 1;
        self.attribute_register_1 |= self.attribute_latch_1;
        self.attribute_register_2 |= self.attribute_latch_2;
    }

    // Memory accesses for next tile data.
    fn fetch_tile_data(&mut self) {
        // We fetch 4 bytes in turn (each fetch takes 2 cycles):
        // These reads begin on cycle 1.
        match self.cycle % 8 {
            // 1. Nametable byte.
            1 => {
                let addr = self.tile_address();
                self.tmp_pattern_coords = self.memory.read(addr);
            },

            // 2. Attribute table byte.
            3 => {
                let addr = self.attribute_address();
                let shift = ((self.coarse_y_scroll() << 1) & 0b100) | (self.coarse_x_scroll() & 0b10);
                self.tmp_attribute_byte = self.memory.read(addr) >> shift;
            },

            // 3. Tile bitmap low.
            5 => {
                let addr = self.pattern_address_low();
                self.tile_latch_low = self.memory.read(addr);
            },

            // 4. Tile bitmap high.
            7 => {
                let addr = self.pattern_address_high();
                self.tile_latch_high = self.memory.read(addr);
            },

            // Do nothing on inbetween cycles.
            _ => (),
        };
    }

    // --- RENDERING
    // Put all rendering logic in one place.
    fn render_pixel(&mut self) -> Colour {
        let should_render_background = self.ppumask.is_set(flags::PPUMASK::BG)
            && (self.ppumask.is_set(flags::PPUMASK::BGL) || self.cycle > 8);
        let (bg_colour, bg_palette) = if should_render_background {
            (self.bg_colour(), self.bg_palette_index())
        } else {
            (0, 0)
        };

        let should_render_sprites = self.ppumask.is_set(flags::PPUMASK::S)
            && (self.ppumask.is_set(flags::PPUMASK::SL) || self.cycle > 8);
        let (sprite_colour, sprite_attribute, sprite_ix) = if should_render_sprites {
            self.sprite_colour()
        } else {
            (0, 0, 0)
        };

        /*  -- DEBUG RENDER --
        if bg_colour != 0 && sprite_colour != 0 && sprite_ix == 0 && self.sprite_0_this_line {
            return Colour { byte: 0x30, em_r: false, em_g: false, em_b: false };
        } else if sprite_colour != 0 && (sprite_attribute & 0x20 == 0 || bg_colour == 0) {
            // Render sprite.
            return Colour { byte: 0x2A, em_r: false, em_g: false, em_b: false };
        } else if bg_colour != 0 {
            // Render BG.
            return Colour { byte: 0x06, em_r: false, em_g: false, em_b: false };
        } else {
            // Universal BG.
            return Colour { byte: 0x00, em_r: false, em_g: false, em_b: false };
        };
        */

        // Trigger sprite 0-hit.
        // Note it does not occur if x = 255 for obscure reasons.
        if bg_colour != 0 && sprite_colour != 0 && sprite_ix == 0 && self.sprite_0_this_line  && self.cycle != 256 {
            self.ppustatus.set(flags::PPUSTATUS::S);
        }

        let colour_addr = if sprite_colour != 0 && (sprite_attribute & 0x20 == 0 || bg_colour == 0) {
            // Render sprite.
            PPU::palette_address((sprite_attribute & 0x3) | 0x04, sprite_colour)
        } else if bg_colour != 0 {
            // Render BG.
            PPU::palette_address(bg_palette, bg_colour)
        } else {
            // Universal BG.
            0x3F00
        };

        let mut colour_byte = self.memory.read(colour_addr);
        if self.ppumask.is_set(flags::PPUMASK::GR) {
            // Grescale mode.
            colour_byte &= 0x30;
        }

        Colour {
            byte: colour_byte,
            em_r: self.ppumask.is_set(flags::PPUMASK::R),
            em_g: self.ppumask.is_set(flags::PPUMASK::G),
            em_b: self.ppumask.is_set(flags::PPUMASK::B),
        }
    }

    // --- SPRITES
    fn sprite_evaluation(&mut self) {
        match self.cycle {
            0 => self.sprite_reset_state(),
            // These 2 phases do not occur on the pre-render scanline.
            1 ... 64 => if self.scanline != 261 { self.sprite_init_cycle() },
            65 ... 256 => if self.scanline != 261 { self.sprite_evaluation_cycle() },
            257 ... 320 => self.sprite_fetch_cycle(),
            _ => (),
        }
    }

    fn sprite_reset_state(&mut self) {
        // Prepare for next sprite evaluation.
        // Num sprites on this scanline is equal to however many we copied during the last
        // scanline.
        self.num_sprites = self.sprites_copied;
        self.sprite_0_this_line = self.sprite_0_next_line;
        self.sprite_0_next_line = false;
        self.tmp_oam_byte = 0;
        self.sprite_n = 0;
        self.sprite_m = 0;
        self.sprites_copied = 0;
        self.sprite_queued_copies = 0;
        self.sprite_eval_phase = 0;
    }

    fn sprite_init_cycle(&mut self) {
        if self.cycle % 2 == 0 {
            self.secondary_oam[((self.cycle / 2) - 1) as usize] = 0xFF;
        }
    }

    fn sprite_evaluation_cycle(&mut self) {
        // Just read on odd cycles.
        if self.cycle % 2 == 1 {
            self.tmp_oam_byte = self.oam[((self.sprite_n * 4) + self.sprite_m) as usize];
            return;
        }

        let sprite_height = if self.ppuctrl.is_set(flags::PPUCTRL::H) { 16 } else { 8 };
        let min_y = self.scanline.saturating_sub(sprite_height - 1);
        let max_y = self.scanline;

        match self.sprite_eval_phase {
            0 => {
                // Phase 0: Sprite copy phase.
                self.secondary_oam[((self.sprites_copied * 4) + self.sprite_m) as usize] = self.tmp_oam_byte;
                if self.sprite_queued_copies > 0 {
                    // Mid-way through a copy.  Keep going.
                    self.sprite_m += 1;
                    self.sprite_queued_copies -= 1;
                } else {
                    // Check if sprite is in range.
                    // If not then skip over it.
                    if (self.tmp_oam_byte as u16) < min_y || self.tmp_oam_byte as u16 > max_y {
                        self.sprite_n += 1;
                    } else {
                        // Track if sprite 0 is visible.
                        if self.sprite_n == 0 {
                            self.sprite_0_next_line = true;
                        }
                        self.sprite_m += 1;
                        self.sprite_queued_copies = 3;
                    }
                }

                // Handle overflows.
                if self.sprite_m >= 4 {
                    self.sprite_n += 1;
                    self.sprite_m = 0;
                    self.sprites_copied += 1;
                }

                if self.sprite_n >= 64 {
                    // We've seen all sprites.  Go to phase 2.
                    self.sprite_eval_phase = 2;
                    self.sprite_n = 0;
                }

                if self.sprites_copied == 8 {
                    // We've filled up secondary OAM.  Go to phase 1.
                    self.sprite_eval_phase = 1;
                }
            },
            1 => {
                // Phase 1: Sprite overflow.
                // Keep looping through like before, checking for overflow.
                // But this time don't write anything, and m gets incremented incorrectly.
                if self.sprite_queued_copies > 0 {
                    self.sprite_m += 1;
                    self.sprite_queued_copies -= 1;
                } else {
                    if (self.tmp_oam_byte as u16) < min_y || self.tmp_oam_byte as u16 > max_y {
                        // Erroneously increment m, causing sprite overflow bug.
                        // Note that m wraps itself here.
                        self.sprite_n += 1;
                        self.sprite_m += 1;
                        self.sprite_m %= 4;
                    } else {
                        // In range, set sprite overflow flag.
                        self.ppustatus.set(flags::PPUSTATUS::O);
                        self.sprite_m += 1;
                        self.sprite_queued_copies = 3;
                    }
                }
                //
                // Handle overflows.
                if self.sprite_m >= 4 {
                    self.sprite_n += 1;
                    self.sprite_m = 0;
                }

                if self.sprite_n >= 64 {
                    // We've seen all sprites.  Go to phase 2.
                    self.sprite_eval_phase = 2;
                    self.sprite_n = 0;
                }
            },
            2 => {
                // Phase 2: All done.
                // Do nothing.
            },
            _ => panic!("Unexpected sprite eval phase: {}", self.sprite_eval_phase),
        }
    }

    fn sprite_fetch_cycle(&mut self) {
        // Loading the sprite data for next scanline into registers.
        // Technically this data should be handled 1 byte per cycle.
        // But because we're only shuffling data around internally, it's impossible
        // for anything weird to happen inbetween, so just do it in one go.
        if self.cycle % 8 != 1 {
            return;
        }

        let sprite_ix = (self.cycle - 256) / 8;
        let y = self.secondary_oam[(sprite_ix * 4) as usize];
        let tile_no = self.secondary_oam[(sprite_ix * 4 + 1) as usize];
        let attribute = self.secondary_oam[(sprite_ix * 4 + 2) as usize];
        let x = self.secondary_oam[(sprite_ix * 4 + 3) as usize];

        // 8x16 sprites?
        let tall_sprites = self.ppuctrl.is_set(flags::PPUCTRL::H);

        let (pattern_table_base, mut tile_index) = match tall_sprites {
            // Set = 8x16 mode, decided by bit 0.
            true => (((tile_no as u16) & 1) << 12, tile_no & 0xFE),

            // Unset = 8x8 mode, table decided by S flag.
            false => (if self.ppuctrl.is_set(flags::PPUCTRL::S) { 0x1000 } else { 0x0000 }, tile_no),
        };

        let mut offset = self.scanline.saturating_sub(y as u16);

        if attribute & 0x80 != 0 {
            // Vertical flip.
            // In 8x16 mode have to flip top and bottom sprite also.
            offset = match tall_sprites {
                true => 15u16.saturating_sub(offset),
                false => 7u16.saturating_sub(offset),
            };
        }

        if offset >= 8 {
            // Lower half of a tall sprite, jump to the next tile.
            tile_index += 1;
            offset -= 8;
        }

        let tile_addr_low = pattern_table_base | ((tile_index as u16) << 4) | offset;
        let tile_addr_high = tile_addr_low | 0b1000 | offset;

        let mut tile_byte_low = self.memory.read(tile_addr_low);
        let mut tile_byte_high = self.memory.read(tile_addr_high);

        if attribute & 0x40 != 0 {
            // Horizontal flip.
            tile_byte_low = util::reverse_bits(tile_byte_low);
            tile_byte_high = util::reverse_bits(tile_byte_high);
        }

        self.sprites_tile_low[sprite_ix as usize] = tile_byte_low;
        self.sprites_tile_high[sprite_ix as usize] = tile_byte_high;
        self.sprites_attribute[sprite_ix as usize] = attribute;
        self.sprites_x[sprite_ix as usize] = x;
    }

    // --- SCROLLING
    // Put all scrolling logic in one place.
    fn handle_scrolling(&mut self) {
        // No scrolling happens if rendering is disabled.
        if !self.rendering_is_enabled() {
            return;
        }

        // If rendering is enabled, on dot 256 of each scanline, the PPU increments y position.
        if self.cycle == 256 {
            self.increment_y();
        }

        // If rendering is enabled, on dot 257 of each scanline, copy all horizontal bits from t to v.
        if self.cycle == 257 {
            let horizontal_bitmask = 0b0000100_00011111;
            self.v &= !horizontal_bitmask;
            self.v |= self.t & horizontal_bitmask;
        }

        // If rendering is enabled, between dots 280 to 304 of the pre-render scanline, the PPU repeatedly copies the
        // vertical bits from t to v.
        if self.scanline == 261 && self.cycle >= 280 && self.cycle <= 304 {
            let vertical_bitmask = 0b1111011_11100000;
            self.v = self.v & !vertical_bitmask;
            self.v = self.v | (self.t & vertical_bitmask);
        }

        // Between dot 328 of a scanline, and 256 of the next scanline, x scroll is incremented
        // on every multiple of 8 dots except 0.  i.e. 328, 336, 8, 16, ..., 256.
        if ((self.cycle > 0 && self.cycle <= 256) || self.cycle >= 328) && (self.cycle % 8 == 0) {
            self.increment_coarse_x();
        }
    }

    // During rendering the VRAM address v is laid out like so:
    // yyy NN YYYYY XXXXX
    // ||| || ||||| +++++-- coarse X scroll
    // ||| || +++++-------- coarse Y scroll
    // ||| ++-------------- nametable select
    // +++----------------- fine Y scroll
    //
    // Here are some convenience methods to pull out these values.
    fn fine_y_scroll(&self) -> u16 {
        ((self.v >> 12) & 0b111) as u16
    }

    fn nametable_select(&self) -> u16 {
        ((self.v >> 10) & 0b11) as u16
    }

    fn coarse_y_scroll(&self) -> u16 {
        ((self.v >> 5) & 0b11111) as u16
    }

    fn coarse_x_scroll(&self) -> u16 {
        (self.v & 0b11111) as u16
    }

    // Scrolling is complex, so split out the logic here.
    fn increment_coarse_x(&mut self) {
        if self.coarse_x_scroll() == 31 {
            self.v &= !0x001F;  // Coarse X = 0.
            self.v ^= 0x0400;  // Switch horizontal nametable.
        } else {
            self.v += 1;  // Increment coarse X.
        }
    }

    fn increment_y(&mut self) {
        if self.fine_y_scroll() < 7 {
            self.v += 0x1000;  // Increment fine Y.
        } else {
            self.v &= !0x7000;  // Fine Y = 0.
            let mut coarse_y = self.coarse_y_scroll();
            if coarse_y == 29 {
                coarse_y = 0;
                self.v ^= 0x0800;  // Switch vertical nametable.
            } else if coarse_y == 31 {
                coarse_y = 0;
            } else {
                coarse_y += 1;
            }

            self.v = (self.v & !0x03E0) | ((coarse_y as u16) << 5);  // Put coarse_y back into v.
        }
    }

    // And then methods to load the tile and attribute addresses to load next.
    fn tile_address(&self) -> u16 {
        0x2000 | (self.v & 0x0FFF)
    }

    fn attribute_address(&self) -> u16 {
        0x23C0  // Attribute table base.
            | (self.v & 0x0C00)  // Select nametable.
            | ((self.coarse_y_scroll() << 1) & 0b111000)  // Y component.
            | ((self.coarse_x_scroll() >> 2) & 0b111)  // X component.
    }

    fn pattern_address_low(&self) -> u16 {
        (if self.ppuctrl.is_set(flags::PPUCTRL::B) { 1 << 12 } else { 0 })  // Left or right half of sprite table.
            | ((self.tmp_pattern_coords as u16) << 4)  // Tile coordinates.
            | 0b0000  // Lower bit plane.
            | self.fine_y_scroll()  // Fine Y offset.
    }

    fn pattern_address_high(&self) -> u16 {
        (if self.ppuctrl.is_set(flags::PPUCTRL::B) { 1 << 12 } else { 0 })  // Left or right half of sprite table.
            | ((self.tmp_pattern_coords as u16) << 4)  // Tile coordinates.
            | 0b1000  // Upper bit plane.
            | self.fine_y_scroll()  // Fine Y offset.
    }

    fn bg_palette_index(&self) -> u8 {
        let low = (self.attribute_register_1 >> (7 - self.fine_x)) & 1;
        let high = (self.attribute_register_2 >> (7 - self.fine_x)) & 1;
        (high << 1) | low
    }

    fn bg_colour(&self) -> u8 {
        let bg_low_bit = (self.tile_register_low >> (15 - self.fine_x)) & 1;
        let bg_high_bit = (self.tile_register_high >> (15 - self.fine_x)) & 1;

        ((bg_high_bit << 1) | bg_low_bit) as u8
    }

    fn sprite_colour(&self) -> (u8, u8, u8) {
        for ix in 0 .. self.num_sprites {
            // Don't consider inactive sprites.
            if self.sprites_x[ix as usize] > 0 {
                continue;
            }

            let colour_high = self.sprites_tile_high[ix as usize] >> 7;
            let colour_low = self.sprites_tile_low[ix as usize] >> 7;
            let sprite_colour = (colour_high << 1) | colour_low;
            let sprite_attribute = self.sprites_attribute[ix as usize];
            if sprite_colour != 0 {
                return (sprite_colour, sprite_attribute, ix);
            }
        }
        (0, 0, 0)
    }

    fn shift_sprite_registers(&mut self) {
        for ix in 0 .. 8 {
            // Decrement x if non-zero, otherwise shift tiles.
            if self.sprites_x[ix] > 0 {
                self.sprites_x[ix] -= 1;
            } else {
                self.sprites_tile_high[ix] <<= 1;
                self.sprites_tile_low[ix] <<= 1;
            }
        }
    }

    fn palette_address(index: u8, colour: u8) -> u16 {
        0x3F00  // Palette memory.
            | ((index << 2) as u16) // Palette select.
            | (colour as u16)  // Colour select.
    }

    // Utility methods to query internal state.
    fn rendering_is_enabled(&self) -> bool {
        self.ppumask.is_set(flags::PPUMASK::S) || self.ppumask.is_set(flags::PPUMASK::BG)
    }

    fn is_vblanking(&self) -> bool {
        self.scanline >= 241
    }

    fn is_rendering(&self) -> bool {
        !self.is_vblanking() && self.rendering_is_enabled()
    }
}
