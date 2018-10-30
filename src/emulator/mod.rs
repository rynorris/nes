#![allow(dead_code)]
pub mod clock;
pub mod components;
pub mod cpu;
pub mod memory;
pub mod ppu;
pub mod util;

// 21.477272 MHz ~= 46.5ns per clock.
const NES_MASTER_CLOCK_TIME_NS: u64 = 46;

// Pause operation if we drift more than 5ms.
const PAUSE_THRESHOLD_NS: u64 = 5_000_000;

pub struct NES {
    clock: clock::Clock,
    cpu: cpu::CPU,
}

pub fn new() -> NES {
    let clock = clock::Clock::new(NES_MASTER_CLOCK_TIME_NS, PAUSE_THRESHOLD_NS);
    let memory = memory::new();
    let cpu = cpu::new(memory);

    NES {
        clock,
        cpu,
    }
}

impl NES {
}
