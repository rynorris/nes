#![allow(dead_code)]
pub mod clock;
pub mod components;
pub mod cpu;
pub mod ines;
pub mod io;
pub mod memory;
pub mod ppu;
pub mod util;

use std::cell::RefCell;
use std::rc::Rc;

use self::io::sdl;
use self::memory::Writer;

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
        let (cpu_memory, ppu_memory) = NES::load(rom);
        cpu_memory.debug_print(0xFFF0, 16);

        let mut manager = memory::new();
        manager.mount(Rc::new(RefCell::new(cpu_memory)), 0x8000, 0xFFFF);
        manager.debug_print(0xFFF0, 16);

        // Create graphics output module and PPU.
        let io = sdl::IO::new();
        let output = sdl::Graphics::new(io);
        let mut ppu_manager = memory::new();
        ppu_manager.mount(Rc::new(RefCell::new(ppu_memory)), 0x0000, 0x1FFF);
        let ppu = Rc::new(RefCell::new(ppu::PPU::new(ppu_manager, Box::new(output))));

        // Mount PPU registers on main memory.
        manager.mount(ppu.clone(), 0x2000, 0x3FFF);
        
        // Create CPU.
        let cpu = Rc::new(RefCell::new(cpu::new(manager)));
        cpu.borrow_mut().disable_bcd();
        cpu.borrow_mut().startup_sequence();

        // Wire up the clock timings.
        let cpu_ticker = clock::ScaledTicker::new(cpu.clone(), NES_CPU_CLOCK_FACTOR);
        let ppu_ticker = clock::ScaledTicker::new(ppu.clone(), NES_PPU_CLOCK_FACTOR);
        clock.manage(Rc::new(RefCell::new(cpu_ticker)));
        clock.manage(Rc::new(RefCell::new(ppu_ticker)));

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

    pub fn load(rom: ines::ROM) -> (memory::RAM, memory::RAM) {
        let mut cpu_memory = memory::RAM::new();
        rom.prg_rom()
            .iter()
            .enumerate()
            .for_each(|(ix, byte)| {
                cpu_memory.write(0x8000 + (ix as u16), *byte);
                cpu_memory.write(0xC000 + (ix as u16), *byte);
            });

        let mut ppu_memory = memory::RAM::new();
        rom.chr_rom()
            .iter()
            .enumerate()
            .for_each(|(ix, byte)| {
                ppu_memory.write(0x0000 + (ix as u16), *byte);
            });

        (cpu_memory, ppu_memory)
    }
}
