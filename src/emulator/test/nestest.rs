use crate::emulator::io::event::{Event, Key};
use crate::emulator::state::SaveState;

use crate::emulator::test::assert_image;
use crate::emulator::test::prepare_ete_test;
use crate::emulator::test::run_for;
use crate::emulator::test::test_resource_path;

// -- Visual nestest.
#[test]
fn test_nestest_visual() {
    let path = test_resource_path("nestest/nestest.nes");
    let (mut nes, event_bus, image) = prepare_ete_test(&path);

    // Check the menu load.
    run_for(&mut nes, 2_000_000);
    assert_image(&image, test_resource_path("nestest/capture_01_menu.bmp"));

    // Start tests.
    event_bus.borrow_mut().broadcast(Event::KeyDown(Key::A));
   
    // Wait for tests to finish and check they pass.
    run_for(&mut nes, 7_000_000);
    assert_image(&image, test_resource_path("nestest/capture_02_passed.bmp"));
}

#[test]
fn test_nestest_savestate() {
    let path = test_resource_path("nestest/nestest.nes");
    let (mut nes, event_bus, image) = prepare_ete_test(&path);

    // Check the menu load.
    run_for(&mut nes, 2_000_000);
    assert_image(&image, test_resource_path("nestest/capture_01_menu.bmp"));

    // Start tests.
    event_bus.borrow_mut().broadcast(Event::KeyDown(Key::A));

    // Half way through the tests, save and load state.
    run_for(&mut nes, 4_000_000);
    let state = nes.freeze();

    let (mut nes_2, _, image_2) = prepare_ete_test(&path);
    nes_2.hydrate(state);
   
    // Wait for tests to finish and check they pass.
    run_for(&mut nes_2, 3_000_000);
    assert_image(&image_2, test_resource_path("nestest/capture_02_passed.bmp"));
}

