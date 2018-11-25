// This file contains the save states API.
// Changes could break old save states.

extern crate serde;

use self::serde::{Serialize, Deserialize};

use emulator::ppu::MirrorMode;

pub trait SaveState<'de, T: Serialize + Deserialize<'de>> {
    fn freeze(&mut self) -> T;
    fn hydrate(&mut self, t: T);
}

#[derive(Serialize, Deserialize)]
pub struct CPUState {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub p: u8,
    pub dec_arith_on: bool,
    pub irq_flip_flop: bool,
    pub nmi_flip_flop: bool,
}

#[derive(Serialize, Deserialize)]
pub struct PPUState {
    pub ppuctrl: u8,
    pub ppumask: u8,
    pub ppustatus: u8,
    pub oamaddr: u8,
    pub write_latch: bool,
    pub v: u16,
    pub t: u16,
    pub fine_x: u8,
    pub tile_register_low: u16,
    pub tile_register_high: u16,
    pub tile_latch_low: u8,
    pub tile_latch_high: u8,
    pub attribute_register_1: u8,
    pub attribute_register_2: u8,
    pub attribute_latch_1: u8,
    pub attribute_latch_2: u8,

    #[serde(with = "serde_bytes")]
    pub oam: Vec<u8>,

    #[serde(with = "serde_bytes")]
    pub secondary_oam: Vec<u8>,

    #[serde(with = "serde_bytes")]
    pub sprites_tile_high: Vec<u8>,

    #[serde(with = "serde_bytes")]
    pub sprites_tile_low: Vec<u8>,

    #[serde(with = "serde_bytes")]
    pub sprites_attribute: Vec<u8>,

    #[serde(with = "serde_bytes")]
    pub sprites_x: Vec<u8>,

    pub scanline: u16,
    pub cycle: u16,
    pub is_odd_frame: bool,
    pub tmp_pattern_coords: u8,
    pub tmp_attribute_byte: u8,
    pub tmp_oam_byte: u8,
    pub sprite_n: u8,
    pub sprite_m: u8,
    pub sprite_queued_copies: u8,
    pub sprites_copied: u8,
    pub sprite_eval_phase: u8,
    pub num_sprites: u8,
    pub sprite_0_next_line: bool,
    pub sprite_0_this_line: bool,
    pub ppudata_read_buffer: u8,
    pub bus_latch: u8,
}

#[derive(Serialize, Deserialize)]
pub enum MapperState {
    NROM,
    MMC1 {
        load_register: u8,
        write_index: u8,
        control: u8,
        prg_bank: u8,
        chr_bank_1: u8,
        chr_bank_2: u8,
        prg_offsets: Vec<u32>,
        chr_offsets: Vec<u32>,
        chr_ram: Vec<u8>,
    },
    UXROM {
        prg_bank: u8,
    },
    CNROM {
        chr_bank: u8,
    },
    MMC3 {
        bank_registers: Vec<usize>,
        bank_select: usize,
        prg_inversion: bool,
        chr_inversion: bool,
        irq_flag: bool,
        irq_counter: u8,
        irq_reload_flag: bool,
        irq_counter_reload: u8,
        irq_enabled: bool,
        ppu_a12: bool,
        ppu_a12_low_counter: u8,
        mirror_mode: MirrorMode,
        chr_ram: Vec<u8>,
    },
    AXROM {
        mirror_mode: MirrorMode,
        prg_bank: u8,
        chr_ram: Vec<u8>,
    },
}
