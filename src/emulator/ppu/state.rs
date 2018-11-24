use emulator::ppu::PPU;
use emulator::state::{PPUState, SaveState};

impl <'de> SaveState<'de, PPUState> for PPU {
    fn freeze(&mut self) -> PPUState {
        PPUState {
            ppuctrl: self.ppuctrl.as_byte(),
            ppumask: self.ppumask.as_byte(),
            ppustatus: self.ppustatus.as_byte(),
            oamaddr: self.oamaddr,
            write_latch: self.write_latch.as_bool(),
            v: self.v,
            t: self.t,
            fine_x: self.fine_x,
            tile_register_low: self.tile_register_low,
            tile_register_high: self.tile_register_high,
            tile_latch_low: self.tile_latch_low,
            tile_latch_high: self.tile_latch_high,
            attribute_register_1: self.attribute_register_1,
            attribute_register_2: self.attribute_register_2,
            attribute_latch_1: self.attribute_latch_1,
            attribute_latch_2: self.attribute_latch_2,
            oam: self.oam.to_vec(),
            secondary_oam: self.secondary_oam.to_vec(),
            sprites_tile_high: self.sprites_tile_high.to_vec(),
            sprites_tile_low: self.sprites_tile_low.to_vec(),
            sprites_attribute: self.sprites_attribute.to_vec(),
            sprites_x: self.sprites_x.to_vec(),
            scanline: self.scanline,
            cycle: self.cycle,
            is_odd_frame: self.is_odd_frame,
            tmp_pattern_coords: self.tmp_pattern_coords,
            tmp_attribute_byte: self.tmp_attribute_byte,
            tmp_oam_byte: self.tmp_oam_byte,
            sprite_n: self.sprite_n,
            sprite_m: self.sprite_m,
            sprite_queued_copies: self.sprite_queued_copies,
            sprites_copied: self.sprites_copied,
            sprite_eval_phase: self.sprite_eval_phase,
            num_sprites: self.num_sprites,
            sprite_0_next_line: self.sprite_0_next_line,
            sprite_0_this_line: self.sprite_0_this_line,
            ppudata_read_buffer: self.ppudata_read_buffer,
            bus_latch: self.bus_latch,
        }
    }

    fn hydrate(&mut self, state: PPUState) {
        self.ppuctrl.load_byte(state.ppuctrl);
        self.ppumask.load_byte(state.ppumask);
        self.ppustatus.load_byte(state.ppustatus);
        self.oamaddr = state.oamaddr;
        self.write_latch.load_bool(state.write_latch);
        self.v = state.v;
        self.t = state.t;
        self.fine_x = state.fine_x;
        self.tile_register_low = state.tile_register_low;
        self.tile_register_high = state.tile_register_high;
        self.tile_latch_low = state.tile_latch_low;
        self.tile_latch_high = state.tile_latch_high;
        self.attribute_register_1 = state.attribute_register_1;
        self.attribute_register_2 = state.attribute_register_2;
        self.attribute_latch_1 = state.attribute_latch_1;
        self.attribute_latch_2 = state.attribute_latch_2;
        self.oam.copy_from_slice(state.oam.as_slice());
        self.secondary_oam.copy_from_slice(state.secondary_oam.as_slice());
        self.sprites_tile_high.copy_from_slice(state.sprites_tile_high.as_slice());
        self.sprites_tile_low.copy_from_slice(state.sprites_tile_low.as_slice());
        self.sprites_attribute.copy_from_slice(state.sprites_attribute.as_slice());
        self.sprites_x.copy_from_slice(state.sprites_x.as_slice());
        self.scanline = state.scanline;
        self.cycle = state.cycle;
        self.is_odd_frame = state.is_odd_frame;
        self.tmp_pattern_coords = state.tmp_pattern_coords;
        self.tmp_attribute_byte = state.tmp_attribute_byte;
        self.tmp_oam_byte = state.tmp_oam_byte;
        self.sprite_n = state.sprite_n;
        self.sprite_m = state.sprite_m;
        self.sprite_queued_copies = state.sprite_queued_copies;
        self.sprites_copied = state.sprites_copied;
        self.sprite_eval_phase = state.sprite_eval_phase;
        self.num_sprites = state.num_sprites;
        self.sprite_0_next_line = state.sprite_0_next_line;
        self.sprite_0_this_line = state.sprite_0_this_line;
        self.ppudata_read_buffer = state.ppudata_read_buffer;
        self.bus_latch = state.bus_latch;
    }
}
