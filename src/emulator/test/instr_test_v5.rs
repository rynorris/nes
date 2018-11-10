use emulator::test::load_and_run_blargg_test_rom_with_cycles;
use emulator::test::test_resource_path;

#[test]
fn test_instr_test_v5_official_only() {
    let path = test_resource_path("instr_test-v5/official_only.nes");
    let (status, output) = load_and_run_blargg_test_rom_with_cycles(path, 1_000_000_000);

    assert_eq!(status, 0x00);
    assert_eq!(output, "All 16 tests passed\n\n\n");
}
