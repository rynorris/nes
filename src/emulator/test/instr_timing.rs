use std::cell::RefCell;
use std::rc::Rc;

use emulator::ines;
use emulator::io;
use emulator::io::event::EventBus;
use emulator::io::sdl::ImageCapture;
use emulator::NES;

use emulator::test::assert_image;
use emulator::test::load_and_run_blargg_test_rom;
use emulator::test::run_for;
use emulator::test::test_resource_path;


#[test]
fn test_instr_timing_1() {
    let path = test_resource_path("instr_timing/rom_singles/1-instr_timing.nes");
    let rom = ines::ROM::load(&path.into_os_string().into_string().unwrap());
    let event_bus = Rc::new(RefCell::new(EventBus::new()));
    let output = Rc::new(RefCell::new(io::Screen::new()));
    let mut image = ImageCapture::new(output.clone());
    let audio = io::nop::DummyAudio{};
    let mut nes = NES::new(event_bus.clone(), output.clone(), audio, rom);

    // This test tests official instructions followed by unofficial.
    // Since we don't implement unofficial instructions, we need to run for just enough CPU cycles until
    // the image proves we're done with official instructions.
    // Note: this is a very long test.
    run_for(&mut nes, 220_500_000);
    assert_image(&mut image, test_resource_path("instr_timing/1-instr_timing.bmp"));
}

#[test]
fn test_instr_timing_2() {
    let path = test_resource_path("instr_timing/rom_singles/2-branch_timing.nes");
    let (status, output) = load_and_run_blargg_test_rom(path);

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n2-branch_timing\n\nPassed\n");
}
