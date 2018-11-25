use std::cell::RefCell;
use std::rc::Rc;

use emulator::ines;
use emulator::io;
use emulator::io::event::{Event, EventBus, Key};
use emulator::io::sdl::ImageCapture;
use emulator::NES;
use emulator::state::SaveState;

use emulator::test::assert_image;
use emulator::test::run_for;
use emulator::test::test_resource_path;

// -- Visual nestest.
#[test]
fn test_nestest_visual() {
    let path = test_resource_path("nestest/nestest.nes");
    let rom = ines::ROM::load(&path.into_os_string().into_string().unwrap());
    let event_bus = Rc::new(RefCell::new(EventBus::new()));
    let output = Rc::new(RefCell::new(io::Screen::new()));
    let mut image = ImageCapture::new(output.clone());
    let audio = io::nop::DummyAudio{};
    let mut nes = NES::new(event_bus.clone(), output.clone(), audio, rom);

    // Check the menu load.
    run_for(&mut nes, 2_000_000);
    assert_image(&mut image, test_resource_path("nestest/capture_01_menu.bmp"));

    // Start tests.
    event_bus.borrow_mut().broadcast(Event::KeyDown(Key::A));
   
    // Wait for tests to finish and check they pass.
    run_for(&mut nes, 7_000_000);
    assert_image(&mut image, test_resource_path("nestest/capture_02_passed.bmp"));
}

#[test]
fn test_nestest_savestate() {
    let path = test_resource_path("nestest/nestest.nes");
    let rom = ines::ROM::load(&path.into_os_string().into_string().unwrap());
    let event_bus = Rc::new(RefCell::new(EventBus::new()));
    let output = Rc::new(RefCell::new(io::Screen::new()));
    let mut image = ImageCapture::new(output.clone());
    let audio = io::nop::DummyAudio{};
    let mut nes = NES::new(event_bus.clone(), output.clone(), audio, rom);

    // Check the menu load.
    run_for(&mut nes, 2_000_000);
    assert_image(&mut image, test_resource_path("nestest/capture_01_menu.bmp"));

    // Start tests.
    event_bus.borrow_mut().broadcast(Event::KeyDown(Key::A));

    // Half way through the tests, save and load state.
    run_for(&mut nes, 4_000_000);
    let state = nes.freeze();

    let path_2 = test_resource_path("nestest/nestest.nes");
    let rom_2 = ines::ROM::load(&path_2.into_os_string().into_string().unwrap());
    let event_bus_2 = Rc::new(RefCell::new(EventBus::new()));
    let output_2 = Rc::new(RefCell::new(io::Screen::new()));
    let mut image_2 = ImageCapture::new(output_2.clone());
    let audio_2 = io::nop::DummyAudio{};
    let mut nes_2 = NES::new(event_bus_2.clone(), output_2.clone(), audio_2, rom_2);
    nes_2.hydrate(state);
   
    // Wait for tests to finish and check they pass.
    run_for(&mut nes_2, 3_000_000);
    assert_image(&mut image_2, test_resource_path("nestest/capture_02_passed.bmp"));
}

