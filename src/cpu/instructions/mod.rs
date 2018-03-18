mod accumulator;

use cpu;
use memory;

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
    pub operation: Operation,
    pub addressing_mode: cpu::AddressingMode,
}

pub fn lookup_opcode(opcode: u8) -> Instruction {
    match opcode {
        _ => panic!("Unknown opcode: {:X}", opcode)
    }
}
