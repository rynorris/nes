use crate::emulator::test::load_and_run_blargg_test_rom;
use crate::emulator::test::test_resource_path;

// -- instr_misc test ROMs --
// TODO: Get 03 and 04 to pass and add tests for them.
#[test]
fn test_instr_misc_01() {
    let path = test_resource_path("instr_misc/rom_singles/01-abs_x_wrap.nes");
    let (status, output) = load_and_run_blargg_test_rom(path);

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n01-abs_x_wrap\n\nPassed\n");
}

#[test]
fn test_instr_misc_02() {
    let path = test_resource_path("instr_misc/rom_singles/02-branch_wrap.nes");
    let (status, output) = load_and_run_blargg_test_rom(path);

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n02-branch_wrap\n\nPassed\n");
}

fn test_instr_misc_03() {
    let path = test_resource_path("instr_misc/rom_singles/03-dummy_reads.nes");
    let (status, output) = load_and_run_blargg_test_rom(path);

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n02-branch_wrap\n\nPassed\n");
}
