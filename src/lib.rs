#![allow(dead_code)]
pub mod cpu;
pub mod memory;
pub mod instructions;

pub struct MOS6500 {
    cpu: cpu::CPU,
    memory: memory::RAM,
}

impl MOS6500 {
    fn execute(&mut self, instruction: instructions::Instruction) {
        instruction.execute(&mut self.cpu, &mut self.memory);
    }
}
