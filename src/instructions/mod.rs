mod accumulator;

use cpu;
use memory;

type AddressingMode = u8;

#[derive(Clone, Copy)]
struct Operand {
    high_byte: u8,
    low_byte: u8,
}

type Operation = fn(
    &mut cpu::CPU,
    &mut memory::RAM,
    operand_value: u8);

pub struct Instruction {
    operation: Operation,
    addressing_mode: AddressingMode,
    operand: Operand,
}

impl Instruction {
    pub fn execute(&self, cpu: &mut cpu::CPU, memory: &mut memory::RAM) {
        (self.operation)(cpu, memory, self.load_operand());
    }

    fn load_operand(&self) -> u8 {
        0
    }
}

