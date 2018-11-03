#![allow(dead_code)]
pub mod clock;
pub mod components;
pub mod controller;
pub mod cpu;
pub mod ines;
pub mod io;
pub mod mappers;
pub mod memory;
pub mod ppu;
pub mod util;

#[cfg(test)]
mod test;

use std::cell::RefCell;
use std::rc::Rc;

use emulator::controller::Button;
use emulator::io::event::Key;
use emulator::io::Input;
use emulator::memory::ReadWriter;
use emulator::ppu::VideoOut;

// Timings (NTSC).
// Master clock = 21.477272 MHz ~= 46.5ns per clock.
// CPU clock = 12 master clocks.
// PPU clock = 4 master clocks.
pub const NES_MASTER_CLOCK_HZ: u64 = 21_477_272;
pub const NES_MASTER_CLOCK_TIME_PS: u64 = 1_000_000_000_000 / NES_MASTER_CLOCK_HZ;
const NES_CPU_CLOCK_FACTOR: u32 = 12;
const NES_PPU_CLOCK_FACTOR: u32 = 4;

// Pause operation if we drift more than 20ms.
const PAUSE_THRESHOLD_NS: u64 = 20_000_000;

pub struct NES {
    clock: clock::Clock,
    pub cpu: Rc<RefCell<cpu::CPU>>,
    ppu: Rc<RefCell<ppu::PPU>>,
    nmi_pin: bool,
}

impl NES {
    pub fn new<I : Input, V: VideoOut + 'static>(mut input: I, video: V, rom: ines::ROM) -> NES {
        // Create master clock.
        let mut clock = clock::Clock::new();

        // Load ROM into memory.
        let mapper = NES::load(rom);

        // Create graphics output module and PPU.
        let ppu_memory = Box::new(memory::PPUMemory::new(
            Box::new(memory::ChrMapper::new(mapper.clone())),
            Box::new(mapper.clone()),
            Box::new(memory::RAM::new()),
        ));

        let ppu = Rc::new(RefCell::new(ppu::PPU::new(
                    ppu_memory,
                    Box::new(video))));

        // Create controllers.
        let joy1 = Rc::new(RefCell::new(controller::Controller::new([
           (Key::Z, Button::A),
           (Key::X, Button::B),
           (Key::A, Button::Start),
           (Key::S, Button::Select),
           (Key::Up, Button::Up),
           (Key::Down, Button::Down),
           (Key::Left, Button::Left),
           (Key::Right, Button::Right),
        ].iter().cloned().collect())));

        let joy2 = Rc::new(RefCell::new(controller::Controller::new([
        ].iter().cloned().collect())));

        input.register_event_handler(Box::new(joy1.clone()));
        input.register_event_handler(Box::new(joy2.clone()));

        // Create CPU.
        let io_registers = Rc::new(RefCell::new(memory::IORegisters::new(
            Box::new(joy1.clone()),
            Box::new(joy2.clone()),
        )));

        let cpu_memory = Box::new(memory::CPUMemory::new(
            Box::new(memory::RAM::new()),
            Box::new(ppu.clone()),
            Box::new(io_registers.clone()),
            Box::new(memory::RAM::new()),
            Box::new(memory::PrgMapper::new(mapper.clone()))
        ));

        let cpu = Rc::new(RefCell::new(cpu::new(cpu_memory)));
        cpu.borrow_mut().disable_bcd();
        cpu.borrow_mut().startup_sequence();

        let dma_controller = DMAController::new(
            Box::new(io_registers.clone()),
            cpu.clone()
        );

        // Wire up the clock timings.
        let cpu_ticker = clock::ScaledTicker::new(Box::new(dma_controller), NES_CPU_CLOCK_FACTOR);
        let ppu_ticker = clock::ScaledTicker::new(Box::new(ppu.clone()), NES_PPU_CLOCK_FACTOR);
        clock.manage(Box::new(cpu_ticker));
        clock.manage(Box::new(ppu_ticker));

        NES {
            clock,
            cpu,
            ppu,
            nmi_pin: false,
        }
    }

    pub fn tick(&mut self) -> u64 {
        let cycles = self.clock.tick();
        if self.ppu.borrow().nmi_triggered() {
            if self.nmi_pin == false {
                self.cpu.borrow_mut().trigger_nmi();
                self.nmi_pin = true;
            }
        } else {
            self.nmi_pin = false;
        }

        cycles
    }

    pub fn load(rom: ines::ROM) -> memory::MapperRef {
        let prg_rom = rom.prg_rom().to_vec();
        let chr_rom = rom.chr_rom().to_vec();
        let mirror_mode = rom.mirror_mode();

        match rom.mapper_number() {
            0 => Rc::new(RefCell::new(mappers::NROM::new(prg_rom, chr_rom, mirror_mode))),
            1 => Rc::new(RefCell::new(mappers::MMC1::new(prg_rom, chr_rom))),
            _ => panic!("Unknown mapper: {}", rom.mapper_number()),
        }
    }
}

pub struct DMAController {
    copies_remaining: u16,
    base_address: u16,
    io_registers: Box<dyn ReadWriter>,
    cpu: Rc<RefCell<cpu::CPU>>,
}

impl DMAController {
    pub fn new(io_registers: Box<dyn ReadWriter>, cpu: Rc<RefCell<cpu::CPU>>) -> DMAController {
        DMAController {
            copies_remaining: 0,
            base_address: 0,
            io_registers,
            cpu, 
        }
    }

    pub fn trigger_dma(&mut self, base_address: u16) {
        self.copies_remaining = 256;
        self.base_address = base_address;
    }
}

impl clock::Ticker for DMAController {
    fn tick(&mut self) -> u32 {
        let byte = self.io_registers.read(0x4014);
        if byte != 0 {
            // DMA triggered.
            self.base_address = (byte as u16) << 8;
            self.copies_remaining = 256;
            self.io_registers.write(0x4014, 0);
        }

        if self.copies_remaining > 0 {
            // CPU is suspended during copy.
            let byte = self.cpu.borrow_mut().load_memory(self.base_address + 256 - self.copies_remaining);
            self.cpu.borrow_mut().store_memory(0x2004, byte);
            self.copies_remaining -= 1;
            2
        } else {
            self.cpu.borrow_mut().tick()
        }
    }
}
