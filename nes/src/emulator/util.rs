pub fn combine_bytes(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) + (low as u16)
}

pub fn split_word(word: u16) -> (u8, u8) {
    // Returns (high, low)
    ((word >> 8) as u8, (word & 0xFF) as u8)
}

pub fn shift_right(byte: u8) -> (u8, bool) {
    let carry = (byte & 0b0000_0001) != 0;
    return (byte >> 1, carry);
}

pub fn shift_left(byte: u8) -> (u8, bool) {
    let carry = (byte & 0b1000_0000) != 0;
    return (byte << 1, carry);
}

pub fn rotate_right(byte: u8, carry: bool) -> (u8, bool) {
    let new_carry = (byte & 0b0000_0001) != 0;
    let rot = (byte >> 1) | (if carry { 0b1000_0000 } else { 0 });
    return (rot, new_carry);
}

pub fn rotate_left(byte: u8, carry: bool) -> (u8, bool) {
    let new_carry = (byte & 0b1000_0000) != 0;
    let rot = (byte << 1) | (if carry { 0b0000_0001 } else { 0 });
    return (rot, new_carry);
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

pub fn reverse_bits(mut byte: u8) -> u8 {
    let mut target = 0x00;
    for _ in 0..8 {
        target <<= 1;
        target |= byte & 0x01;
        byte >>= 1;
    }
    target
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_combine_bytes() {
        assert_eq!(combine_bytes(0x12, 0xAB), 0x12AB);
    }

    #[test]
    fn test_reverse_bits() {
        assert_eq!(reverse_bits(0b1101_0101), 0b1010_1011);
    }

    macro_rules! shr_test {
        ($name:ident: $inp:expr => $out:expr, $cry:expr) => {
            #[test]
            fn $name() {
                let (out, carry) = shift_right($inp);
                assert_eq!(out, $out);
                assert_eq!(carry, $cry);
            }
        };
    }

    shr_test!(test_shr_0: 0x00 => 0x00, false);
    shr_test!(test_shr_carry: 0b0000_0001 => 0b0000_0000, true);
    shr_test!(test_shr_128: 0b1000_0000 => 0b0100_0000, false);

    macro_rules! shl_test {
        ($name:ident: $inp:expr => $out:expr, $cry:expr) => {
            #[test]
            fn $name() {
                let (out, carry) = shift_left($inp);
                assert_eq!(out, $out);
                assert_eq!(carry, $cry);
            }
        };
    }

    shl_test!(test_shl_0: 0x00 => 0x00, false);
    shl_test!(test_shl_1: 0b0000_0001 => 0b0000_0010, false);
    shl_test!(test_shl_carry: 0b1000_0000 => 0b0000_0000, true);

    macro_rules! rtr_test {
        ($name:ident: $inp:expr, $cry:expr => $out:expr, $ncy:expr) => {
            #[test]
            fn $name() {
                let (out, carry) = rotate_right($inp, $cry);
                assert_eq!(out, $out);
                assert_eq!(carry, $ncy);
            }
        };
    }

    rtr_test!(test_rtr_0: 0x00, false => 0x00, false);
    rtr_test!(test_rtr_0_carry: 0x00, true => 0b1000_0000, false);
    rtr_test!(test_rtr_1_carry: 0x01, true => 0b1000_0000, true);
    rtr_test!(test_rtr_128_no_carry: 0b1000_0000, false => 0b0100_0000, false);
    rtr_test!(test_rtr_128_carry: 0b1000_0000, true => 0b1100_0000, false);

    macro_rules! rtl_test {
        ($name:ident: $inp:expr, $cry:expr => $out:expr, $ncy:expr) => {
            #[test]
            fn $name() {
                let (out, carry) = rotate_left($inp, $cry);
                assert_eq!(out, $out);
                assert_eq!(carry, $ncy);
            }
        };
    }

    rtl_test!(test_rtl_0: 0x00, false => 0x00, false);
    rtl_test!(test_rtl_0_carry: 0x00, true => 0x01, false);
    rtl_test!(test_rtl_1_carry: 0x01, true => 0b0000_0011, false);
    rtl_test!(test_rtl_128_no_carry: 0b1000_0000, false => 0b0000_0000, true);
    rtl_test!(test_rtl_128_carry: 0b1000_0000, true => 0b0000_0001, true);

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
}
