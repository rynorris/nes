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
