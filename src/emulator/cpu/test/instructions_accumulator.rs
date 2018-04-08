use emulator::cpu;

use emulator::cpu::test::load_data;
use emulator::cpu::test::new_cpu;
use emulator::cpu::test::run_program;

#[test]
fn test_lda_sets_zero_flag() {
    let mut cpu = new_cpu();
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), false);
    run_program(&mut cpu, &[0xA9, 0x00]);
    assert_eq!(cpu.a, 0x00);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), true);
}

#[test]
fn test_lda_sets_negative_flag() {
    let mut cpu = new_cpu();
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), false);
    run_program(&mut cpu, &[0xA9, 0x90]);
    assert_eq!(cpu.a, 0x90);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
}

#[test]
fn test_lda_immediate() {
    let mut cpu = new_cpu();
    let cycles = run_program(&mut cpu, &[0xA9, 0x12]);
    assert_eq!(cpu.a, 0x12);
    assert_eq!(cycles, 2);
}

#[test]
fn test_lda_zero_page() {
    let mut cpu = new_cpu();
    load_data(&mut cpu.memory, 0x0034, &[0x97]);
    let cycles = run_program(&mut cpu, &[0xA5, 0x34]);
    assert_eq!(cpu.a, 0x97);
    assert_eq!(cycles, 3);
}

#[test]
fn test_lda_zero_page_indexed() {
    let mut cpu = new_cpu();
    cpu.x = 0x12;
    load_data(&mut cpu.memory, 0x0034, &[0x97]);
    let cycles = run_program(&mut cpu, &[0xB5, 0x22]);
    assert_eq!(cpu.a, 0x97);
    assert_eq!(cycles, 4);
}

#[test]
fn test_lda_absolute() {
    let mut cpu = new_cpu();
    load_data(&mut cpu.memory, 0x5678, &[0x97]);
    let cycles = run_program(&mut cpu, &[0xAD, 0x78, 0x56]);
    assert_eq!(cpu.a, 0x97);
    assert_eq!(cycles, 4);
}

#[test]
fn test_lda_absolute_x() {
    let mut cpu = new_cpu();
    cpu.x = 0x34;
    load_data(&mut cpu.memory, 0x5678, &[0x97]);
    let cycles = run_program(&mut cpu, &[0xBD, 0x44, 0x56]);
    assert_eq!(cpu.a, 0x97);
    assert_eq!(cycles, 4);
}

#[test]
fn test_lda_absolute_y() {
    let mut cpu = new_cpu();
    cpu.y = 0x02;
    load_data(&mut cpu.memory, 0x5701, &[0x97]);
    let cycles = run_program(&mut cpu, &[0xB9, 0xFF, 0x56]);
    assert_eq!(cpu.a, 0x97);
    // Crosses page boundary.
    assert_eq!(cycles, 5);
}

#[test]
fn test_lda_indexed_indirect() {
    let mut cpu = new_cpu();
    cpu.x = 0x12;
    load_data(&mut cpu.memory, 0x0046, &[0xEF, 0xBE]);
    load_data(&mut cpu.memory, 0xBEEF, &[0x97]);
    let cycles = run_program(&mut cpu, &[0xA1, 0x34]);
    assert_eq!(cpu.a, 0x97);
    assert_eq!(cycles, 6);
}

#[test]
fn test_lda_indirect_indexed() {
    let mut cpu = new_cpu();
    cpu.y = 0x12;
    load_data(&mut cpu.memory, 0x0034, &[0xDD, 0xBE]);
    load_data(&mut cpu.memory, 0xBEEF, &[0x97]);
    let cycles = run_program(&mut cpu, &[0xB1, 0x34]);
    assert_eq!(cpu.a, 0x97);
    assert_eq!(cycles, 5);
}

#[test]
fn test_sta_zero_page() {
    let mut cpu = new_cpu();
    cpu.a = 0x34;
    let cycles = run_program(&mut cpu, &[0x85, 0x67]);
    assert_eq!(cpu.a, 0x34);
    assert_eq!(cpu.memory.load(0x0067), 0x34);
    assert_eq!(cycles, 3);
}

#[test]
fn test_sta_zero_page_indexed() {
    let mut cpu = new_cpu();
    cpu.a = 0x34;
    cpu.x = 0x08;
    let cycles = run_program(&mut cpu, &[0x95, 0x67]);
    assert_eq!(cpu.a, 0x34);
    assert_eq!(cpu.memory.load(0x006F), 0x34);
    assert_eq!(cycles, 4);
}

#[test]
fn test_sta_absolute() {
    let mut cpu = new_cpu();
    cpu.a = 0x34;
    let cycles = run_program(&mut cpu, &[0x8D, 0x67, 0x45]);
    assert_eq!(cpu.a, 0x34);
    assert_eq!(cpu.memory.load(0x4567), 0x34);
    assert_eq!(cycles, 4);
}

#[test]
fn test_sta_absolute_x() {
    let mut cpu = new_cpu();
    cpu.a = 0x97;
    cpu.x = 0x34;
    let cycles = run_program(&mut cpu, &[0x9D, 0x44, 0x56]);
    assert_eq!(cpu.a, 0x97);
    assert_eq!(cpu.memory.load(0x5678), 0x97);
    assert_eq!(cycles, 5);
}

#[test]
fn test_sta_absolute_y() {
    let mut cpu = new_cpu();
    cpu.a = 0x97;
    cpu.y = 0x34;
    let cycles = run_program(&mut cpu, &[0x99, 0x44, 0x56]);
    assert_eq!(cpu.a, 0x97);
    assert_eq!(cpu.memory.load(0x5678), 0x97);
    assert_eq!(cycles, 5);
}

#[test]
fn test_sta_indexed_indirect() {
    let mut cpu = new_cpu();
    cpu.a = 0x97;
    cpu.x = 0x12;
    load_data(&mut cpu.memory, 0x0046, &[0xEF, 0xBE]);
    let cycles = run_program(&mut cpu, &[0x81, 0x34]);
    assert_eq!(cpu.a, 0x97);
    assert_eq!(cpu.memory.load(0xBEEF), 0x97);
    assert_eq!(cycles, 6);
}

#[test]
fn test_sta_indirect_indexed() {
    let mut cpu = new_cpu();
    cpu.a = 0x97;
    cpu.y = 0x12;
    load_data(&mut cpu.memory, 0x0034, &[0xDD, 0xBE]);
    let cycles = run_program(&mut cpu, &[0x91, 0x34]);
    assert_eq!(cpu.a, 0x97);
    assert_eq!(cpu.memory.load(0xBEEF), 0x97);
    assert_eq!(cycles, 6);
}

#[test]
fn test_nop_does_nothing_and_takes_2_cycles() {
    let mut cpu = new_cpu();
    let cycles = run_program(&mut cpu, &[0xEA]);
    assert_eq!(cycles, 2);
}

