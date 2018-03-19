pub fn combine_bytes(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) + (low as u16)
}

pub fn bcd_to_hex(bcd: u8) -> u8 {
    let high_nibble = bcd >> 4;
    let low_nibble = bcd & 0x0F;
    (high_nibble * 10) + low_nibble
}

pub fn hex_to_bcd(hex: u8) -> u8 {
    if hex > 99 {
        panic!("Integers greater than 99 cannot be represented in a BCD byte.");
    }

    let tens = hex / 10;
    let units = hex % 10;
    (tens << 4) | units
}

#[test]
fn test_combine_bytes() {
    assert_eq!(combine_bytes(0x12, 0xAB), 0x12AB);
}

macro_rules! b2h_test {
    ($name:ident: $bcd:expr => $hex:expr) => {
        #[test]
        fn $name() {
            assert_eq!(bcd_to_hex($bcd), $hex);
        }
    };
}

b2h_test!(test_b2h_0: 0b0000_0000 => 0);
b2h_test!(test_b2h_9: 0b0000_1001 => 9);
b2h_test!(test_b2h_11: 0b0001_0001 => 11);
b2h_test!(test_b2h_98: 0b1001_1000 => 98);

macro_rules! h2b_test {
    ($name:ident: $hex:expr => $bcd:expr) => {
        #[test]
        fn $name() {
            assert_eq!(hex_to_bcd($hex), $bcd);
        }
    };
}

h2b_test!(test_h2b_0: 0 => 0b0000_0000);
h2b_test!(test_h2b_9: 9 => 0b0000_1001);
h2b_test!(test_h2b_11: 11 => 0b0001_0001);
h2b_test!(test_h2b_98: 98 => 0b1001_1000);
