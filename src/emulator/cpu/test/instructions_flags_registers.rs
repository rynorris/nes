use emulator::cpu;

use emulator::cpu::test::new_cpu;
use emulator::cpu::test::run_program;

#[test]
fn test_sec() {
    let mut cpu = new_cpu();
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    let cycles = run_program(&mut cpu, &[0x38]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_clc() {
    let mut cpu = new_cpu();
    cpu.p.set(cpu::flags::Flag::C);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    let cycles = run_program(&mut cpu, &[0x18]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cycles, 2);
}

#[test]
fn test_sei() {
    let mut cpu = new_cpu();
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::I), false);
    let cycles = run_program(&mut cpu, &[0x78]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::I), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_cli() {
    let mut cpu = new_cpu();
    cpu.p.set(cpu::flags::Flag::I);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::I), true);
    let cycles = run_program(&mut cpu, &[0x58]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::I), false);
    assert_eq!(cycles, 2);
}

#[test]
fn test_sed() {
    let mut cpu = new_cpu();
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::D), false);
    let cycles = run_program(&mut cpu, &[0xF8]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::D), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_cld() {
    let mut cpu = new_cpu();
    cpu.p.set(cpu::flags::Flag::D);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::D), true);
    let cycles = run_program(&mut cpu, &[0xD8]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::D), false);
    assert_eq!(cycles, 2);
}

#[test]
fn test_clv() {
    let mut cpu = new_cpu();
    cpu.p.set(cpu::flags::Flag::V);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), true);
    let cycles = run_program(&mut cpu, &[0xB8]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 2);
}
