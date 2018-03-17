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

pub enum AddressingMode {
    // Implied: no operand.
    Implied,

    // Absolute: two byte operand indicates memory address.
    Absolute,

    // Immediate: one byte literal operand.
    Immediate,

    // Zero page: one byte operand indicates address in page 0 of memory.
    ZeroPage,

    // Relative: one byte operand indicates address relative to PC.
    Relative,

    // Absolute indexed: same as absolute addressing, but adds an index register to the
    // address.
    AbsoluteIndexedX,
    AbsoluteIndexedY,

    // Zero page indexed: same as zero page, but adds an index register to the address.
    // Only supported for index X.
    // If the resulting value is greated than 255, the address wraps within page 0.
    ZeroPageIndexedX,

    // Indirect addressing is where we look up the two byte address to read from a location in page-zero.
    // i.e. pointers.
    //
    // Indexed Indirect is where we add index X to the one byte zero page operand to find the
    // lookup address. As with Zero page indexed, the resulting zero page address wraps.
    //
    // Indirect Indexed is where we look up the address first from the specified location in page
    // zero, and _then_ add index Y to the absolute address.
    //
    // Indirect absolute is where we look up the address to read from another absolute address.
    // This is only used by the jump instruction.
    IndexedIndirect,
    IndirectIndexed,
    IndirectAbsolute,
}

pub fn lookup_opcode(opcode: u8) -> (Operation, AddressingMode) {
    match opcode {
        _ => panic!("Unknown opcode: {:X}", opcode)
    }
}
