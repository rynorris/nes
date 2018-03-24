use emulator::cpu::test::load_data;
use emulator::cpu::test::load_program;
use emulator::cpu::test::new_cpu;
use emulator::cpu::test::run_instructions;

#[test]
fn test_jmp_absolute() {
    let mut cpu = new_cpu();
    load_program(&mut cpu, &[0x4C, 0xEF, 0xBE]);
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, 0xBEEF);
    assert_eq!(cycles, 3);
}

#[test]
fn test_jmp_indirect() {
    let mut cpu = new_cpu();
    load_data(&mut cpu.memory, 0xDEAD, &[0xEF, 0xBE]);
    load_program(&mut cpu, &[0x6C, 0xAD, 0xDE]);
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, 0xBEEF);
    assert_eq!(cycles, 5);
}

