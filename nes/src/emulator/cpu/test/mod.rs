mod instructions_accumulator;
mod instructions_arithmetic;
mod instructions_branch;
mod instructions_flags_registers;
mod instructions_index_registers;
mod instructions_logical;
mod instructions_reset_interrupt;
mod instructions_shift_modify;
mod instructions_stack;
mod nestest;
mod programs;
mod startup_interrupts;

use crate::emulator::cpu;
use crate::emulator::memory;
use crate::emulator::memory::ReadWriter;

pub const PROGRAM_ROOT: u16 = 0xF000;
fn new_cpu() -> cpu::CPU {
    cpu::new(Box::new(memory::Memory::new_ram(0x10000)))
}

fn load_data(memory: &mut Box<dyn ReadWriter>, addr: u16, bytes: &[u8]) {
    for (ix, byte) in bytes.iter().enumerate() {
        memory.write(addr + (ix as u16), *byte);
    }
}

fn load_program(cpu: &mut cpu::CPU, program: &[u8]) {
    load_data(&mut cpu.memory, PROGRAM_ROOT, program);
    cpu.pc = PROGRAM_ROOT;
}

// Returns total number of elapsed cycles.
fn run_program(cpu: &mut cpu::CPU, program: &[u8]) -> u32 {
    let program_size = program.len() as u16;
    load_program(cpu, program);
    cpu.pc = PROGRAM_ROOT;

    let mut cycles = 0;
    for _ in 1..1000 {
        if cpu.pc >= PROGRAM_ROOT + program_size {
            return cycles;
        }
        cycles += cpu.execute_next_instruction();
    }

    panic!("Program didn't terminate after 1000 ticks");
}

fn run_instructions(cpu: &mut cpu::CPU, num_instructions: u32) -> u32 {
    let mut cycles = 0;
    for _ in 0..num_instructions {
        cycles += cpu.execute_next_instruction();
    }
    cycles
}
