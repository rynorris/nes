use simul::cpu;
use simul::memory;

const PROGRAM_ROOT: u16 = 0xF000;

#[test]
fn test_lda_immediate() {
    let mut cpu = new_cpu();
    run_program(&mut cpu, &[0xA9, 0x12]);
    assert_eq!(cpu.a, 0x12);
}

#[test]
fn test_lda_zero_page() {
    let mut cpu = new_cpu();
    load_data(&mut cpu.memory, 0x0034, &[0x97]);
    run_program(&mut cpu, &[0xA9, 0x12]);
    assert_eq!(cpu.a, 0x12);
}

#[test]
fn test_sta_zero_page() {
    let mut cpu = new_cpu();
    cpu.a = 0x34;
    run_program(&mut cpu, &[0x85, 0x67]);
    assert_eq!(cpu.a, 0x34);
    assert_eq!(cpu.memory.load(0x0067), 0x34);
}

fn new_cpu() -> cpu::CPU {
    cpu::new(memory::new())
}

fn load_data(memory: &mut memory::RAM, addr: u16, bytes: &[u8]) {
    for (ix, byte) in bytes.iter().enumerate() {
        memory.store(addr + (ix as u16), *byte);
    }
}

fn load_program(cpu: &mut cpu::CPU, program: &[u8]) {
    load_data(&mut cpu.memory, PROGRAM_ROOT, program);
    cpu.pc = PROGRAM_ROOT;
}

fn run_program(cpu: &mut cpu::CPU, program: &[u8]) {
    let program_size = program.len() as u16;
    load_program(cpu, program);
    cpu.pc = PROGRAM_ROOT;

    for _ in 1..1000 {
        if cpu.pc >= PROGRAM_ROOT + program_size {
            return;
        }
        cpu.execute_next_instruction();
    }

    panic!("Program didn't terminate after 1000 ticks");
}
