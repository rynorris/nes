use simul::cpu;

use simul::cpu::test::load_data;
use simul::cpu::test::new_cpu;
use simul::cpu::test::run_program;

#[test]
fn test_adc_immediate_no_carry() {
    let mut cpu = new_cpu();
    cpu.a = 0x23;
    let cycles = run_program(&mut cpu, &[0x69, 0x12]);
    assert_eq!(cpu.a, 0x35);
    assert_eq!(cycles, 2);
}

#[test]
fn test_adc_immediate_with_carry() {
    let mut cpu = new_cpu();
    cpu.a = 0xF8;
    run_program(&mut cpu, &[0x69, 0x12]);
    assert_eq!(cpu.a, 0x0A);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
}

#[test]
fn test_adc_immediate_with_overflow() {
    let mut cpu = new_cpu();
    cpu.a = 0x08;
    run_program(&mut cpu, &[0x69, 0x82]);
    assert_eq!(cpu.a, 0x8A);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), true);
}

#[test]
fn test_adc_immediate_with_negative() {
    let mut cpu = new_cpu();
    cpu.a = 0x08;
    run_program(&mut cpu, &[0x69, 0x82]);
    assert_eq!(cpu.a, 0x8A);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
}

#[test]
fn test_adc_zero_page() {
    let mut cpu = new_cpu();
    cpu.a = 0x23;
    load_data(&mut cpu.memory, 0x0024, &[0x12]);
    run_program(&mut cpu, &[0x65, 0x24]);
    assert_eq!(cpu.a, 0x35);
}

#[test]
fn test_adc_zero_page_indexed() {
    let mut cpu = new_cpu();
    cpu.a = 0x23;
    cpu.x = 0x10;
    load_data(&mut cpu.memory, 0x0034, &[0x12]);
    run_program(&mut cpu, &[0x75, 0x24]);
    assert_eq!(cpu.a, 0x35);
}

#[test]
fn test_adc_absolute() {
    let mut cpu = new_cpu();
    cpu.a = 0x23;
    load_data(&mut cpu.memory, 0xBEEF, &[0x12]);
    run_program(&mut cpu, &[0x6D, 0xEF, 0xBE]);
    assert_eq!(cpu.a, 0x35);
}

#[test]
fn test_adc_absolute_x() {
    let mut cpu = new_cpu();
    cpu.a = 0x23;
    cpu.x = 0x10;
    load_data(&mut cpu.memory, 0xBEEF, &[0x12]);
    run_program(&mut cpu, &[0x7D, 0xDF, 0xBE]);
    assert_eq!(cpu.a, 0x35);
}

#[test]
fn test_adc_absolute_y() {
    let mut cpu = new_cpu();
    cpu.a = 0x23;
    cpu.y = 0x10;
    load_data(&mut cpu.memory, 0xBEEF, &[0x12]);
    run_program(&mut cpu, &[0x79, 0xDF, 0xBE]);
    assert_eq!(cpu.a, 0x35);
}

#[test]
fn test_adc_indexed_indirect() {
    let mut cpu = new_cpu();
    cpu.a = 0x23;
    cpu.x = 0x10;
    load_data(&mut cpu.memory, 0x0046, &[0xEF, 0xBE]);
    load_data(&mut cpu.memory, 0xBEEF, &[0x12]);
    run_program(&mut cpu, &[0x61, 0x36]);
    assert_eq!(cpu.a, 0x35);
}

#[test]
fn test_adc_indirect_indexed() {
    let mut cpu = new_cpu();
    cpu.a = 0x23;
    cpu.y = 0x10;
    load_data(&mut cpu.memory, 0x0046, &[0xDF, 0xBE]);
    load_data(&mut cpu.memory, 0xBEEF, &[0x12]);
    run_program(&mut cpu, &[0x71, 0x46]);
    assert_eq!(cpu.a, 0x35);
}

