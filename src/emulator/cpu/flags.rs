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
