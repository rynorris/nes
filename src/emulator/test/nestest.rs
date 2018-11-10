use std::cell::RefCell;
use std::rc::Rc;

use emulator::ines;
use emulator::io;
use emulator::io::event::{Event, EventBus, Key};
use emulator::io::sdl::ImageCapture;
use emulator::NES;

use emulator::test::assert_image;
use emulator::test::run_for;
use emulator::test::test_resource_path;

// -- Visual nestest.
#[test]
fn test_nestest_visual() {
    let path = test_resource_path("nestest/nestest.nes");
    let rom = ines::ROM::load(&path.into_os_string().into_string().unwrap());
    let event_bus = Rc::new(RefCell::new(EventBus::new()));
    let output = Rc::new(RefCell::new(io::SimpleVideoOut::new()));
    let mut image = ImageCapture::new(output.clone());
    let audio = io::nop::DummyAudio{};
    let mut nes = NES::new(event_bus.clone(), output.clone(), audio, rom);

    // Check the menu load.
    run_for(&mut nes, 550_000);
    assert_image(&mut image, test_resource_path("nestest/capture_01_menu.bmp"));

    // Start tests.
    event_bus.borrow_mut().broadcast(Event::KeyDown(Key::A));
   
    // Wait for tests to finish and check they pass.
    run_for(&mut nes, 2_000_000);
    assert_image(&mut image, test_resource_path("nestest/capture_02_passed.bmp"));
}

