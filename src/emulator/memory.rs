use std::cell::RefCell;
use std::rc::Rc;

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

pub struct CPUMemory {
    ram: Box<dyn ReadWriter>,
    ppu_registers: Box<dyn ReadWriter>,
    apu_registers: Box<dyn ReadWriter>,
    sram: Box<dyn ReadWriter>,
    prg_rom: Box<dyn ReadWriter>,
}

impl CPUMemory {
    pub fn new(ram: Box<dyn ReadWriter>, ppu_registers: Box<dyn ReadWriter>, apu_registers: Box<dyn ReadWriter>, sram: Box<dyn ReadWriter>, prg_rom: Box<dyn ReadWriter>) -> CPUMemory {
        CPUMemory { ram, ppu_registers, apu_registers, sram, prg_rom }
    }

    fn map(&mut self, address: u16) -> Option<(&mut Box<dyn ReadWriter>, u16)> {
        match address {
            0x0000 ... 0x1FFF => Some((&mut self.ram, address & 0x7FF)),
            0x2000 ... 0x3FFF => Some((&mut self.ppu_registers, address & 0x7)),
            0x4000 ... 0x401F => Some((&mut self.apu_registers, address)),
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
    vram: Box<dyn ReadWriter>,
}

impl PPUMemory {
    pub fn new(chr_rom: Box<dyn ReadWriter>, vram: Box<dyn ReadWriter>) -> PPUMemory {
        PPUMemory { chr_rom, vram }
    }

    fn map(&mut self, address: u16) -> Option<(&mut Box<dyn ReadWriter>, u16)> {
        match address {
            0x0000 ... 0x1FFF => Some((&mut self.chr_rom, address)),
            0x2000 ... 0x3FFF => Some((&mut self.vram, address & !0x0800)),
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
