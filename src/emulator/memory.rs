use std::cell::RefCell;
use std::rc::Rc;

use emulator::ppu::{Mirrorer, MirrorMode};

const ADDRESS_SPACE: usize = 65536;

pub trait Reader {
    fn read(&mut self, address: u16) -> u8;
}

pub trait Writer {
    fn write(&mut self, address: u16, byte: u8);
}

pub trait ReadWriter : Reader + Writer {}
impl<T: Reader + Writer> ReadWriter for T {}

impl <M : Reader> Reader for Rc<RefCell<M>> {
    fn read(&mut self, address: u16) -> u8 {
        self.borrow_mut().read(address)
    }
}

impl <M : Writer> Writer for Rc<RefCell<M>> {
    fn write(&mut self, address: u16, byte: u8) {
        self.borrow_mut().write(address, byte);
    }
}

pub struct IORegisters {
    apu: Box<dyn ReadWriter>,
    oamdma: u8,
    joy1: Box<dyn ReadWriter>,
    joy2: Box<dyn ReadWriter>,
}

impl IORegisters {
    pub fn new(apu: Box<dyn ReadWriter>, joy1: Box<dyn ReadWriter>, joy2: Box<dyn ReadWriter>) -> IORegisters {
        IORegisters { apu, oamdma: 0, joy1, joy2 }
    }
}

impl Reader for IORegisters {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0x4000 ... 0x4013 | 0x4015 => self.apu.read(address),
            0x4014 => self.oamdma,
            0x4016 => self.joy1.read(address),
            0x4017 => self.joy2.read(address),
            _ => 0,
        }
    }
}

impl Writer for IORegisters {
    fn write(&mut self, address: u16, byte: u8) {
        match address {
            0x4000 ... 0x4013 | 0x4015 => self.apu.write(address, byte),
            0x4014 => self.oamdma = byte,
            0x4016 => self.joy1.write(address, byte),
            0x4017 => {
                // This address half drives the APU and half the joypad.
                self.apu.write(address, byte);
                self.joy2.write(address, byte);
            },
            _ => (),
        }
    }
}

pub struct CPUMemory {
    ram: Box<dyn ReadWriter>,
    ppu_registers: Box<dyn ReadWriter>,
    io_registers: Box<dyn ReadWriter>,
    sram: Box<dyn ReadWriter>,
    prg_rom: Box<dyn ReadWriter>,
}

impl CPUMemory {
    pub fn new(
        ram: Box<dyn ReadWriter>,
        ppu_registers: Box<dyn ReadWriter>,
        io_registers: Box<dyn ReadWriter>,
        sram: Box<dyn ReadWriter>,
        prg_rom: Box<dyn ReadWriter>,
    ) -> CPUMemory {
        CPUMemory { ram, ppu_registers, io_registers, sram, prg_rom }
    }

    fn map(&mut self, address: u16) -> Option<(&mut Box<dyn ReadWriter>, u16)> {
        match address {
            0x0000 ... 0x1FFF => Some((&mut self.ram, address & 0x7FF)),
            0x2000 ... 0x3FFF => Some((&mut self.ppu_registers, address & 0x7)),
            0x4000 ... 0x401F => Some((&mut self.io_registers, address)),
            0x6000 ... 0x7FFF => Some((&mut self.sram, address - 0x6000)),
            0x8000 ... 0xFFFF => Some((&mut self.prg_rom, address)),
            _ => None
        }
    }
}

impl Reader for CPUMemory {
    fn read(&mut self, address: u16) -> u8 {
        self.map(address).map(|(mem, addr)| mem.read(addr)).unwrap_or(0)
    }
}

impl Writer for CPUMemory {
    fn write(&mut self, address: u16, byte: u8) {
        self.map(address).map(|(mem, addr)| mem.write(addr, byte));
    }
}

pub struct PPUMemory {
    chr_rom: Box<dyn ReadWriter>,
    mirrorer: Box<dyn Mirrorer>,
    vram: Box<dyn ReadWriter>,
}

impl PPUMemory {
    pub fn new(chr_rom: Box<dyn ReadWriter>, mirrorer: Box<dyn Mirrorer>, vram: Box<dyn ReadWriter>) -> PPUMemory {
        PPUMemory { chr_rom, mirrorer, vram }
    }

