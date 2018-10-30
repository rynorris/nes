#![allow(dead_code)]
pub mod components;
pub mod cpu;
pub mod memory;
pub mod ppu;
pub mod util;

pub struct NES {
    cpu: cpu::CPU,
}

pub fn new() -> NES {
    let memory = memory::new();
    let cpu = cpu::new(memory);
    NES {
        cpu,
    }
}

impl NES {
}
