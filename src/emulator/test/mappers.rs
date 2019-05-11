// -- Mapper tests using Holy Diver Batman test roms.
// -- Test each mapper once normally, and then once with a savestate.

macro_rules! test_mapper {
    ($name:ident, $rom:expr, $cycles:expr) => {
        mod $name {
            use crate::emulator::state::SaveState;

            use crate::emulator::test::assert_image;
            use crate::emulator::test::prepare_ete_test;
            use crate::emulator::test::run_for;
            use crate::emulator::test::test_resource_path;

            #[test]
            fn test() {
                let path = test_resource_path(&format!("mappers/{}.nes", $rom));
                let (mut nes, _, image) = prepare_ete_test(&path);
                run_for(&mut nes, $cycles);
                assert_image(&image, test_resource_path(&format!("mappers/{}.bmp", $rom)));
            }

            #[test]
            fn test_savestate() {
                let path = test_resource_path(&format!("mappers/{}.nes", $rom));
                let (mut nes, _, _) = prepare_ete_test(&path);
                run_for(&mut nes, $cycles / 2);
                let state = nes.freeze();

                let (mut nes_2, _, image_2) = prepare_ete_test(&path);
                nes_2.hydrate(state);
                run_for(&mut nes_2, $cycles / 2);
                assert_image(&image_2, test_resource_path(&format!("mappers/{}.bmp", $rom)));
            }
        }
    };
}

test_mapper!(nrom, "M0_P32K_C8K_V", 100_000_000);

// Note that test status here is 1000.
// This means that bit 4 of $Exxx doesn't disable WRAM.
// Right now our mappers aren't hooked up to WRAM, so accepting this as-is.
test_mapper!(mmc1, "M1_P128K_C128K", 500_000_000);
test_mapper!(uxrom, "M2_P128K_V", 150_000_000);
test_mapper!(cnrom, "M3_P32K_C32K_H", 100_000_000);
test_mapper!(mmc3, "M4_P256K_C256K", 200_000_000);
test_mapper!(axrom, "M7_P128K", 120_000_000);
