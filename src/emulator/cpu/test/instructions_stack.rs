use emulator::cpu;
use emulator::util;

use emulator::cpu::test::load_program;
use emulator::cpu::test::new_cpu;
use emulator::cpu::test::run_program;
use emulator::cpu::test::run_instructions;

#[test]
fn test_jsr() {
    let mut cpu = new_cpu();
    load_program(&mut cpu, &[0x20, 0xEF, 0xBE]);
    let pc_before = cpu.pc;
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, 0xBEEF);

    // Test that the PC was pushed to the stack.
    let pc_low = cpu.stack_pop();
    let pc_high = cpu.stack_pop();
    assert_eq!(util::combine_bytes(pc_high, pc_low), pc_before + 2);

    assert_eq!(cycles, 6);
}

#[test]
fn test_rts() {
    let mut cpu = new_cpu();
    cpu.stack_push(0xBE);
    cpu.stack_push(0xEE);
    load_program(&mut cpu, &[0x60]);
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, 0xBEEF);
    assert_eq!(cycles, 6);
}

#[test]
fn test_pha() {
    let mut cpu = new_cpu();
    cpu.a = 0x34;
    let cycles = run_program(&mut cpu, &[0x48]);
    assert_eq!(cpu.a, 0x34);
    assert_eq!(cpu.stack_pop(), 0x34);
    assert_eq!(cycles, 3);
}

#[test]
fn test_pla() {
    let mut cpu = new_cpu();
    cpu.stack_push(0x34);
    let cycles = run_program(&mut cpu, &[0x68]);
    assert_eq!(cpu.a, 0x34);
    assert_eq!(cycles, 3);
}

#[test]
fn test_pla_sets_negative_flag() {
    let mut cpu = new_cpu();
    cpu.stack_push(0xFF);
    let cycles = run_program(&mut cpu, &[0x68]);
    assert_eq!(cpu.a, 0xFF);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cycles, 3);
}

#[test]
fn test_pla_sets_zero_flag() {
    let mut cpu = new_cpu();
    cpu.stack_push(0x00);
    let cycles = run_program(&mut cpu, &[0x68]);
    assert_eq!(cpu.a, 0x00);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), true);
    assert_eq!(cycles, 3);
}

