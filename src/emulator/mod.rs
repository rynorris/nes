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
const NES_MASTER_CLOCK_TIME_NS: u64 = 46;
const NES_CPU_CLOCK_FACTOR: u32 = 12;
const NES_PPU_CLOCK_FACTOR: u32 = 4;

// Pause operation if we drift more than 5ms.
const PAUSE_THRESHOLD_NS: u64 = 5_000_000;

pub struct NES {
    clock: clock::Clock,
    cpu: Rc<RefCell<cpu::CPU>>,
}

impl NES {
    pub fn new(rom: ines::ROM) -> NES {
        // Create master clock.
        let mut clock = clock::Clock::new(NES_MASTER_CLOCK_TIME_NS, PAUSE_THRESHOLD_NS);

        // Load ROM into memory.
        let memory = NES::load(rom);
        let mut manager = memory::new();
        manager.mount(Box::new(memory), 0x8000, 0xFFFF);

        // Create CPU.
        let cpu = Rc::new(RefCell::new(cpu::new(manager)));

        // Create graphics output module and PPU.
        let io = sdl::IO::new();
        let output = sdl::Graphics::new(io);
        let ppu = ppu::PPU::new(Box::new(output));

        // Wire up the clock timings.
        let cpu_ticker = clock::ScaledTicker::new(cpu.clone(), NES_CPU_CLOCK_FACTOR);
        clock.manage(Rc::new(RefCell::new(cpu_ticker)));

        NES {
            clock,
            cpu,
        }
    }

    pub fn tick(&mut self) {
        self.clock.tick();
    }

    pub fn load(rom: ines::ROM) -> impl memory::ReadWriter {
        let mut memory = memory::new();
        rom.prg_rom()
            .iter()
            .enumerate()
            .for_each(|(ix, byte)| memory.write(0x8000 + (ix as u16), *byte));

        memory
    }
}
