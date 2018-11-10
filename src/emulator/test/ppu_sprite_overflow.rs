use emulator::test::load_and_run_blargg_test_rom;
use emulator::test::test_resource_path;

// -- ppu_sprite_overflow test ROMs --
// TODO: Add test for 03 once my CPU is timed cycle-accurate.
#[test]
fn test_ppu_sprite_overflow_01() {
    let path = test_resource_path("ppu_sprite_overflow/rom_singles/01-basics.nes");
    let (status, output) = load_and_run_blargg_test_rom(path);

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n01-basics\n\nPassed\n");
}

#[test]
fn test_ppu_sprite_overflow_02() {
    let path = test_resource_path("ppu_sprite_overflow/rom_singles/02-details.nes");
    let (status, output) = load_and_run_blargg_test_rom(path);

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n02-details\n\nPassed\n");
}

#[test]
fn test_ppu_sprite_overflow_04() {
    let path = test_resource_path("ppu_sprite_overflow/rom_singles/04-obscure.nes");
    let (status, output) = load_and_run_blargg_test_rom(path);

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n04-obscure\n\nPassed\n");
}

#[test]
fn test_ppu_sprite_overflow_05() {
    let path = test_resource_path("ppu_sprite_overflow/rom_singles/05-emulator.nes");
    let (status, output) = load_and_run_blargg_test_rom(path);

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n05-emulator\n\nPassed\n");
}

