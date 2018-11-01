#![allow(dead_code)]
pub mod clock;
pub mod components;
pub mod cpu;
pub mod ines;
pub mod io;
pub mod mappers;
pub mod memory;
pub mod ppu;
pub mod util;

use std::cell::RefCell;
use std::rc::Rc;

use emulator::io::sdl;

// Timings (NTSC).
// Master clock = 21.477272 MHz ~= 46.5ns per clock.
// CPU clock = 12 master clocks.
// PPU clock = 4 master clocks.
const NES_MASTER_CLOCK_TIME_PS: u64 = 1_000_000_000_000 / 21_477_272;
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
    pub fn new(rom: ines::ROM) -> NES {
        // Create master clock.
        let mut clock = clock::Clock::new(0, PAUSE_THRESHOLD_NS);

        // Load ROM into memory.
        let mapper = NES::load(rom);

        // Create graphics output module and PPU.
        let io = sdl::IO::new();
        let output = sdl::Graphics::new(io);

        let ppu_memory = Box::new(memory::PPUMemory::new(
            Box::new(memory::ChrMapper::new(mapper.clone())),
            Box::new(memory::RAM::new()),
        ));

        let ppu = Rc::new(RefCell::new(ppu::PPU::new(
                    ppu_memory,
                    Box::new(output))));

        // Create CPU.
        let cpu_memory = Box::new(memory::CPUMemory::new(
            Box::new(memory::RAM::new()),
            Box::new(ppu.clone()),
            Box::new(memory::RAM::new()),
            Box::new(memory::RAM::new()),
            Box::new(memory::PrgMapper::new(mapper.clone()))
        ));

        let cpu = Rc::new(RefCell::new(cpu::new(cpu_memory)));
        cpu.borrow_mut().disable_bcd();
        cpu.borrow_mut().startup_sequence();

        // Wire up the clock timings.
        let cpu_ticker = clock::ScaledTicker::new(Box::new(cpu.clone()), NES_CPU_CLOCK_FACTOR);
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

    pub fn tick(&mut self) {
        self.clock.tick();
        if self.ppu.borrow().nmi_triggered() {
            if self.nmi_pin == false {
                self.cpu.borrow_mut().trigger_nmi();
                self.nmi_pin = true;
            }
        } else {
            self.nmi_pin = false;
        }

    }

    pub fn elapsed_seconds(&self) -> u64 {
        self.clock.elapsed_seconds()
    }

    pub fn load(rom: ines::ROM) -> memory::MapperRef {
        let prg_rom = rom.prg_rom().to_vec();
        let chr_rom = rom.chr_rom().to_vec();

        match rom.mapper_number() {
            0 => Rc::new(RefCell::new(mappers::NROM::new(prg_rom, chr_rom))),
            1 => Rc::new(RefCell::new(mappers::MMC1::new(prg_rom, chr_rom))),
            _ => panic!("Unknown mapper: {}", rom.mapper_number()),
        }
    }
}
