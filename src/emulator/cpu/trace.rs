use std::io::Write;

use emulator::cpu::opcodes;

pub fn write_trace_frame<W : Write>(w: &mut W, frame: &[u8]) {
    if let [a, x, y, sp, pch, pcl, p, opcode, arg1, arg2] = frame {
        write!(w, "{:02X}{:02X}  ", pch, pcl);
        write!(w, "{}", format_instruction(*opcode, *arg1, *arg2));
        write!(w, "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}", a, x, y, p, sp);
    }
}

fn format_implied() -> String {
    return format!("");
}

fn format_immediate(arg: u8) -> String {
    return format!("#${:02X}", arg);
}

fn format_zero_page(arg: u8) -> String {
    return format!("${:02X}", arg);
}

// Note: identical to zero page formatting, but keeping both for clarity.
fn format_relative(arg: u8) -> String {
    return format!("${:02X}", arg);
}

fn format_zero_page_x(arg: u8) -> String {
    return format!("${:02X},X", arg);
}

fn format_zero_page_y(arg: u8) -> String {
    return format!("${:02X},Y", arg);
}

fn format_absolute(arg1: u8, arg2: u8) -> String {
    return format!("${:02X}{:02X}", arg1, arg2);
}

fn format_absolute_x(arg1: u8, arg2: u8) -> String {
    return format!("${:02X}{:02X},X", arg1, arg2);
}

fn format_absolute_y(arg1: u8, arg2: u8) -> String {
    return format!("${:02X}{:02X},Y", arg1, arg2);
}

fn format_indexed_indirect(arg: u8) -> String {
    return format!("(${:02X},X)", arg);
}

fn format_indirect_indexed(arg: u8) -> String {
    return format!("(${:02X}),Y", arg);
}

fn format_indirect(arg1: u8, arg2: u8) -> String {
    return format!("(${:02X}{:02X})", arg1, arg2);
}

