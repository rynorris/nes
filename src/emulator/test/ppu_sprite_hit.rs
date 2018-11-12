use emulator::test::load_and_run_blargg_test_rom;
use emulator::test::test_resource_path;

// -- ppu_sprite_hit test ROMs --
// TODO: Get test 09 to pass once my CPU/PPU timing is PPU-cycle accurate.
#[test]
fn test_ppu_sprite_hit_01() {
    let path = test_resource_path("ppu_sprite_hit/rom_singles/01-basics.nes");
    let (status, output) = load_and_run_blargg_test_rom(path);

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n01-basics\n\nPassed\n");
}

#[test]
fn test_ppu_sprite_hit_02() {
    let path = test_resource_path("ppu_sprite_hit/rom_singles/02-alignment.nes");
    let (status, output) = load_and_run_blargg_test_rom(path);

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n02-alignment\n\nPassed\n");
}

#[test]
fn test_ppu_sprite_hit_03() {
    let path = test_resource_path("ppu_sprite_hit/rom_singles/03-corners.nes");
    let (status, output) = load_and_run_blargg_test_rom(path);

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n03-corners\n\nPassed\n");
}

#[test]
fn test_ppu_sprite_hit_04() {
    let path = test_resource_path("ppu_sprite_hit/rom_singles/04-flip.nes");
    let (status, output) = load_and_run_blargg_test_rom(path);

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n04-flip\n\nPassed\n");
}

#[test]
fn test_ppu_sprite_hit_05() {
    let path = test_resource_path("ppu_sprite_hit/rom_singles/05-left_clip.nes");
    let (status, output) = load_and_run_blargg_test_rom(path);

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n05-left_clip\n\nPassed\n");
}

#[test]
fn test_ppu_sprite_hit_06() {
    let path = test_resource_path("ppu_sprite_hit/rom_singles/06-right_edge.nes");
    let (status, output) = load_and_run_blargg_test_rom(path);

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n06-right_edge\n\nPassed\n");
}

#[test]
fn test_ppu_sprite_hit_07() {
    let path = test_resource_path("ppu_sprite_hit/rom_singles/07-screen_bottom.nes");
    let (status, output) = load_and_run_blargg_test_rom(path);

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n07-screen_bottom\n\nPassed\n");
}

#[test]
fn test_ppu_sprite_hit_08() {
    let path = test_resource_path("ppu_sprite_hit/rom_singles/08-double_height.nes");
    let (status, output) = load_and_run_blargg_test_rom(path);

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n08-double_height\n\nPassed\n");
}

#[test]
fn test_ppu_sprite_hit_10() {
    let path = test_resource_path("ppu_sprite_hit/rom_singles/10-timing_order.nes");
    let (status, output) = load_and_run_blargg_test_rom(path);

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n10-timing_order\n\nPassed\n");
}
