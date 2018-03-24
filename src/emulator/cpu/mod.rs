mod addressing;
mod instructions;
mod flags;
mod opcodes;

#[cfg(test)]
mod test;

use emulator::memory;

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

            // AND
            opcodes::AND_IMM => (instructions::and, addressing::immediate, 2),
            opcodes::AND_ZPG => (instructions::and, addressing::zero_page, 3),
            opcodes::AND_ZPG_X => (instructions::and, addressing::zero_page_indexed, 4),
            opcodes::AND_ABS => (instructions::and, addressing::absolute, 4),
            opcodes::AND_ABS_X => (instructions::and, addressing::absolute_indexed_x, 4),
            opcodes::AND_ABS_Y => (instructions::and, addressing::absolute_indexed_y, 4),
            opcodes::AND_IX_IND => (instructions::and, addressing::indexed_indirect, 6),
            opcodes::AND_IND_IX => (instructions::and, addressing::indirect_indexed, 5),

            //  BCC, BCS, BEQ, BMI, BNE, BPL, BVC, BVS
            opcodes::BCC => (instructions::bcc, addressing::relative, 2),
            opcodes::BCS => (instructions::bcs, addressing::relative, 2),
            opcodes::BEQ => (instructions::beq, addressing::relative, 2),
            opcodes::BMI => (instructions::bmi, addressing::relative, 2),
            opcodes::BNE => (instructions::bne, addressing::relative, 2),
            opcodes::BPL => (instructions::bpl, addressing::relative, 2),
            opcodes::BVC => (instructions::bvc, addressing::relative, 2),
            opcodes::BVS => (instructions::bvs, addressing::relative, 2),

            // CLC, CLD, CLI
            opcodes::CLC => (instructions::clc, addressing::immediate, 2),
            opcodes::CLD => (instructions::cld, addressing::immediate, 2),
            opcodes::CLI => (instructions::cli, addressing::immediate, 2),
            opcodes::CLV => (instructions::clv, addressing::immediate, 2),

            // EOR
            opcodes::EOR_IMM => (instructions::eor, addressing::immediate, 2),
            opcodes::EOR_ZPG => (instructions::eor, addressing::zero_page, 3),
            opcodes::EOR_ZPG_X => (instructions::eor, addressing::zero_page_indexed, 4),
            opcodes::EOR_ABS => (instructions::eor, addressing::absolute, 4),
            opcodes::EOR_ABS_X => (instructions::eor, addressing::absolute_indexed_x, 4),
            opcodes::EOR_ABS_Y => (instructions::eor, addressing::absolute_indexed_y, 4),
            opcodes::EOR_IX_IND => (instructions::eor, addressing::indexed_indirect, 6),
            opcodes::EOR_IND_IX => (instructions::eor, addressing::indirect_indexed, 5),

            // JMP
            opcodes::JMP_ABS => (instructions::jmp, addressing::absolute, 3),
            opcodes::JMP_IND => (instructions::jmp, addressing::indirect, 5),

            // LDA
            opcodes::LDA_IMM => (instructions::lda, addressing::immediate, 2),
            opcodes::LDA_ZPG => (instructions::lda, addressing::zero_page, 3),
            opcodes::LDA_ZPG_X => (instructions::lda, addressing::zero_page_indexed, 4),
            opcodes::LDA_ABS => (instructions::lda, addressing::absolute, 4),
            opcodes::LDA_ABS_X => (instructions::lda, addressing::absolute_indexed_x, 4),
            opcodes::LDA_ABS_Y => (instructions::lda, addressing::absolute_indexed_y, 4),
            opcodes::LDA_IX_IND => (instructions::lda, addressing::indexed_indirect, 6),
            opcodes::LDA_IND_IX => (instructions::lda, addressing::indirect_indexed, 5),

            // ORA
            opcodes::ORA_IMM => (instructions::ora, addressing::immediate, 2),
            opcodes::ORA_ZPG => (instructions::ora, addressing::zero_page, 3),
            opcodes::ORA_ZPG_X => (instructions::ora, addressing::zero_page_indexed, 4),
            opcodes::ORA_ABS => (instructions::ora, addressing::absolute, 4),
            opcodes::ORA_ABS_X => (instructions::ora, addressing::absolute_indexed_x, 4),
            opcodes::ORA_ABS_Y => (instructions::ora, addressing::absolute_indexed_y, 4),
            opcodes::ORA_IX_IND => (instructions::ora, addressing::indexed_indirect, 6),
            opcodes::ORA_IND_IX => (instructions::ora, addressing::indirect_indexed, 5),

            // SBC
            opcodes::SBC_IMM => (instructions::sbc, addressing::immediate, 2),
            opcodes::SBC_ZPG => (instructions::sbc, addressing::zero_page, 3),
            opcodes::SBC_ZPG_X => (instructions::sbc, addressing::zero_page_indexed, 4),
            opcodes::SBC_ABS => (instructions::sbc, addressing::absolute, 4),
            opcodes::SBC_ABS_X => (instructions::sbc, addressing::absolute_indexed_x, 4),
            opcodes::SBC_ABS_Y => (instructions::sbc, addressing::absolute_indexed_y, 4),
            opcodes::SBC_IX_IND => (instructions::sbc, addressing::indexed_indirect, 6),
            opcodes::SBC_IND_IX => (instructions::sbc, addressing::indirect_indexed, 5),

            // SEC, SED, SEI
            opcodes::SEC => (instructions::sec, addressing::immediate, 2),
            opcodes::SED => (instructions::sed, addressing::immediate, 2),
            opcodes::SEI => (instructions::sei, addressing::immediate, 2),

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
