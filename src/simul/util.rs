pub fn combine_bytes(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) + (low as u16)
}

#[test]
fn test_combine_bytes() {
    assert_eq!(combine_bytes(0x12, 0xAB), 0x12AB);
}
