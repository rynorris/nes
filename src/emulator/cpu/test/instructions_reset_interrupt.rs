use emulator::cpu::test::load_data;
use emulator::cpu::test::load_program;
use emulator::cpu::test::new_cpu;
use emulator::cpu::test::run_instructions;

#[test]
fn test_rti() {
    let mut cpu = new_cpu();
    // Push PCH, PCL, P.
    cpu.stack_push(0xBE);
    cpu.stack_push(0xEF);
    cpu.stack_push(0b0011_1010);
    load_program(&mut cpu, &[0x40]);
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, 0xBEEF);

    // Bits 3 and 4 should be ignored when loading status register from stack.
    assert_eq!(cpu.p.as_byte(), 0b0000_1010);
    assert_eq!(cycles, 6);
}

#[test]
fn test_brk() {
    let mut cpu = new_cpu();
    // Set interrupt vector.
    load_data(&mut cpu.memory, 0xFFFE, &[0xEF, 0xBE]);
    load_program(&mut cpu, &[0x00]);
    let pc_before = cpu.pc;
    let stored_pc = pc_before + 2;
    let p_before = cpu.p.as_byte();
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, 0xBEEF); // Jumped to interrup vector.
    assert_eq!(cpu.stack_pop(), p_before | 0b0001_0000); // P was stored, and break bit was set.
    assert_eq!(cpu.stack_pop(), stored_pc as u8); // PCL was stored.
    assert_eq!(cpu.stack_pop(), (stored_pc >> 8) as u8); // PCH was stored.
    assert_eq!(cycles, 7);
}

