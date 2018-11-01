mod addressing;
mod instructions;
mod flags;
mod opcodes;
mod trace;

#[cfg(test)]
mod test;

use std::io::Write;

use emulator::clock;
use emulator::components::bitfield::BitField;
use emulator::memory::ReadWriter;
use emulator::util;

// Program vector locations.
pub const START_VECTOR: u16 = 0xFFFC;
pub const IRQ_VECTOR: u16= 0xFFFE;
pub const NMI_VECTOR: u16= 0xFFFA;

pub enum Flag {
    N = 1 << 7, // Negative
    V = 1 << 6, // Overflow
    B = 1 << 4, // Break Flag
    D = 1 << 3, // BCD Mode
    I = 1 << 2, // Interrupt Disable
    Z = 1 << 1, // Zero
    C = 1, // Carry
}

impl Into<u8> for Flag {
    fn into(self) -> u8 {
        self as u8
    }
}

pub struct CPU {
    // Connection to main memory.
    memory: Box<ReadWriter>,

    // Accumulator
    a: u8,

    // X Index Register
    x: u8,

    
    y: u8,

    // Stack Pointer
    sp: u8,

    // Program Counter
    pc: u16,

    // Processor Flags NV_BDIZC
    p: BitField,

    // Decimal arithmetic enabled?
    dec_arith_on: bool,

    // NMI triggered?
    nmi_flip_flop: bool,
}

pub fn new(memory: Box<ReadWriter>) -> CPU {
    let mut p = BitField::new();
    p.load_byte(0x00);
    CPU {
        memory,
        a: 0,
        x: 0,
        y: 0,
        sp: 0xFD,
        pc: 0,
        p,
        dec_arith_on: true,
        nmi_flip_flop: false,
    }
}

impl clock::Ticker for CPU {
    fn tick(&mut self) -> u32 {
        return if self.should_non_maskable_interrupt() {
            self.nmi_flip_flop = false;
            self.non_maskable_interrupt()
        } else if self.should_interrupt() {
            self.interrupt()
        } else {
            self.execute_next_instruction()
        }
    }
}

impl CPU {
    pub fn startup_sequence(&mut self) -> u32 {
        self.load_vector_to_pc(START_VECTOR);

        // Disable interrupts at startup.  The programmer should re-enable once they have completed
        // initializing the system.
        self.p.set(flags::Flag::I);
        0
    }

    pub fn load_program(&mut self, program: &[u8]) {
        for (ix, byte) in program.iter().enumerate() {
            self.memory.write(ix as u16, *byte);
        }
    }

    pub fn disable_bcd(&mut self) {
        self.dec_arith_on = false;
    }

    pub fn enable_bcd(&mut self) {
        self.dec_arith_on = true;
    }

    pub fn trigger_nmi(&mut self) {
        self.nmi_flip_flop = true;
    }

    fn peek_next_instruction(&mut self) -> (u8, Option<u8>, Option<u8>) {
        // Note since addressing modes modify the PC themselves we have to hack a bit here
        // to figure out which bytes form the next instruction.
        // Should probably refactor addressing modes so we can just query how many bytes it is.
        let saved_pc = self.pc;
        let opcode = self.memory.read(self.pc);
        let (_, addressing_mode, _) = CPU::decode_instruction(opcode);
        let (_, _) = addressing_mode(self);
        let num_bytes = self.pc - saved_pc;
        self.pc = saved_pc;

        // Now we have the number of bytes, lets trace out the instruction.
        let b1 = if num_bytes > 0 { Some(self.memory.read(self.pc + 1)) } else { None };
        let b2 = if num_bytes > 1 { Some(self.memory.read(self.pc + 2)) } else { None };

        (opcode, b1, b2)
    }

    pub fn trace_next_instruction<W : Write>(&mut self, mut w: W) {
        let (opcode, b1, b2) = self.peek_next_instruction();

        write!(w, "{:04X}  {:02X} ", self.pc, opcode);

        let _ = match b1 {
            Some(b) => write!(w, "{:02X} ", b),
            None => write!(w, "   "),
        };

        let _ = match b2 {
            Some(b) => write!(w, "{:02X}  ", b),
            None => write!(w, "    "),
        };

        write!(w, "{:<32}", trace::format_instruction(opcode, b1.unwrap_or(0), b2.unwrap_or(0)));

        // Dump registers.
        write!(w, "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}", self.a, self.x, self.y, self.p.as_byte(), self.sp);

    }

    // Returns number of elapsed cycles.
    fn execute_next_instruction(&mut self) -> u32 {
        let opcode = self.memory.read(self.pc);
        self.pc += 1;
        let (operation, addressing_mode, cycles) = CPU::decode_instruction(opcode);
        let extra_cycles = operation(self, addressing_mode);

        cycles + extra_cycles
    }

