use emulator::cpu;

use emulator::cpu::test::load_data;
use emulator::cpu::test::new_cpu;
use emulator::cpu::test::run_program;

#[test]
fn test_and_immediate() {
    let mut cpu = new_cpu();
    cpu.a = 0b0111_1001;
    let cycles = run_program(&mut cpu, &[0x29, 0b0000_1111]);
    assert_eq!(cpu.a, 0b0000_1001);
    assert_eq!(cycles, 2);
}

#[test]
fn test_and_immediate_sets_zero_flag() {
    let mut cpu = new_cpu();
    cpu.a = 0b0111_1001;
    let cycles = run_program(&mut cpu, &[0x29, 0b0000_0110]);
    assert_eq!(cpu.a, 0b0000_0000);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_and_immediate_sets_negative_flag() {
    let mut cpu = new_cpu();
    cpu.a = 0b1111_1001;
    let cycles = run_program(&mut cpu, &[0x29, 0b1000_0110]);
    assert_eq!(cpu.a, 0b1000_0000);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_and_zero_page() {
    let mut cpu = new_cpu();
    cpu.a = 0b0111_1001;
    load_data(&mut cpu.memory, 0x0024, &[0b1111_0000]);
    let cycles = run_program(&mut cpu, &[0x25, 0x24]);
    assert_eq!(cpu.a, 0b0111_0000);
    assert_eq!(cycles, 3);
}

#[test]
fn test_and_zero_page_indexed() {
    let mut cpu = new_cpu();
    cpu.a = 0b0111_1001;
    cpu.x = 0x10;
    load_data(&mut cpu.memory, 0x0034, &[0b1111_0000]);
    let cycles = run_program(&mut cpu, &[0x35, 0x24]);
    assert_eq!(cpu.a, 0b0111_0000);
    assert_eq!(cycles, 4);
}

#[test]
fn test_and_absolute() {
    let mut cpu = new_cpu();
    cpu.a = 0b0111_1001;
    load_data(&mut cpu.memory, 0xBEEF, &[0b1111_0000]);
    let cycles = run_program(&mut cpu, &[0x2D, 0xEF, 0xBE]);
    assert_eq!(cpu.a, 0b0111_0000);
    assert_eq!(cycles, 4);
}

#[test]
fn test_and_absolute_x() {
    let mut cpu = new_cpu();
    cpu.a = 0b0111_1001;
    cpu.x = 0x10;
    load_data(&mut cpu.memory, 0xBEEF, &[0b1111_0000]);
    let cycles = run_program(&mut cpu, &[0x3D, 0xDF, 0xBE]);
    assert_eq!(cpu.a, 0b0111_0000);
    assert_eq!(cycles, 4);
}

#[test]
fn test_and_absolute_y() {
    let mut cpu = new_cpu();
    cpu.a = 0b0111_1001;
    cpu.y = 0x10;
    load_data(&mut cpu.memory, 0xBEEF, &[0b1111_0000]);
    let cycles = run_program(&mut cpu, &[0x39, 0xDF, 0xBE]);
    assert_eq!(cpu.a, 0b0111_0000);
    assert_eq!(cycles, 4);
}

#[test]
fn test_and_indexed_indirect() {
    let mut cpu = new_cpu();
    cpu.a = 0b0111_1001;
    cpu.x = 0x10;
    load_data(&mut cpu.memory, 0x0046, &[0xEF, 0xBE]);
    load_data(&mut cpu.memory, 0xBEEF, &[0b0000_1111]);
    let cycles = run_program(&mut cpu, &[0x21, 0x36]);
    assert_eq!(cpu.a, 0b0000_1001);
    assert_eq!(cycles, 6);
}

#[test]
fn test_and_indirect_indexed() {
    let mut cpu = new_cpu();
    cpu.a = 0b0111_1001;
    cpu.y = 0x10;
    load_data(&mut cpu.memory, 0x0046, &[0xDF, 0xBE]);
    load_data(&mut cpu.memory, 0xBEEF, &[0b0000_1111]);
    let cycles = run_program(&mut cpu, &[0x31, 0x46]);
    assert_eq!(cpu.a, 0b0000_1001);
    assert_eq!(cycles, 5);
}

