use emulator::cpu::opcodes;

fn format_implied(opstring: &str) -> String {
    return format!("{}", opstring);
}

fn format_immediate(opstring: &str, arg: u8) -> String {
    return format!("{} #${:02X}", opstring, arg);
}

fn format_zero_page(opstring: &str, arg: u8) -> String {
    return format!("{} ${:02X}", opstring, arg);
}

// Note: identical to zero page formatting, but keeping both for clarity.
fn format_relative(opstring: &str, arg: u8) -> String {
    return format!("{} ${:02X}", opstring, arg);
}

fn format_zero_page_x(opstring: &str, arg: u8) -> String {
    return format!("{} ${:02X},X", opstring, arg);
}

fn format_zero_page_y(opstring: &str, arg: u8) -> String {
    return format!("{} ${:02X},Y", opstring, arg);
}

fn format_absolute(opstring: &str, arg1: u8, arg2: u8) -> String {
    return format!("{} ${:02X}{:02X}", opstring, arg1, arg2);
}

fn format_absolute_x(opstring: &str, arg1: u8, arg2: u8) -> String {
    return format!("{} ${:02X}{:02X},X", opstring, arg1, arg2);
}

fn format_absolute_y(opstring: &str, arg1: u8, arg2: u8) -> String {
    return format!("{} ${:02X}{:02X},Y", opstring, arg1, arg2);
}

fn format_indexed_indirect(opstring: &str, arg: u8) -> String {
    return format!("{} (${:02X},X)", opstring, arg);
}

fn format_indirect_indexed(opstring: &str, arg: u8) -> String {
    return format!("{} (${:02X}),Y", opstring, arg);
}

fn format_indirect(opstring: &str, arg1: u8, arg2: u8) -> String {
    return format!("{} (${:02X}{:02X})", opstring, arg1, arg2);
}

