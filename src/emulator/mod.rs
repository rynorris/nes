#![allow(dead_code)]
pub mod clock;
pub mod components;
pub mod cpu;
pub mod io;
pub mod memory;
pub mod ppu;
pub mod util;

use std::cell::RefCell;
use std::rc::Rc;

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

pub fn new() -> NES {
    // Create components.
    let mut clock = clock::Clock::new(NES_MASTER_CLOCK_TIME_NS, PAUSE_THRESHOLD_NS);
    let memory = memory::new();
    let cpu = Rc::new(RefCell::new(cpu::new(memory)));

    // Wire up the clock timings.
    let cpu_ticker = clock::ScaledTicker::new(cpu.clone(), NES_CPU_CLOCK_FACTOR);
    clock.manage(Rc::new(RefCell::new(cpu_ticker)));

    NES {
        clock,
        cpu,
    }
}

impl NES {
}
