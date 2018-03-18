mod instructions;

pub struct CPU {
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
    p: u8,

    /*
    // Address bus
    address_bus: bus::AddressBus,

    // Data bus
    data_bus: bus::DataBus,
    */
}

pub fn new() -> CPU {
    CPU {
        a: 0,
        x: 0,
        y: 0,
        sp: 0,
        pc: 0,
        p: 0,
    }
}

pub enum Flag {
    N = 0b1000_0000, // Negative
    V = 0b0100_0000, // Overflow
    B = 0b0001_0000,
    D = 0b0000_1000, // BCD Mode
    I = 0b0000_0100,
    Z = 0b0000_0010, // Zero
    C = 0b0000_0001, // Carry
}

impl CPU {
    pub fn flag_is_set(&self, flag: Flag) -> bool {
        self.p & (flag as u8) != 0
    }

    pub fn set_flag(&mut self, flag: Flag) {
        self.p = self.p | (flag as u8);
    }

    pub fn clear_flag(&mut self, flag: Flag) {
        self.p = self.p & (0xFF ^ (flag as u8));
    }
}

#[derive(Debug)]
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


#[test]
fn test_flags() {
    let mut cpu = new();
    assert_eq!(cpu.flag_is_set(Flag::N), false);
    assert_eq!(cpu.flag_is_set(Flag::V), false);

    cpu.set_flag(Flag::V);
    assert_eq!(cpu.flag_is_set(Flag::N), false);
    assert_eq!(cpu.flag_is_set(Flag::V), true);

    cpu.clear_flag(Flag::N);
    assert_eq!(cpu.flag_is_set(Flag::N), false);
    assert_eq!(cpu.flag_is_set(Flag::V), true);

    cpu.set_flag(Flag::N);
    assert_eq!(cpu.flag_is_set(Flag::N), true);
    assert_eq!(cpu.flag_is_set(Flag::V), true);

    cpu.clear_flag(Flag::V);
    assert_eq!(cpu.flag_is_set(Flag::N), true);
    assert_eq!(cpu.flag_is_set(Flag::V), false);

    cpu.clear_flag(Flag::N);
    assert_eq!(cpu.flag_is_set(Flag::N), false);
    assert_eq!(cpu.flag_is_set(Flag::V), false);
}
