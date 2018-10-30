#[derive(Debug)]
pub struct BitField {
    byte: u8,
}

impl BitField {
    pub fn new() -> BitField {
        BitField{byte: 0x00}
    }

    pub fn is_set<F : Into<u8>>(&self, mask: F) -> bool {
        self.byte & (mask.into()) != 0
    }

    pub fn set<F : Into<u8>>(&mut self, mask: F) {
        self.byte = self.byte | mask.into();
    }

    pub fn clear<F : Into<u8>>(&mut self, mask: F) {
        self.byte = self.byte & !mask.into();
    }

    pub fn as_byte(&self) -> u8 {
        self.byte
    }

    pub fn load_byte(&mut self, byte: u8) {
        self.byte = byte;
    }
}

#[test]
fn tests() {
    let mut bits = BitField::new();
    let bit1: u8 = 1;
    let bit2: u8 = 1 << 1;
    assert_eq!(bits.is_set(bit1), false);
    assert_eq!(bits.is_set(bit2), false);

    bits.set(bit2);
    assert_eq!(bits.is_set(bit1), false);
    assert_eq!(bits.is_set(bit2), true);

    bits.clear(bit1);
    assert_eq!(bits.is_set(bit1), false);
    assert_eq!(bits.is_set(bit2), true);

    bits.set(bit1);
    assert_eq!(bits.is_set(bit1), true);
    assert_eq!(bits.is_set(bit2), true);

    bits.clear(bit2);
    assert_eq!(bits.is_set(bit1), true);
    assert_eq!(bits.is_set(bit2), false);

    bits.clear(bit1);
    assert_eq!(bits.is_set(bit1), false);
    assert_eq!(bits.is_set(bit2), false);
}