pub fn format_instruction(opcode: u8, b1: u8, b2: u8) -> String {
    let (opstring, num_args, human) = match opcode {
        // ADC
        opcodes::ADC_IMM => ("ADC", 1, format_immediate(b1)),
        opcodes::ADC_ZPG => ("ADC", 1, format_zero_page(b1)),
        opcodes::ADC_ZPG_X => ("ADC", 1, format_zero_page_x(b1)),
        opcodes::ADC_ABS => ("ADC", 2, format_absolute(b2, b1)),
        opcodes::ADC_ABS_X => ("ADC", 2, format_absolute_x(b2, b1)),
        opcodes::ADC_ABS_Y => ("ADC", 2, format_absolute_y(b2, b1)),
        opcodes::ADC_IX_IND => ("ADC", 1, format_indexed_indirect(b1)),
        opcodes::ADC_IND_IX => ("ADC", 1, format_indirect_indexed(b1)),

        // AND
        opcodes::AND_IMM => ("AND", 1, format_immediate(b1)),
        opcodes::AND_ZPG => ("AND", 1, format_zero_page(b1)),
        opcodes::AND_ZPG_X => ("AND", 1, format_zero_page_x(b1)),
        opcodes::AND_ABS => ("AND", 2, format_absolute(b2, b1)),
        opcodes::AND_ABS_X => ("AND", 2, format_absolute_x(b2, b1)),
        opcodes::AND_ABS_Y => ("AND", 2, format_absolute_y(b2, b1)),
        opcodes::AND_IX_IND => ("AND", 1, format_indexed_indirect(b1)),
        opcodes::AND_IND_IX => ("AND", 1, format_indirect_indexed(b1)),

        // ASL
        opcodes::ASL_A => ("ASL", 0, format_implied()),
        opcodes::ASL_ZPG => ("ASL", 1, format_zero_page(b1)),
        opcodes::ASL_ZPG_X => ("ASL", 1, format_zero_page_x(b1)),
        opcodes::ASL_ABS => ("ASL", 2, format_absolute(b2, b1)),
        opcodes::ASL_ABS_X => ("ASL", 2, format_absolute_x(b2, b1)),

        // BCC, BCS, BEQ
        opcodes::BCC => ("BCC", 1, format_relative(b1)),
        opcodes::BCS => ("BCS", 1, format_relative(b1)),
        opcodes::BEQ => ("BEQ", 1, format_relative(b1)),
        //
        // BIT
        opcodes::BIT_ZPG => ("BIT", 1, format_zero_page(b1)),
        opcodes::BIT_ABS => ("BIT", 2, format_absolute(b2, b1)),

        // BMI, BNE, BPL, BVC, BVS
        opcodes::BMI => ("BMI", 1, format_relative(b1)),
        opcodes::BNE => ("BNE", 1, format_relative(b1)),
        opcodes::BPL => ("BPL", 1, format_relative(b1)),
        opcodes::BVC => ("BVC", 1, format_relative(b1)),
        opcodes::BVS => ("BVS", 1, format_relative(b1)),

        // BRK
        opcodes::BRK => ("BRK", 0, format_implied()),

        // CLC, CLD, CLI
        opcodes::CLC => ("CLC", 0, format_implied()),
        opcodes::CLD => ("CLD", 0, format_implied()),
        opcodes::CLI => ("CLI", 0, format_implied()),
        opcodes::CLV => ("CLV", 0, format_implied()),

        // CMP
        opcodes::CMP_IMM => ("CMP", 1, format_immediate(b1)),
        opcodes::CMP_ZPG => ("CMP", 1, format_zero_page(b1)),
        opcodes::CMP_ZPG_X => ("CMP", 1, format_zero_page_x(b1)),
        opcodes::CMP_ABS => ("CMP", 2, format_absolute(b2, b1)),
        opcodes::CMP_ABS_X => ("CMP", 2, format_absolute_x(b2, b1)),
        opcodes::CMP_ABS_Y => ("CMP", 2, format_absolute_y(b2, b1)),
        opcodes::CMP_IX_IND => ("CMP", 1, format_indexed_indirect(b1)),
        opcodes::CMP_IND_IX => ("CMP", 1, format_indirect_indexed(b1)),

        // CPX
        opcodes::CPX_IMM => ("CPX", 1, format_immediate(b1)),
        opcodes::CPX_ZPG => ("CPX", 1, format_zero_page(b1)),
        opcodes::CPX_ABS => ("CPX", 2, format_absolute(b2, b1)),

        // CPY
        opcodes::CPY_IMM => ("CPY", 1, format_immediate(b1)),
        opcodes::CPY_ZPG => ("CPY", 1, format_zero_page(b1)),
        opcodes::CPY_ABS => ("CPY", 2, format_absolute(b2, b1)),

        // DEC
        opcodes::DEC_ZPG => ("DEC", 1, format_zero_page(b1)),
        opcodes::DEC_ZPG_X => ("DEC", 1, format_zero_page_x(b1)),
        opcodes::DEC_ABS => ("DEC", 2, format_absolute(b2, b1)),
        opcodes::DEC_ABS_X => ("DEC", 2, format_absolute_x(b2, b1)),

        // DEX, INY
        opcodes::DEX => ("DEX", 0, format_implied()),
        opcodes::DEY => ("DEY", 0, format_implied()),

        // EOR
        opcodes::EOR_IMM => ("EOR", 1, format_immediate(b1)),
        opcodes::EOR_ZPG => ("EOR", 1, format_zero_page(b1)),
        opcodes::EOR_ZPG_X => ("EOR", 1, format_zero_page_x(b1)),
        opcodes::EOR_ABS => ("EOR", 2, format_absolute(b2, b1)),
        opcodes::EOR_ABS_X => ("EOR", 2, format_absolute_x(b2, b1)),
        opcodes::EOR_ABS_Y => ("EOR", 2, format_absolute_y(b2, b1)),
        opcodes::EOR_IX_IND => ("EOR", 1, format_indexed_indirect(b1)),
        opcodes::EOR_IND_IX => ("EOR", 1, format_indirect_indexed(b1)),

        // INC
        opcodes::INC_ZPG => ("INC", 1, format_zero_page(b1)),
        opcodes::INC_ZPG_X => ("INC", 1, format_zero_page_x(b1)),
        opcodes::INC_ABS => ("INC", 2, format_absolute(b2, b1)),
        opcodes::INC_ABS_X => ("INC", 2, format_absolute_x(b2, b1)),

        // INX, INY
        opcodes::INX => ("INX", 0, format_implied()),
        opcodes::INY => ("INY", 0, format_implied()),

        // JMP
        opcodes::JMP_ABS => ("JMP", 2, format_absolute(b2, b1)),
        opcodes::JMP_IND => ("JMP", 2, format_indirect(b2, b1)),

        // JSR
        opcodes::JSR => ("JSR", 2, format_absolute(b2, b1)),

        // LDA
        opcodes::LDA_IMM => ("LDA", 1, format_immediate(b1)),
        opcodes::LDA_ZPG => ("LDA", 1, format_zero_page(b1)),
        opcodes::LDA_ZPG_X => ("LDA", 1, format_zero_page_x(b1)),
        opcodes::LDA_ABS => ("LDA", 2, format_absolute(b2, b1)),
        opcodes::LDA_ABS_X => ("LDA", 2, format_absolute_x(b2, b1)),
        opcodes::LDA_ABS_Y => ("LDA", 2, format_absolute_y(b2, b1)),
        opcodes::LDA_IX_IND => ("LDA", 1, format_indexed_indirect(b1)),
        opcodes::LDA_IND_IX => ("LDA", 1, format_indirect_indexed(b1)),

        // LDX
        opcodes::LDX_IMM => ("LDX", 1, format_immediate(b1)),
        opcodes::LDX_ZPG => ("LDX", 1, format_zero_page(b1)),
        opcodes::LDX_ZPG_Y => ("LDX", 1, format_zero_page_y(b1)),
        opcodes::LDX_ABS => ("LDX", 2, format_absolute(b2, b1)),
        opcodes::LDX_ABS_Y => ("LDX", 2, format_absolute_y(b2, b1)),

        // LDY
        opcodes::LDY_IMM => ("LDY", 1, format_immediate(b1)),
        opcodes::LDY_ZPG => ("LDY", 1, format_zero_page(b1)),
        opcodes::LDY_ZPG_X => ("LDY", 1, format_zero_page_x(b1)),
        opcodes::LDY_ABS => ("LDY", 2, format_absolute(b2, b1)),
        opcodes::LDY_ABS_X => ("LDY", 2, format_absolute_x(b2, b1)),

        // LSR
        opcodes::LSR_A => ("LSR", 0, format_implied()),
        opcodes::LSR_ZPG => ("LSR", 1, format_zero_page(b1)),
        opcodes::LSR_ZPG_X => ("LSR", 1, format_zero_page_x(b1)),
        opcodes::LSR_ABS => ("LSR", 2, format_absolute(b2, b1)),
        opcodes::LSR_ABS_X => ("LSR", 2, format_absolute_x(b2, b1)),

        // NOP
        opcodes::NOP => ("NOP", 0, format_implied()),

        // ORA
        opcodes::ORA_IMM => ("ORA", 1, format_immediate(b1)),
        opcodes::ORA_ZPG => ("ORA", 1, format_zero_page(b1)),
        opcodes::ORA_ZPG_X => ("ORA", 1, format_zero_page_x(b1)),
        opcodes::ORA_ABS => ("ORA", 2, format_absolute(b2, b1)),
        opcodes::ORA_ABS_X => ("ORA", 2, format_absolute_x(b2, b1)),
        opcodes::ORA_ABS_Y => ("ORA", 2, format_absolute_y(b2, b1)),
        opcodes::ORA_IX_IND => ("ORA", 1, format_indexed_indirect(b1)),
        opcodes::ORA_IND_IX => ("ORA", 1, format_indirect_indexed(b1)),

        // PHA, PLA, PHP, PLP
        opcodes::PHA => ("PHA", 0, format_implied()),
        opcodes::PLA => ("PLA", 0, format_implied()),
        opcodes::PHP => ("PHP", 0, format_implied()),
        opcodes::PLP => ("PLP", 0, format_implied()),

        // ROL
        opcodes::ROL_A => ("ROL", 0, format_implied()),
        opcodes::ROL_ZPG => ("ROL", 1, format_zero_page(b1)),
        opcodes::ROL_ZPG_X => ("ROL", 1, format_zero_page_x(b1)),
        opcodes::ROL_ABS => ("ROL", 2, format_absolute(b2, b1)),
        opcodes::ROL_ABS_X => ("ROL", 2, format_absolute_x(b2, b1)),

        // ROR
        opcodes::ROR_A => ("ROR", 0, format_implied()),
        opcodes::ROR_ZPG => ("ROR", 1, format_zero_page(b1)),
        opcodes::ROR_ZPG_X => ("ROR", 1, format_zero_page_x(b1)),
        opcodes::ROR_ABS => ("ROR", 2, format_absolute(b2, b1)),
        opcodes::ROR_ABS_X => ("ROR", 2, format_absolute_x(b2, b1)),

        // RTI, RTS
        opcodes::RTI => ("RTI", 0, format_implied()),
        opcodes::RTS => ("RTS", 0, format_implied()),

        // SBC
        opcodes::SBC_IMM => ("SBC", 1, format_immediate(b1)),
        opcodes::SBC_ZPG => ("SBC", 1, format_zero_page(b1)),
        opcodes::SBC_ZPG_X => ("SBC", 1, format_zero_page_x(b1)),
        opcodes::SBC_ABS => ("SBC", 2, format_absolute(b2, b1)),
        opcodes::SBC_ABS_X => ("SBC", 2, format_absolute_x(b2, b1)),
        opcodes::SBC_ABS_Y => ("SBC", 2, format_absolute_y(b2, b1)),
        opcodes::SBC_IX_IND => ("SBC", 1, format_indexed_indirect(b1)),
        opcodes::SBC_IND_IX => ("SBC", 1, format_indirect_indexed(b1)),

        // SEC, SED, SEI
        opcodes::SEC => ("SEC", 0, format_implied()),
        opcodes::SED => ("SED", 0, format_implied()),
        opcodes::SEI => ("SEI", 0, format_implied()),

        // STA
        opcodes::STA_ZPG => ("STA", 1, format_zero_page(b1)),
        opcodes::STA_ZPG_X => ("STA", 1, format_zero_page_x(b1)),
        opcodes::STA_ABS => ("STA", 2, format_absolute(b2, b1)),
        opcodes::STA_ABS_X => ("STA", 2, format_absolute_x(b2, b1)),
        opcodes::STA_ABS_Y => ("STA", 2, format_absolute_y(b2, b1)),
        opcodes::STA_IX_IND => ("STA", 1, format_indexed_indirect(b1)),
        opcodes::STA_IND_IX => ("STA", 1, format_indirect_indexed(b1)),

        // STX
        opcodes::STX_ZPG => ("STX", 1, format_zero_page(b1)),
        opcodes::STX_ZPG_Y => ("STX", 1, format_zero_page_y(b1)),
        opcodes::STX_ABS => ("STX", 2, format_absolute(b2, b1)),

        // STY
        opcodes::STY_ZPG => ("STY", 1, format_zero_page(b1)),
        opcodes::STY_ZPG_X => ("STY", 1, format_zero_page_x(b1)),
        opcodes::STY_ABS => ("STY", 2, format_absolute(b2, b1)),

        // TAX, TXA, TAY, TYA, TSX, TXS
        opcodes::TAX => ("TAX", 0, format_implied()),
        opcodes::TXA => ("TXA", 0, format_implied()),
        opcodes::TAY => ("TAY", 0, format_implied()),
        opcodes::TYA => ("TYA", 0, format_implied()),
        opcodes::TSX => ("TSX", 0, format_implied()),
        opcodes::TXS => ("TXS", 0, format_implied()),

        _ => panic!("Unknown opcode: {:X}", opcode)
    };

    let mut output = format!("{:02X} ", opcode);
    let b1_str = if num_args >= 1 { format!("{:02X} ", b1) } else { String::from("   ") };
    output.push_str(&b1_str);
    let b2_str = if num_args >= 2 { format!("{:02X}  ", b2) } else { String::from("    ") };
    output.push_str(&b2_str);
    output.push_str(&format!("{} {:<28}", opstring, human));

    output
}

