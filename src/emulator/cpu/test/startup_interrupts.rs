use emulator::cpu;
use emulator::cpu::opcodes;

use emulator::cpu::test::load_data;
use emulator::cpu::test::new_cpu;

#[test]
fn test_startup_sequence() {
    let mut cpu = new_cpu();
    load_data(&mut cpu.memory, 0xFFFC, &[0xAD, 0xDE]);
    cpu.startup_sequence();
    assert_eq!(cpu.pc, 0xDEAD);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::I), true);
}

#[test]
fn test_interrupt_during_simple_program() {
    let mut cpu = new_cpu();

    // Simple program just loads a byte, adds 0x56, and stores it elsewhere.
    // Interrupt routine loads a byte into X and returns.
    // Program lives at address 0x8000.
    // Interrupt routine lives at address 0x6000.
    load_data(&mut cpu.memory, 0x0099, &[0x34]);
    let program = [
        opcodes::LDA_ZPG, 0x99,  // LDA 0x34
        opcodes::ADC_IMM, 0x56,  // ADC # 0x56
        opcodes::STA_ABS, 0xEF, 0xBE,  // STA 0xBEEF
    ];

    let interrupt_routine = [
        opcodes::LDX_IMM, 0x24,
        opcodes::RTI,
    ];

    // Load programs.
    load_data(&mut cpu.memory, 0x8000, &program);
    load_data(&mut cpu.memory, 0x6000, &interrupt_routine);

    // Set up vectors.
    load_data(&mut cpu.memory, cpu::IRQ_VECTOR, &[0x00, 0x60]);

    // Skip startup sequence for this test.  Only want to test interrupt.
    cpu.pc = 0x8000;

    // Run half of the program.
    cpu.execute_next_instruction();
    cpu.execute_next_instruction();

    // Interrupt.
    cpu.interrupt();

    // Run interrupt routine.
    cpu.execute_next_instruction();
    cpu.execute_next_instruction();

    // Finish program.
    cpu.execute_next_instruction();

    // Check program finished successfully.
    assert_eq!(cpu.load_memory(0xBEEF), 0x8A);

    // Check interrupt routine ran.
    assert_eq!(cpu.x, 0x24);
}

