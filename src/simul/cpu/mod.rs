mod addressing;
mod instructions;
mod flags;
mod opcodes;

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
        // Note: Maintain list in alphabetical order.
        match opcode {
            // ADC
            opcodes::ADC_IMM => (instructions::adc, addressing::immediate, 2),
            opcodes::ADC_ZPG => (instructions::adc, addressing::zero_page, 3),
            opcodes::ADC_ZPG_X => (instructions::adc, addressing::zero_page_indexed, 4),
            opcodes::ADC_ABS => (instructions::adc, addressing::absolute, 4),
            opcodes::ADC_ABS_X => (instructions::adc, addressing::absolute_indexed_x, 4),
            opcodes::ADC_ABS_Y => (instructions::adc, addressing::absolute_indexed_y, 4),
            opcodes::ADC_IX_IND => (instructions::adc, addressing::indexed_indirect, 6),
            opcodes::ADC_IND_IX => (instructions::adc, addressing::indirect_indexed, 5),

            // LDA
            opcodes::LDA_IMM => (instructions::lda, addressing::immediate, 2),
            opcodes::LDA_ZPG => (instructions::lda, addressing::zero_page, 3),
            opcodes::LDA_ZPG_X => (instructions::lda, addressing::zero_page_indexed, 4),
            opcodes::LDA_ABS => (instructions::lda, addressing::absolute, 4),
            opcodes::LDA_ABS_X => (instructions::lda, addressing::absolute_indexed_x, 4),
            opcodes::LDA_ABS_Y => (instructions::lda, addressing::absolute_indexed_y, 4),
            opcodes::LDA_IX_IND => (instructions::lda, addressing::indexed_indirect, 6),
            opcodes::LDA_IND_IX => (instructions::lda, addressing::indirect_indexed, 5),

            // SBC
            opcodes::SBC_IMM => (instructions::sbc, addressing::immediate, 2),
            opcodes::SBC_ZPG => (instructions::sbc, addressing::zero_page, 3),
            opcodes::SBC_ZPG_X => (instructions::sbc, addressing::zero_page_indexed, 4),
            opcodes::SBC_ABS => (instructions::sbc, addressing::absolute, 4),
            opcodes::SBC_ABS_X => (instructions::sbc, addressing::absolute_indexed_x, 4),
            opcodes::SBC_ABS_Y => (instructions::sbc, addressing::absolute_indexed_y, 4),
            opcodes::SBC_IX_IND => (instructions::sbc, addressing::indexed_indirect, 6),
            opcodes::SBC_IND_IX => (instructions::sbc, addressing::indirect_indexed, 5),

            // STA
            opcodes::STA_ZPG => (instructions::sta, addressing::zero_page, 3),
            opcodes::STA_ZPG_X => (instructions::sta, addressing::zero_page_indexed, 4),
            opcodes::STA_ABS => (instructions::sta, addressing::absolute, 4),
            opcodes::STA_ABS_X => (instructions::sta, addressing::absolute_indexed_x, 5),
            opcodes::STA_ABS_Y => (instructions::sta, addressing::absolute_indexed_y, 5),
            opcodes::STA_IX_IND => (instructions::sta, addressing::indexed_indirect, 6),
            opcodes::STA_IND_IX => (instructions::sta, addressing::indirect_indexed, 6),

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
