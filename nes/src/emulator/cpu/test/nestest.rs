use std::fs::File;
use std::io::{BufRead, BufReader, Read};

use crate::emulator::clock::Ticker;
use crate::emulator::cpu;

use crate::emulator::cpu::test::new_cpu;
use crate::emulator::test::test_resource_path;

#[test]
fn test_nestest() {
    println!("Hello, world!");
    let mut cpu = new_cpu();
    cpu.disable_bcd();

    load_rom(&mut cpu);

    let mut trace_lines = load_trace();

    cpu.startup_sequence();

    // TODO(rnorris): Figure out if these starting conditions are universal.
    cpu.p.load_byte(0x24);
    cpu.sp = 0xFD;

    let mut cycles: u64 = 0;

    // At instruction 5004 it starts testing undocumented opcodes which we don't support.
    for _ in 0..5003 {
        // We know there are more than 5000 lines.
        let line = trace_lines.next().unwrap();
        assert_state(&mut cpu, cycles, line);

        let new_cycles = cpu.tick();
        cycles += new_cycles as u64;
    }
}

fn assert_state(cpu: &mut cpu::CPU, cycles: u64, line: String) {
    // Check PC.
    assert_eq!(cpu.pc, cpu::trace::parse_pc(&line));

    // Check next instruction.
    let (opcode, b1, b2) = cpu.peek_next_instruction();
    assert_eq!(opcode, cpu::trace::parse_opcode(&line));
    match b1 {
        Some(b) => assert_eq!(b, cpu::trace::parse_instruction_byte_1(&line)),
        None => (),
    }

    match b2 {
        Some(b) => assert_eq!(b, cpu::trace::parse_instruction_byte_2(&line)),
        None => (),
    }

    // Check registers.
    assert_eq!(cpu.a, cpu::trace::parse_a(&line));
    assert_eq!(cpu.x, cpu::trace::parse_x(&line));
    assert_eq!(cpu.y, cpu::trace::parse_y(&line));
    assert_eq!(cpu.p.as_byte(), cpu::trace::parse_p(&line));
    assert_eq!(cpu.sp, cpu::trace::parse_sp(&line));

    // Check CYC.
    // Note that this is actually the PPU x-coordinate, not CPU cycles.
    // So we have to convert.
    let ppu_x = (cycles * 3) % 341;
    assert_eq!(ppu_x, cpu::trace::parse_cyc(&line));
}

fn load_rom(cpu: &mut cpu::CPU) {
    let path = test_resource_path("nestest/nestest.nes");
    let mut file = match File::open(&path) {
        Err(cause) => panic!("Couldn't open {}: {}", path.display(), cause),
        Ok(file) => file,
    };

    let mut contents = vec![];
    match file.read_to_end(&mut contents) {
        Err(cause) => panic!("Failed to read file: {}", cause),
        Ok(_) => (),
    };

    // Hack for now.  Later should properly map memory.
    let mut program = [0; 65536];
    for ix in 0..0x4000 {
        program[0x8000 + ix] = contents[0x0010 + ix];
        program[0xC000 + ix] = contents[0x0010 + ix];

        // Override the startup vector so we start from the right place.
        program[0xFFFC] = 0x00;
        program[0xFFFD] = 0xC0;
    }
    cpu.load_program(&program);
}

fn load_trace() -> impl Iterator<Item = String> {
    let path = test_resource_path("nestest/nestest.trace");
    let file = match File::open(&path) {
        Err(cause) => panic!("Couldn't open {}: {}", path.display(), cause),
        Ok(file) => file,
    };

    let reader = BufReader::new(file);
    reader.lines().map(|l| l.unwrap())
}
