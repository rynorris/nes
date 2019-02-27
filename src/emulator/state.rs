// This file contains the save states API.
// Changes could break old save states.

use std::fs::{create_dir_all, File};
use std::path::PathBuf;

use dirs;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Serialize, Deserialize};
use serde_json::Serializer;

use emulator::ppu::MirrorMode;
use emulator::NES;

pub trait SaveState<'de, T: Serialize + Deserialize<'de>> {
    fn freeze(&mut self) -> T;
    fn hydrate(&mut self, t: T);
}

fn save_state_dir() -> PathBuf {
    let mut path = match dirs::data_dir() {
        Some(path) => path,
        None => panic!("Couldn't get data dir!"),
    };

    path.push("nes");
    path.push("save_states");
    path
}

fn save_state_file_path(name: &str) -> PathBuf {
    let mut state_file_path = save_state_dir();
    state_file_path.push(format!("{}.gz", name));
    state_file_path
}

pub fn save_state(nes: &mut NES, name: &str) -> Result<(), String> {
    create_dir_all(save_state_dir()).map_err(|e| e.to_string())?;

    let state = nes.freeze();
    let state_file = File::create(save_state_file_path(name)).map_err(|e| e.to_string())?;

    let gzip = GzEncoder::new(state_file, Compression::best());
    let mut serializer = Serializer::new(gzip);
    state.serialize(&mut serializer).map_err(|e| e.to_string())?;

    serializer.into_inner().try_finish().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_state(nes: &mut NES, name: &str) -> Result<(), String> {
    let state_file = File::open(save_state_file_path(name)).map_err(|e| e.to_string())?;
    let gzip = GzDecoder::new(state_file);
    let state = serde_json::from_reader(gzip).map_err(|e| e.to_string())?;
    nes.hydrate(state);
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NESState {
    pub cpu: CPUState,
    pub ppu: PPUState,
    pub mapper: MapperState,
    pub ram: MemoryState,
    pub sram: MemoryState,
    pub vram: MemoryState,
    pub screen: ScreenState,
    pub joy1: ControllerState,
    pub joy2: ControllerState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryState {
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScreenState {
    pub scanline: u32,
    pub dot: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ControllerState {
    pub strobe_ix: u8,
    pub register: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MapperState {
    NROM,
    MMC1(MMC1State),
    UXROM(UXROMState),
    CNROM(CNROMState),
    MMC3(MMC3State),
    AXROM(AXROMState),
    ColorDreams(ColorDreamsState),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MMC1State {
    pub load_register: u8,
    pub write_index: u8,
    pub control: u8,
    pub prg_bank: u8,
    pub chr_bank_1: u8,
    pub chr_bank_2: u8,
    pub prg_offsets: Vec<u32>,
    pub chr_offsets: Vec<u32>,
    pub chr_mem: MemoryState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UXROMState {
    pub prg_bank: u8,
    pub chr_mem: MemoryState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CNROMState {
    pub chr_bank: u8,
    pub chr_mem: MemoryState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MMC3State {
    pub bank_registers: Vec<usize>,
    pub bank_select: usize,
    pub prg_inversion: bool,
    pub chr_inversion: bool,
    pub irq_flag: bool,
    pub irq_counter: u8,
    pub irq_reload_flag: bool,
    pub irq_counter_reload: u8,
    pub irq_enabled: bool,
    pub ppu_a12: bool,
    pub ppu_a12_low_counter: u8,
    pub mirror_mode: MirrorMode,
    pub chr_mem: MemoryState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AXROMState {
    pub mirror_mode: MirrorMode,
    pub prg_bank: u8,
    pub chr_mem: MemoryState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorDreamsState {
    pub prg_bank: u8,
    pub chr_bank: u8,
    pub chr_mem: MemoryState,
}
