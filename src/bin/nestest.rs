extern crate mos_6500;

use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use mos_6500::emulator::cpu;
use mos_6500::emulator::memory;


fn main() {
    println!("Hello, world!");
    let mut cpu = cpu::new(memory::new());
    cpu.disable_bcd();

    load_rom(&mut cpu, String::from("./nestest.nes"));

    let trace_file = match File::create("cpu.trace") {
        Err(cause) => panic!("Couldn't create cpu.trace: {}", cause),
        Ok(file) => file,
    };

    cpu.startup_sequence();
    let mut cycles: u64 = 0;
    for ix in 0..8000 {
        cpu.trace_next_instruction(&trace_file);
        let new_cycles = cpu.tick();
        
        // Append the cycles to the trace since the CPU doesn't track these itself.
        // Needs to be the cycles BEFORE the instruction, because that's just how nestest traces.
        write!(&trace_file, " CYC:{:>3}", (cycles * 3) % 341);
        write!(&trace_file, "\n");

        cycles += new_cycles as u64;
    }
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
}