pub fn format_instruction(opcode: u8, b1: u8, b2: u8) -> String {
    match opcode {
        // ADC
        opcodes::ADC_IMM => format_immediate("ADC", b1),
        opcodes::ADC_ZPG => format_zero_page("ADC", b1),
        opcodes::ADC_ZPG_X => format_zero_page_x("ADC", b1),
        opcodes::ADC_ABS => format_absolute("ADC", b2, b1),
        opcodes::ADC_ABS_X => format_absolute_x("ADC", b2, b1),
        opcodes::ADC_ABS_Y => format_absolute_y("ADC", b2, b1),
        opcodes::ADC_IX_IND => format_indexed_indirect("ADC", b1),
        opcodes::ADC_IND_IX => format_indirect_indexed("ADC", b1),

        // AND
        opcodes::AND_IMM => format_immediate("AND", b1),
        opcodes::AND_ZPG => format_zero_page("AND", b1),
        opcodes::AND_ZPG_X => format_zero_page_x("AND", b1),
        opcodes::AND_ABS => format_absolute("AND", b2, b1),
        opcodes::AND_ABS_X => format_absolute_x("AND", b2, b1),
        opcodes::AND_ABS_Y => format_absolute_y("AND", b2, b1),
        opcodes::AND_IX_IND => format_indexed_indirect("AND", b1),
        opcodes::AND_IND_IX => format_indirect_indexed("AND", b1),

        // ASL
        opcodes::ASL_A => format_implied("ASL"),
        opcodes::ASL_ZPG => format_zero_page("ASL", b1),
        opcodes::ASL_ZPG_X => format_zero_page_x("ASL", b1),
        opcodes::ASL_ABS => format_absolute("ASL", b2, b1),
        opcodes::ASL_ABS_X => format_absolute_x("ASL", b2, b1),

        // BCC, BCS, BEQ
        opcodes::BCC => format_relative("BCC", b1),
        opcodes::BCS => format_relative("BCS", b1),
        opcodes::BEQ => format_relative("BEQ", b1),
        //
        // BIT
        opcodes::BIT_ZPG => format_zero_page("BIT", b1),
        opcodes::BIT_ABS => format_absolute("BIT", b2, b1),

        // BMI, BNE, BPL, BVC, BVS
        opcodes::BMI => format_relative("BMI", b1),
        opcodes::BNE => format_relative("BNE", b1),
        opcodes::BPL => format_relative("BPL", b1),
        opcodes::BVC => format_relative("BVC", b1),
        opcodes::BVS => format_relative("BVS", b1),

        // BRK
        opcodes::BRK => format_implied("BRK"),

        // CLC, CLD, CLI
        opcodes::CLC => format_implied("CLC"),
        opcodes::CLD => format_implied("CLD"),
        opcodes::CLI => format_implied("CLI"),
        opcodes::CLV => format_implied("CLV"),

        // CMP
        opcodes::CMP_IMM => format_immediate("CMP", b1),
        opcodes::CMP_ZPG => format_zero_page("CMP", b1),
        opcodes::CMP_ZPG_X => format_zero_page_x("CMP", b1),
        opcodes::CMP_ABS => format_absolute("CMP", b2, b1),
        opcodes::CMP_ABS_X => format_absolute_x("CMP", b2, b1),
        opcodes::CMP_ABS_Y => format_absolute_y("CMP", b2, b1),
        opcodes::CMP_IX_IND => format_indexed_indirect("CMP", b1),
        opcodes::CMP_IND_IX => format_indirect_indexed("CMP", b1),

        // CPX
        opcodes::CPX_IMM => format_immediate("CPX", b1),
        opcodes::CPX_ZPG => format_zero_page("CPX", b1),
        opcodes::CPX_ABS => format_absolute("CPX", b2, b1),

        // CPY
        opcodes::CPY_IMM => format_immediate("CPY", b1),
        opcodes::CPY_ZPG => format_zero_page("CPY", b1),
        opcodes::CPY_ABS => format_absolute("CPY", b2, b1),

        // DEC
        opcodes::DEC_ZPG => format_zero_page("DEC", b1),
        opcodes::DEC_ZPG_X => format_zero_page_x("DEC", b1),
        opcodes::DEC_ABS => format_absolute("DEC", b2, b1),
        opcodes::DEC_ABS_X => format_absolute_x("DEC", b2, b1),

        // DEX, INY
        opcodes::DEX => format_implied("DEX"),
        opcodes::DEY => format_implied("DEY"),

        // EOR
        opcodes::EOR_IMM => format_immediate("EOR", b1),
        opcodes::EOR_ZPG => format_zero_page("EOR", b1),
        opcodes::EOR_ZPG_X => format_zero_page_x("EOR", b1),
        opcodes::EOR_ABS => format_absolute("EOR", b2, b1),
        opcodes::EOR_ABS_X => format_absolute_x("EOR", b2, b1),
        opcodes::EOR_ABS_Y => format_absolute_y("EOR", b2, b1),
        opcodes::EOR_IX_IND => format_indexed_indirect("EOR", b1),
        opcodes::EOR_IND_IX => format_indirect_indexed("EOR", b1),

        // INC
        opcodes::INC_ZPG => format_zero_page("INC", b1),
        opcodes::INC_ZPG_X => format_zero_page_x("INC", b1),
        opcodes::INC_ABS => format_absolute("INC", b2, b1),
        opcodes::INC_ABS_X => format_absolute_x("INC", b2, b1),

        // INX, INY
        opcodes::INX => format_implied("INX"),
        opcodes::INY => format_implied("INY"),

        // JMP
        opcodes::JMP_ABS => format_absolute("JMP", b2, b1),
        opcodes::JMP_IND => format_indirect("JMP", b2, b1),

        // JSR
        opcodes::JSR => format_absolute("JSR", b2, b1),

        // LDA
        opcodes::LDA_IMM => format_immediate("LDA", b1),
        opcodes::LDA_ZPG => format_zero_page("LDA", b1),
        opcodes::LDA_ZPG_X => format_zero_page_x("LDA", b1),
        opcodes::LDA_ABS => format_absolute("LDA", b2, b1),
        opcodes::LDA_ABS_X => format_absolute_x("LDA", b2, b1),
        opcodes::LDA_ABS_Y => format_absolute_y("LDA", b2, b1),
        opcodes::LDA_IX_IND => format_indexed_indirect("LDA", b1),
        opcodes::LDA_IND_IX => format_indirect_indexed("LDA", b1),

        // LDX
        opcodes::LDX_IMM => format_immediate("LDX", b1),
        opcodes::LDX_ZPG => format_zero_page("LDX", b1),
        opcodes::LDX_ZPG_Y => format_zero_page_y("LDX", b1),
        opcodes::LDX_ABS => format_absolute("LDX", b2, b1),
        opcodes::LDX_ABS_Y => format_absolute_y("LDX", b2, b1),

        // LDY
        opcodes::LDY_IMM => format_immediate("LDY", b1),
        opcodes::LDY_ZPG => format_zero_page("LDY", b1),
        opcodes::LDY_ZPG_X => format_zero_page_x("LDY", b1),
        opcodes::LDY_ABS => format_absolute("LDY", b2, b1),
        opcodes::LDY_ABS_X => format_absolute_x("LDY", b2, b1),

        // LSR
        opcodes::LSR_A => format_implied("LSR"),
        opcodes::LSR_ZPG => format_zero_page("LSR", b1),
        opcodes::LSR_ZPG_X => format_zero_page_x("LSR", b1),
        opcodes::LSR_ABS => format_absolute("LSR", b2, b1),
        opcodes::LSR_ABS_X => format_absolute_x("LSR", b2, b1),

        // NOP
        opcodes::NOP => format_implied("NOP"),

        // ORA
        opcodes::ORA_IMM => format_immediate("ORA", b1),
        opcodes::ORA_ZPG => format_zero_page("ORA", b1),
        opcodes::ORA_ZPG_X => format_zero_page_x("ORA", b1),
        opcodes::ORA_ABS => format_absolute("ORA", b2, b1),
        opcodes::ORA_ABS_X => format_absolute_x("ORA", b2, b1),
        opcodes::ORA_ABS_Y => format_absolute_y("ORA", b2, b1),
        opcodes::ORA_IX_IND => format_indexed_indirect("ORA", b1),
        opcodes::ORA_IND_IX => format_indirect_indexed("ORA", b1),

        // PHA, PLA, PHP, PLP
        opcodes::PHA => format_implied("PHA"),
        opcodes::PLA => format_implied("PLA"),
        opcodes::PHP => format_implied("PHP"),
        opcodes::PLP => format_implied("PLP"),

        // ROL
        opcodes::ROL_A => format_implied("ROL"),
        opcodes::ROL_ZPG => format_zero_page("ROL", b1),
        opcodes::ROL_ZPG_X => format_zero_page_x("ROL", b1),
        opcodes::ROL_ABS => format_absolute("ROL", b2, b1),
        opcodes::ROL_ABS_X => format_absolute_x("ROL", b2, b1),

        // ROR
        opcodes::ROR_A => format_implied("ROR"),
        opcodes::ROR_ZPG => format_zero_page("ROR", b1),
        opcodes::ROR_ZPG_X => format_zero_page_x("ROR", b1),
        opcodes::ROR_ABS => format_absolute("ROR", b2, b1),
        opcodes::ROR_ABS_X => format_absolute_x("ROR", b2, b1),

        // RTI, RTS
        opcodes::RTI => format_implied("RTI"),
        opcodes::RTS => format_implied("RTS"),

        // SBC
        opcodes::SBC_IMM => format_immediate("SBC", b1),
        opcodes::SBC_ZPG => format_zero_page("SBC", b1),
        opcodes::SBC_ZPG_X => format_zero_page_x("SBC", b1),
        opcodes::SBC_ABS => format_absolute("SBC", b2, b1),
        opcodes::SBC_ABS_X => format_absolute_x("SBC", b2, b1),
        opcodes::SBC_ABS_Y => format_absolute_y("SBC", b2, b1),
        opcodes::SBC_IX_IND => format_indexed_indirect("SBC", b1),
        opcodes::SBC_IND_IX => format_indirect_indexed("SBC", b1),

        // SEC, SED, SEI
        opcodes::SEC => format_implied("SEC"),
        opcodes::SED => format_implied("SED"),
        opcodes::SEI => format_implied("SEI"),

        // STA
        opcodes::STA_ZPG => format_zero_page("STA", b1),
        opcodes::STA_ZPG_X => format_zero_page_x("STA", b1),
        opcodes::STA_ABS => format_absolute("STA", b2, b1),
        opcodes::STA_ABS_X => format_absolute_x("STA", b2, b1),
        opcodes::STA_ABS_Y => format_absolute_y("STA", b2, b1),
        opcodes::STA_IX_IND => format_indexed_indirect("STA", b1),
        opcodes::STA_IND_IX => format_indirect_indexed("STA", b1),

        // STX
        opcodes::STX_ZPG => format_zero_page("STX", b1),
        opcodes::STX_ZPG_Y => format_zero_page_y("STX", b1),
        opcodes::STX_ABS => format_absolute("STX", b2, b1),

        // STY
        opcodes::STY_ZPG => format_zero_page("STY", b1),
        opcodes::STY_ZPG_X => format_zero_page_x("STY", b1),
        opcodes::STY_ABS => format_absolute("STY", b2, b1),

        // TAX, TXA, TAY, TYA, TSX, TXS
        opcodes::TAX => format_implied("TAX"),
        opcodes::TXA => format_implied("TXA"),
        opcodes::TAY => format_implied("TAY"),
        opcodes::TYA => format_implied("TYA"),
        opcodes::TSX => format_implied("TSX"),
        opcodes::TXS => format_implied("TXS"),

        _ => panic!("Unknown opcode: {:X}", opcode)
    }
}
