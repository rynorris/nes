extern crate mos_6500;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use mos_6500::emulator::cpu;
use mos_6500::emulator::memory;


fn main() {
    println!("Hello, world!");
    let mut cpu = cpu::new(memory::new());

    load_rom(&mut cpu, String::from("./nestest.nes"));

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

    // Hack for now.  Later should properly map memory.
    let mut program = [0; 65536];
    for ix in 0..0x4000 {
        program[0x8000 + ix] = contents[0x0010 + ix];
        program[0xC000 + ix] = contents[0x0010 + ix];
        program[0xFFFC] = 0x00;
        program[0xFFFD] = 0xC0;
    }
    cpu.load_program(&program);

    let trace_file = match File::create("cpu.trace") {
        Err(cause) => panic!("Couldn't create {}: {}", path.display(), cause),
        Ok(file) => file,
    };

    cpu.startup_sequence();
    for ix in 0..100 {
        cpu.trace_next_instruction(&trace_file);
        cpu.tick();
    }
}