// And parsing functions.
pub fn parse_pc(line: &str) -> u16 {
    u16::from_str_radix(&line[..4], 16).unwrap()
}

pub fn parse_opcode(line: &str) -> u8 {
    u8::from_str_radix(&line[6..8], 16).unwrap()
}

pub fn parse_instruction_byte_1(line: &str) -> u8 {
    u8::from_str_radix(&line[9..11], 16).unwrap()
}

pub fn parse_instruction_byte_2(line: &str) -> u8 {
    u8::from_str_radix(&line[12..14], 16).unwrap()
}

pub fn parse_a(line: &str) -> u8 {
    u8::from_str_radix(&line[50..52], 16).unwrap()
}

pub fn parse_x(line: &str) -> u8 {
    u8::from_str_radix(&line[55..57], 16).unwrap()
}

pub fn parse_y(line: &str) -> u8 {
    u8::from_str_radix(&line[60..62], 16).unwrap()
}

pub fn parse_p(line: &str) -> u8 {
    u8::from_str_radix(&line[65..67], 16).unwrap()
}

pub fn parse_sp(line: &str) -> u8 {
    u8::from_str_radix(&line[71..73], 16).unwrap()
}

pub fn parse_cyc(line: &str) -> u64 {
    u64::from_str_radix(&line[78..81].trim(), 10).unwrap()
}
