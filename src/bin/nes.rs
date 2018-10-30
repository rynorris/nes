extern crate mos_6500;

use std::env;

use mos_6500::emulator;
use mos_6500::emulator::ines;

fn main() {
    let args: Vec<String> = env::args().collect();

    let rom_path = match args.get(0) {
        None => panic!("You must pass in a path to a iNes ROM file."),
        Some(path) => path,
    };

    let rom = ines::ROM::load(rom_path);

    let mut nes = emulator::NES::new(rom);

    loop {
        nes.tick();
    }
}
