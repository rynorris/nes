extern crate mos_6500;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use mos_6500::emulator::cpu;
use mos_6500::emulator::memory;
use mos_6500::emulator::memory::Writer;


fn main() {
    println!("Hello, world!");
    let mut cpu = cpu::new(memory::new());

    load_rom(&mut cpu, String::from("./data/nestest.NES"));

}

fn load_rom(cpu: &mut cpu::CPU, path_string: String) {
    let path = Path::new(&path_string);
    let mut file = match File::open(&path) {
        Err(cause) => panic!("Couldn't open {}: {}", path.display(), cause),
        Ok(file) => file,
    };

    let mut contents = vec![];
    match file.read_to_end(&mut contents) {
        Err(cause) => panic!("Failed to read file: {}", cause),
        Ok(_) => ()
    };

    cpu.load_program(contents.as_slice());
}
