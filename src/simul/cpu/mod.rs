mod addressing;
mod instructions;
mod flags;

#[cfg(test)]
mod test;

use simul::memory;

// CPU Implemented as a state machine.
pub struct CPU {
    // Connection to main memory.
    pub memory: memory::RAM,

    // Accumulator
    pub a: u8,

    // X Index Register
    pub x: u8,

    // Y Index Register
    pub y: u8,

    // Stack Pointer
    pub sp: u8,

    // Program Counter
    pub pc: u16,

    // Processor Flags NV_BDIZC
    p: flags::ProcessorFlags,
}

pub fn new(memory: memory::RAM) -> CPU {
    CPU {
        memory,
        a: 0,
        x: 0,
        y: 0,
        sp: 0,
        pc: 0,
        p: flags::new(),
    }
}

impl CPU {
    // Returns number of elapsed cycles.
    pub fn execute_next_instruction(&mut self) -> u32 {
        let opcode = self.memory.load(self.pc);
        self.pc += 1;
        let (operation, addressing_mode, cycles) = CPU::decode_instruction(opcode);
        let extra_cycles = operation(self, addressing_mode);

        cycles + extra_cycles
    }

    fn decode_instruction(opcode: u8) -> (instructions::Operation, addressing::AddressingMode, u32) {
        match opcode {
            // LDA
            0xA9 => (instructions::lda, addressing::immediate, 2),
            0xA5 => (instructions::lda, addressing::zero_page, 3),
            0xB5 => (instructions::lda, addressing::zero_page_indexed, 4),
            0xAD => (instructions::lda, addressing::absolute, 4),
            0xBD => (instructions::lda, addressing::absolute_indexed_x, 4),
            0xB9 => (instructions::lda, addressing::absolute_indexed_y, 4),
            0xA1 => (instructions::lda, addressing::indexed_indirect, 6),
            0xB1 => (instructions::lda, addressing::indirect_indexed, 5),

            // STA
            0x85 => (instructions::sta, addressing::zero_page, 3),
            0x95 => (instructions::sta, addressing::zero_page_indexed, 4),
            0x8D => (instructions::sta, addressing::absolute, 4),
            0x9D => (instructions::sta, addressing::absolute_indexed_x, 5),
            0x99 => (instructions::sta, addressing::absolute_indexed_y, 5),

            _ => panic!("Unknown opcode: {:X}", opcode)
        }
    }

    pub fn load_memory(&self, address: u16) -> u8 {
        self.memory.load(address)
    }

    pub fn store_memory(&mut self, address: u16, byte: u8) {
        self.memory.store(address, byte);
    }
}
