use crate::emulator::test::assert_image;
use crate::emulator::test::load_and_run_blargg_test_rom;
use crate::emulator::test::prepare_ete_test;
use crate::emulator::test::run_for;
use crate::emulator::test::test_resource_path;

#[test]
fn test_instr_timing_1() {
    let path = test_resource_path("instr_timing/rom_singles/1-instr_timing.nes");
    let (mut nes, _, mut image) = prepare_ete_test(&path);

    // This test tests official instructions followed by unofficial.
    // Since we don't implement unofficial instructions, we need to run for just enough CPU cycles until
    // the image proves we're done with official instructions.
    // Note: this is a very long test.
    run_for(&mut nes, 220_500_000);
    assert_image(
        &mut image,
        test_resource_path("instr_timing/1-instr_timing.bmp"),
    );
}

#[test]
fn test_instr_timing_2() {
    let path = test_resource_path("instr_timing/rom_singles/2-branch_timing.nes");
    let (status, output) = load_and_run_blargg_test_rom(path);

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n2-branch_timing\n\nPassed\n");
}
