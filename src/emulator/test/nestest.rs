use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use emulator::ines;
use emulator::io;
use emulator::io::event::{Event, EventBus, Key};
use emulator::io::sdl::ImageCapture;
use emulator::NES;

use emulator::test::file_digest;
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

    let tmp_dir = env::temp_dir();

    // Wait for main menu to load.
    for _ in 1 .. 550_000 {
        nes.tick();
    }

    // Check the menu loaded properly.
    let mut bmp_01_path = tmp_dir.clone();
    bmp_01_path.push("01.bmp");
    image.save_bmp(&bmp_01_path);
    println!("Saving image to tempfile at: {}", bmp_01_path.display());
    assert_eq!(file_digest(bmp_01_path), file_digest(test_resource_path("nestest/capture_01_menu.bmp")));

    // Start tests.
    event_bus.borrow_mut().broadcast(Event::KeyDown(Key::A));
   
    // Wait for tests to finish.
    for _ in 1 .. 2_000_000 {
        nes.tick();
    }

    // Check the tests passed.
    let mut bmp_02_path = tmp_dir.clone();
    bmp_02_path.push("02.bmp");
    println!("Saving image to tempfile at: {}", bmp_02_path.display());
    image.save_bmp(&bmp_02_path);
    assert_eq!(file_digest(bmp_02_path), file_digest(test_resource_path("nestest/capture_02_passed.bmp")));
}