    fn map(&mut self, address: u16) -> Option<(&mut Box<dyn ReadWriter>, u16)> {
        // Whole thing is mirrored above $4000.
        match address & 0x3FFF {
            0x0000 ... 0x1FFF => Some((&mut self.chr_rom, address & 0x3FFF)),
            0x2000 ... 0x3EFF => {
                // Nametable and nametable mirrors.
                // Note that we don't just literally mirror the address horizontally/vertically.
                // We need to make sure we always read from one of just 2 banks of memory.
                let nt_bank = match self.mirrorer.mirror_mode() {
                    MirrorMode::SingleLower => 0,
                    MirrorMode::SingleUpper => 1,
                    MirrorMode::Vertical => (address & 0x0400) >> 10,
                    MirrorMode::Horizontal => (address & 0x0800) >> 11,
                };
                let mirrored_addr = 0x2000 | (nt_bank << 10) | (address & 0x03FF);
                Some((&mut self.vram, mirrored_addr & 0x2FFF))
            },
            0x3F00 ... 0x3FFF => {
                // Palettes and palette mirrors.
                let mirrored_addr = if address % 4 == 0 {
                    // Colour 0 in sprite palettes is mirrored to the BG palettes.
                    address & 0x3F0F
                } else {
                    address & 0x3F1F
                };
                Some((&mut self.vram, mirrored_addr))
            },
            _ => None
        }
    }
}

impl Reader for PPUMemory {
    fn read(&mut self, address: u16) -> u8 {
        self.map(address).map(|(mem, addr)| mem.read(addr)).unwrap_or(0)
    }
}

impl Writer for PPUMemory {
    fn write(&mut self, address: u16, byte: u8) {
        self.map(address).map(|(mem, addr)| mem.write(addr, byte));
    }
}

pub trait Mapper {
    fn read_chr(&mut self, address: u16) -> u8;
    fn write_chr(&mut self, address: u16, byte: u8);
    fn read_prg(&mut self, address: u16) -> u8;
    fn write_prg(&mut self, address: u16, byte: u8);
    fn mirror_mode(&self) -> MirrorMode;
}

pub type MapperRef = Rc<RefCell<dyn Mapper>>;

impl Mapper for MapperRef {
    fn read_chr(&mut self, address: u16) -> u8 {
        self.borrow_mut().read_chr(address)
    }

    fn write_chr(&mut self, address: u16, byte: u8) {
        self.borrow_mut().write_chr(address, byte)
    }

    fn read_prg(&mut self, address: u16) -> u8 {
        self.borrow_mut().read_prg(address)
    }

    fn write_prg(&mut self, address: u16, byte: u8) {
        self.borrow_mut().write_prg(address, byte)
    }

    fn mirror_mode(&self) -> MirrorMode {
        self.borrow().mirror_mode()
    }
}

impl Mirrorer for MapperRef {
    fn mirror_mode(&self) -> MirrorMode {
        self.borrow().mirror_mode()
    }
}

pub struct PrgMapper<M : Mapper> {
    mapper: M,
}

impl <M : Mapper> PrgMapper<M> {
    pub fn new(mapper: M) -> PrgMapper<M> {
        PrgMapper { mapper }
    }
}

impl <M : Mapper> Reader for PrgMapper<M> {
    fn read(&mut self, address: u16) -> u8 {
        self.mapper.read_prg(address)
    }
}

impl <M : Mapper> Writer for PrgMapper<M> {
    fn write(&mut self, address: u16, byte: u8) {
        self.mapper.write_prg(address, byte)
    }
}

pub struct ChrMapper<M : Mapper> {
    mapper: M,
}

impl <M : Mapper> ChrMapper<M> {
    pub fn new(mapper: M) -> ChrMapper<M> {
        ChrMapper { mapper }
    }
}

impl <M : Mapper> Reader for ChrMapper<M> {
    fn read(&mut self, address: u16) -> u8 {
        self.mapper.read_chr(address)
    }
}

impl <M : Mapper> Writer for ChrMapper<M> {
    fn write(&mut self, address: u16, byte: u8) {
        self.mapper.write_chr(address, byte)
    }
}

pub struct RAM {
    memory: [u8; ADDRESS_SPACE],
}

impl Reader for RAM {
    fn read(&mut self, address: u16) -> u8 {
        self.memory[address as usize]
    }
}

impl Writer for RAM {
    fn write(&mut self, address: u16, byte: u8) {
        self.memory[address as usize] = byte
    }
}

impl RAM {
    pub fn new() -> RAM {
        RAM{
            memory: [0; ADDRESS_SPACE],
        }
    }

    pub fn debug_print(&self, start_addr: u16, num_bytes: u16) {
        let end_addr = start_addr - 1 + num_bytes;
        println!("RAM [{:X}..{:X}]: {:?}", start_addr, end_addr, &self.memory[(start_addr as usize) .. (end_addr as usize)]);
    }
}

#[test]
fn test_get_and_set() {
    let mut ram = RAM::new();
    ram.write(1234, 23);
    assert_eq!(ram.read(1234), 23);
}
