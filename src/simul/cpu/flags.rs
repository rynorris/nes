pub enum Flag {
    N = 0b1000_0000, // Negative
    V = 0b0100_0000, // Overflow
    B = 0b0001_0000,
    D = 0b0000_1000, // BCD Mode
    I = 0b0000_0100,
    Z = 0b0000_0010, // Zero
    C = 0b0000_0001, // Carry
}

#[derive(Debug)]
pub struct ProcessorFlags {
    byte: u8,
}

pub fn new() -> ProcessorFlags {
    ProcessorFlags{byte: 0x00}
}

impl ProcessorFlags {
    pub fn is_set(&self, flag: Flag) -> bool {
        self.byte & (flag as u8) != 0
    }

    pub fn set(&mut self, flag: Flag) {
        self.byte = self.byte | (flag as u8);
    }

    pub fn clear(&mut self, flag: Flag) {
        self.byte = self.byte & (0xFF ^ (flag as u8));
    }
}

#[test]
fn tests() {
    let mut flags = new();
    assert_eq!(flags.is_set(Flag::N), false);
    assert_eq!(flags.is_set(Flag::V), false);

    flags.set(Flag::V);
    assert_eq!(flags.is_set(Flag::N), false);
    assert_eq!(flags.is_set(Flag::V), true);

    flags.clear(Flag::N);
    assert_eq!(flags.is_set(Flag::N), false);
    assert_eq!(flags.is_set(Flag::V), true);

    flags.set(Flag::N);
    assert_eq!(flags.is_set(Flag::N), true);
    assert_eq!(flags.is_set(Flag::V), true);

    flags.clear(Flag::V);
    assert_eq!(flags.is_set(Flag::N), true);
    assert_eq!(flags.is_set(Flag::V), false);

    flags.clear(Flag::N);
    assert_eq!(flags.is_set(Flag::N), false);
    assert_eq!(flags.is_set(Flag::V), false);
}