    fn interrupt(&mut self) -> u32 {
        self.interrupt_to_vector(IRQ_VECTOR)
    }

    fn non_maskable_interrupt(&mut self) -> u32 {
        self.interrupt_to_vector(NMI_VECTOR)
    }

    fn interrupt_to_vector(&mut self, vector: u16) -> u32 {
        // Store processor state.
        let pch = (self.pc >> 8) as u8;
        let pcl = self.pc as u8;
        self.stack_push(pch);
        self.stack_push(pcl);
        let p = self.p.as_byte();
        self.stack_push(p);

        self.load_vector_to_pc(vector);

        // Disable interrupts.  The programmer should re-enable once they have completed
        // initial interrupt handling.
        self.p.set(flags::Flag::I);
        8
    }

    fn should_interrupt(&self) -> bool {
        false
    }

    fn should_non_maskable_interrupt(&self) -> bool {
        self.nmi_flip_flop
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

            // ASL
            opcodes::ASL_A => (instructions::asla, addressing::implied, 2),
            opcodes::ASL_ZPG => (instructions::asl, addressing::zero_page, 5),
            opcodes::ASL_ZPG_X => (instructions::asl, addressing::zero_page_indexed, 6),
            opcodes::ASL_ABS => (instructions::asl, addressing::absolute, 6),
            opcodes::ASL_ABS_X => (instructions::asl, addressing::absolute_indexed_x, 7),

            // BCC, BCS, BEQ
            opcodes::BCC => (instructions::bcc, addressing::relative, 2),
            opcodes::BCS => (instructions::bcs, addressing::relative, 2),
            opcodes::BEQ => (instructions::beq, addressing::relative, 2),

            // BIT
            opcodes::BIT_ZPG => (instructions::bit, addressing::zero_page, 3),
            opcodes::BIT_ABS => (instructions::bit, addressing::absolute, 4),

            // BMI, BNE, BPL, BVC, BVS
            opcodes::BMI => (instructions::bmi, addressing::relative, 2),
            opcodes::BNE => (instructions::bne, addressing::relative, 2),
            opcodes::BPL => (instructions::bpl, addressing::relative, 2),
            opcodes::BVC => (instructions::bvc, addressing::relative, 2),
            opcodes::BVS => (instructions::bvs, addressing::relative, 2),

            // BRK
            opcodes::BRK => (instructions::brk, addressing::implied, 7),

            // CLC, CLD, CLI
            opcodes::CLC => (instructions::clc, addressing::implied, 2),
            opcodes::CLD => (instructions::cld, addressing::implied, 2),
            opcodes::CLI => (instructions::cli, addressing::implied, 2),
            opcodes::CLV => (instructions::clv, addressing::implied, 2),

            // CMP
            opcodes::CMP_IMM => (instructions::cmp, addressing::immediate, 2),
            opcodes::CMP_ZPG => (instructions::cmp, addressing::zero_page, 3),
            opcodes::CMP_ZPG_X => (instructions::cmp, addressing::zero_page_indexed, 4),
            opcodes::CMP_ABS => (instructions::cmp, addressing::absolute, 4),
            opcodes::CMP_ABS_X => (instructions::cmp, addressing::absolute_indexed_x, 4),
            opcodes::CMP_ABS_Y => (instructions::cmp, addressing::absolute_indexed_y, 4),
            opcodes::CMP_IX_IND => (instructions::cmp, addressing::indexed_indirect, 6),
            opcodes::CMP_IND_IX => (instructions::cmp, addressing::indirect_indexed, 5),

            // CPX
            opcodes::CPX_IMM => (instructions::cpx, addressing::immediate, 2),
            opcodes::CPX_ZPG => (instructions::cpx, addressing::zero_page, 3),
            opcodes::CPX_ABS => (instructions::cpx, addressing::absolute, 4),

            // CPY
            opcodes::CPY_IMM => (instructions::cpy, addressing::immediate, 2),
            opcodes::CPY_ZPG => (instructions::cpy, addressing::zero_page, 3),
            opcodes::CPY_ABS => (instructions::cpy, addressing::absolute, 4),

            // DEC
            opcodes::DEC_ZPG => (instructions::dec, addressing::zero_page, 5),
            opcodes::DEC_ZPG_X => (instructions::dec, addressing::zero_page_indexed, 6),
            opcodes::DEC_ABS => (instructions::dec, addressing::absolute, 6),
            opcodes::DEC_ABS_X => (instructions::dec, addressing::absolute_indexed_x, 7),

            // DEX, INY
            opcodes::DEX => (instructions::dex, addressing::implied, 2),
            opcodes::DEY => (instructions::dey, addressing::implied, 2),

            // EOR
            opcodes::EOR_IMM => (instructions::eor, addressing::immediate, 2),
            opcodes::EOR_ZPG => (instructions::eor, addressing::zero_page, 3),
            opcodes::EOR_ZPG_X => (instructions::eor, addressing::zero_page_indexed, 4),
            opcodes::EOR_ABS => (instructions::eor, addressing::absolute, 4),
            opcodes::EOR_ABS_X => (instructions::eor, addressing::absolute_indexed_x, 4),
            opcodes::EOR_ABS_Y => (instructions::eor, addressing::absolute_indexed_y, 4),
            opcodes::EOR_IX_IND => (instructions::eor, addressing::indexed_indirect, 6),
            opcodes::EOR_IND_IX => (instructions::eor, addressing::indirect_indexed, 5),

            // INC
            opcodes::INC_ZPG => (instructions::inc, addressing::zero_page, 5),
            opcodes::INC_ZPG_X => (instructions::inc, addressing::zero_page_indexed, 6),
            opcodes::INC_ABS => (instructions::inc, addressing::absolute, 6),
            opcodes::INC_ABS_X => (instructions::inc, addressing::absolute_indexed_x, 7),

            // INX, INY
            opcodes::INX => (instructions::inx, addressing::implied, 2),
            opcodes::INY => (instructions::iny, addressing::implied, 2),

            // JMP
            opcodes::JMP_ABS => (instructions::jmp, addressing::absolute, 3),
            opcodes::JMP_IND => (instructions::jmp, addressing::indirect, 5),

            // JSR
            opcodes::JSR => (instructions::jsr, addressing::absolute, 6),

            // LDA
            opcodes::LDA_IMM => (instructions::lda, addressing::immediate, 2),
            opcodes::LDA_ZPG => (instructions::lda, addressing::zero_page, 3),
            opcodes::LDA_ZPG_X => (instructions::lda, addressing::zero_page_indexed, 4),
            opcodes::LDA_ABS => (instructions::lda, addressing::absolute, 4),
            opcodes::LDA_ABS_X => (instructions::lda, addressing::absolute_indexed_x, 4),
            opcodes::LDA_ABS_Y => (instructions::lda, addressing::absolute_indexed_y, 4),
            opcodes::LDA_IX_IND => (instructions::lda, addressing::indexed_indirect, 6),
            opcodes::LDA_IND_IX => (instructions::lda, addressing::indirect_indexed, 5),

            // LDX
            opcodes::LDX_IMM => (instructions::ldx, addressing::immediate, 2),
            opcodes::LDX_ZPG => (instructions::ldx, addressing::zero_page, 3),
            opcodes::LDX_ZPG_Y => (instructions::ldx, addressing::zero_page_indexed_y, 4),
            opcodes::LDX_ABS => (instructions::ldx, addressing::absolute, 4),
            opcodes::LDX_ABS_Y => (instructions::ldx, addressing::absolute_indexed_y, 4),

            // LDY
            opcodes::LDY_IMM => (instructions::ldy, addressing::immediate, 2),
            opcodes::LDY_ZPG => (instructions::ldy, addressing::zero_page, 3),
            opcodes::LDY_ZPG_X => (instructions::ldy, addressing::zero_page_indexed, 4),
            opcodes::LDY_ABS => (instructions::ldy, addressing::absolute, 4),
            opcodes::LDY_ABS_X => (instructions::ldy, addressing::absolute_indexed_x, 4),

            // LSR
            opcodes::LSR_A => (instructions::lsra, addressing::implied, 2),
            opcodes::LSR_ZPG => (instructions::lsr, addressing::zero_page, 5),
            opcodes::LSR_ZPG_X => (instructions::lsr, addressing::zero_page_indexed, 6),
            opcodes::LSR_ABS => (instructions::lsr, addressing::absolute, 6),
            opcodes::LSR_ABS_X => (instructions::lsr, addressing::absolute_indexed_x, 7),

            // NOP
            opcodes::NOP => (instructions::nop, addressing::implied, 2),

            // ORA
            opcodes::ORA_IMM => (instructions::ora, addressing::immediate, 2),
            opcodes::ORA_ZPG => (instructions::ora, addressing::zero_page, 3),
            opcodes::ORA_ZPG_X => (instructions::ora, addressing::zero_page_indexed, 4),
            opcodes::ORA_ABS => (instructions::ora, addressing::absolute, 4),
            opcodes::ORA_ABS_X => (instructions::ora, addressing::absolute_indexed_x, 4),
            opcodes::ORA_ABS_Y => (instructions::ora, addressing::absolute_indexed_y, 4),
            opcodes::ORA_IX_IND => (instructions::ora, addressing::indexed_indirect, 6),
            opcodes::ORA_IND_IX => (instructions::ora, addressing::indirect_indexed, 5),

            // PHA, PLA, PHP, PLP
            opcodes::PHA => (instructions::pha, addressing::implied, 3),
            opcodes::PLA => (instructions::pla, addressing::implied, 4),
            opcodes::PHP => (instructions::php, addressing::implied, 3),
            opcodes::PLP => (instructions::plp, addressing::implied, 4),

            // ROL
            opcodes::ROL_A => (instructions::rola, addressing::implied, 2),
            opcodes::ROL_ZPG => (instructions::rol, addressing::zero_page, 5),
            opcodes::ROL_ZPG_X => (instructions::rol, addressing::zero_page_indexed, 6),
            opcodes::ROL_ABS => (instructions::rol, addressing::absolute, 6),
            opcodes::ROL_ABS_X => (instructions::rol, addressing::absolute_indexed_x, 7),

            // ROR
            opcodes::ROR_A => (instructions::rora, addressing::implied, 2),
            opcodes::ROR_ZPG => (instructions::ror, addressing::zero_page, 5),
            opcodes::ROR_ZPG_X => (instructions::ror, addressing::zero_page_indexed, 6),
            opcodes::ROR_ABS => (instructions::ror, addressing::absolute, 6),
            opcodes::ROR_ABS_X => (instructions::ror, addressing::absolute_indexed_x, 7),

            // RTI, RTS
            opcodes::RTI => (instructions::rti, addressing::implied, 6),
            opcodes::RTS => (instructions::rts, addressing::implied, 6),

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
            opcodes::SEC => (instructions::sec, addressing::implied, 2),
            opcodes::SED => (instructions::sed, addressing::implied, 2),
            opcodes::SEI => (instructions::sei, addressing::implied, 2),

            // STA
            opcodes::STA_ZPG => (instructions::sta, addressing::zero_page, 3),
            opcodes::STA_ZPG_X => (instructions::sta, addressing::zero_page_indexed, 4),
            opcodes::STA_ABS => (instructions::sta, addressing::absolute, 4),
            opcodes::STA_ABS_X => (instructions::sta, addressing::absolute_indexed_x, 5),
            opcodes::STA_ABS_Y => (instructions::sta, addressing::absolute_indexed_y, 5),
            opcodes::STA_IX_IND => (instructions::sta, addressing::indexed_indirect, 6),
            opcodes::STA_IND_IX => (instructions::sta, addressing::indirect_indexed, 6),

            // STX
            opcodes::STX_ZPG => (instructions::stx, addressing::zero_page, 3),
            opcodes::STX_ZPG_Y => (instructions::stx, addressing::zero_page_indexed_y, 4),
            opcodes::STX_ABS => (instructions::stx, addressing::absolute, 4),

            // STY
            opcodes::STY_ZPG => (instructions::sty, addressing::zero_page, 3),
            opcodes::STY_ZPG_X => (instructions::sty, addressing::zero_page_indexed, 4),
            opcodes::STY_ABS => (instructions::sty, addressing::absolute, 4),

            // TAX, TXA, TAY, TYA, TSX, TXS
            opcodes::TAX => (instructions::tax, addressing::implied, 2),
            opcodes::TXA => (instructions::txa, addressing::implied, 2),
            opcodes::TAY => (instructions::tay, addressing::implied, 2),
            opcodes::TYA => (instructions::tya, addressing::implied, 2),
            opcodes::TSX => (instructions::tsx, addressing::implied, 2),
            opcodes::TXS => (instructions::txs, addressing::implied, 2),

            _ => panic!("Unknown opcode: {:X}", opcode)
        }
    }

    fn load_memory(&mut self, address: u16) -> u8 {
        self.memory.read(address)
    }

    fn store_memory(&mut self, address: u16, byte: u8) {
        self.memory.write(address, byte);
    }

    fn stack_push(&mut self, byte: u8) {
        let addr = 0x0100 | (self.sp as u16);
        self.sp = self.sp.wrapping_sub(1);
        self.store_memory(addr, byte);
    }

    fn stack_pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let addr = 0x0100 | (self.sp as u16);
        self.load_memory(addr)
    }

    fn load_vector_to_pc(&mut self, vector: u16) {
        let vector_low = self.load_memory(vector);
        let vector_high = self.load_memory(vector + 1);
        self.pc = util::combine_bytes(vector_high, vector_low);
    }
}
