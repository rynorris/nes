#![allow(dead_code)]
pub mod bus;
pub mod cpu;
pub mod memory;

pub struct MOS6500 {
    cpu: cpu::CPU,
    memory: memory::RAM,
    address_bus: bus::AddressBus,
    data_bus: bus::DataBus,
}

pub fn new() -> MOS6500 {
    let address_bus = bus::new_address_bus();
    let data_bus = bus::new_data_bus();
    let cpu = cpu::new(address_bus.clone(), data_bus.clone());
    let memory = memory::new();
    MOS6500 {
        cpu, memory, address_bus, data_bus,
    }
}

impl MOS6500 {
    fn execute_next_instruction(&mut self) {
        let instruction = self.load_instruction();
        let operand_value = self.load_operand(instruction.addressing_mode);
        (instruction.operation)(&mut self.cpu, &mut self.memory, operand_value);
    }

    fn load_instruction(&mut self) -> instructions::Instruction {
        let opcode = self.memory.get(self.cpu.pc);
        return instructions::lookup_opcode(opcode);
    }

    fn load_operand(&mut self, addressing_mode: instructions::AddressingMode) -> u8 {
        match addressing_mode {
            instructions::AddressingMode::Implied => 0,
            instructions::AddressingMode::Absolute => {
                let high_byte = self.load_next_byte();
                let low_byte = self.load_next_byte();
                self.load_memory_indexed(high_byte, low_byte, 0)
            },
            instructions::AddressingMode::Immediate => self.memory.get(self.cpu.pc),
            instructions::AddressingMode::ZeroPage => {
                let low_byte = self.load_next_byte();
                self.memory.get(low_byte as u16)
            },
            instructions::AddressingMode::Relative => 0,
            instructions::AddressingMode::AbsoluteIndexedX => {
                let high_byte = self.load_next_byte();
                let low_byte = self.load_next_byte();
                self.load_memory_indexed(high_byte, low_byte, self.cpu.x)
            },
            instructions::AddressingMode::AbsoluteIndexedY => {
                let high_byte = self.load_next_byte();
                let low_byte = self.load_next_byte();
                self.load_memory_indexed(high_byte, low_byte, self.cpu.y)
            },
            instructions::AddressingMode::ZeroPageIndexedX => {
                let low_byte = self.load_next_byte();
                self.load_memory_indexed(0, low_byte, self.cpu.x)
            },
            instructions::AddressingMode::IndexedIndirect => {
                let base = self.load_next_byte();
                let sum = (base as u16) + (self.cpu.x as u16);
                let pointer_address = sum | 0xFF;
                let low_byte = self.memory.get(pointer_address);
                let high_byte = self.memory.get(pointer_address + 1);
                self.load_memory_indexed(high_byte, low_byte, 0)
            },
            instructions::AddressingMode::IndirectIndexed => {
                let pointer_address = self.load_next_byte() as u16;
                let low_byte = self.memory.get(pointer_address);
                let high_byte = self.memory.get(pointer_address + 1);
                self.load_memory_indexed(high_byte, low_byte, self.cpu.y)
            },
            instructions::AddressingMode::IndirectAbsolute => {
                let pointer_high = self.load_next_byte();
                let pointer_low = self.load_next_byte();
                let pointer_address = combine_bytes(pointer_high, pointer_low);
                let low_byte = self.memory.get(pointer_address);
                let high_byte = self.memory.get(pointer_address + 1);
                self.load_memory_indexed(high_byte, low_byte, 0)
            },
        }
    }

    fn load_next_byte(&mut self) -> u8 {
        self.cpu.pc += 1;
        self.memory.get(self.cpu.pc)
    }

    fn load_memory_indexed(&self, high_byte: u8, low_byte: u8, index: u8) -> u8 {
        let address = combine_bytes(high_byte, low_byte) + (index as u16);
        self.memory.get(address)
    }
}

fn combine_bytes(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) + (low as u16)
}

#[test]
fn test_combine_bytes() {
    assert_eq!(combine_bytes(0x12, 0xAB), 0x12AB);
}
