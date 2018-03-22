#![allow(dead_code)]
pub mod cpu;
pub mod memory;
pub mod util;

pub struct MOS6500 {
    cpu: cpu::CPU,
}

pub fn new() -> MOS6500 {
    let memory = memory::new();
    let cpu = cpu::new(memory);
    MOS6500 {
        cpu,
    }
}

impl MOS6500 {
}
