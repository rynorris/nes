use emulator::cpu;

use emulator::cpu::test::load_data;
use emulator::cpu::test::new_cpu;
use emulator::cpu::test::run_program;

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
fn test_adc_immediate_with_positive_overflow() {
    let mut cpu = new_cpu();
    cpu.a = 0b0111_1111;
    run_program(&mut cpu, &[0x69, 0b0000_0001]);
    assert_eq!(cpu.a, 0b1000_0000);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), true);
}

#[test]
fn test_adc_immediate_with_negative_overflow() {
    let mut cpu = new_cpu();
    cpu.a = 0b1000_0000;
    run_program(&mut cpu, &[0x69, 0b1111_1111]);
    assert_eq!(cpu.a, 0b0111_1111);
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
    let cycles = run_program(&mut cpu, &[0x65, 0x24]);
    assert_eq!(cpu.a, 0x35);
    assert_eq!(cycles, 3);
}

#[test]
fn test_adc_zero_page_indexed() {
    let mut cpu = new_cpu();
    cpu.a = 0x23;
    cpu.x = 0x10;
    load_data(&mut cpu.memory, 0x0034, &[0x12]);
    let cycles = run_program(&mut cpu, &[0x75, 0x24]);
    assert_eq!(cpu.a, 0x35);
    assert_eq!(cycles, 4);
}

#[test]
fn test_adc_absolute() {
    let mut cpu = new_cpu();
    cpu.a = 0x23;
    load_data(&mut cpu.memory, 0xBEEF, &[0x12]);
    let cycles = run_program(&mut cpu, &[0x6D, 0xEF, 0xBE]);
    assert_eq!(cpu.a, 0x35);
    assert_eq!(cycles, 4);
}

#[test]
fn test_adc_absolute_x() {
    let mut cpu = new_cpu();
    cpu.a = 0x23;
    cpu.x = 0x10;
    load_data(&mut cpu.memory, 0xBEEF, &[0x12]);
    let cycles = run_program(&mut cpu, &[0x7D, 0xDF, 0xBE]);
    assert_eq!(cpu.a, 0x35);
    assert_eq!(cycles, 4);
}

#[test]
fn test_adc_absolute_y() {
    let mut cpu = new_cpu();
    cpu.a = 0x23;
    cpu.y = 0x10;
    load_data(&mut cpu.memory, 0xBEEF, &[0x12]);
    let cycles = run_program(&mut cpu, &[0x79, 0xDF, 0xBE]);
    assert_eq!(cpu.a, 0x35);
    assert_eq!(cycles, 4);
}

#[test]
fn test_adc_indexed_indirect() {
    let mut cpu = new_cpu();
    cpu.a = 0x23;
    cpu.x = 0x10;
    load_data(&mut cpu.memory, 0x0046, &[0xEF, 0xBE]);
    load_data(&mut cpu.memory, 0xBEEF, &[0x12]);
    let cycles = run_program(&mut cpu, &[0x61, 0x36]);
    assert_eq!(cpu.a, 0x35);
    assert_eq!(cycles, 6);
}

#[test]
fn test_adc_indirect_indexed() {
    let mut cpu = new_cpu();
    cpu.a = 0x23;
    cpu.y = 0x10;
    load_data(&mut cpu.memory, 0x0046, &[0xDF, 0xBE]);
    load_data(&mut cpu.memory, 0xBEEF, &[0x12]);
    let cycles = run_program(&mut cpu, &[0x71, 0x46]);
    assert_eq!(cpu.a, 0x35);
    assert_eq!(cycles, 5);
}

#[test]
fn test_adc_bcd_no_carry() {
    let mut cpu = new_cpu();
    // 79 + 12 = 91
    cpu.a = 0b0111_1001;
    cpu.p.set(cpu::flags::Flag::D);
    let cycles = run_program(&mut cpu, &[0x69, 0b0001_0010]);
    assert_eq!(cpu.a, 0b1001_0001);
    assert_eq!(cycles, 2);
}

#[test]
fn test_adc_bcd_with_carry() {
    let mut cpu = new_cpu();
    // 79 + 42 = 121
    cpu.a = 0b0111_1001;
    cpu.p.set(cpu::flags::Flag::D);
    let cycles = run_program(&mut cpu, &[0x69, 0b0100_0010]);
    assert_eq!(cpu.a, 0b0010_0001);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_sbc_immediate() {
    let mut cpu = new_cpu();
    cpu.a = 0x35;
    cpu.p.set(cpu::flags::Flag::C);
    let cycles = run_program(&mut cpu, &[0xE9, 0x23]);
    assert_eq!(cpu.a, 0x12);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_sbc_immediate_with_borrow() {
    let mut cpu = new_cpu();
    cpu.a = 0x35;
    cpu.p.clear(cpu::flags::Flag::C);
    let cycles = run_program(&mut cpu, &[0xE9, 0x23]);
    assert_eq!(cpu.a, 0x11);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_sbc_immediate_sets_zero_flag() {
    let mut cpu = new_cpu();
    cpu.a = 0x35;
    cpu.p.set(cpu::flags::Flag::C);
    let cycles = run_program(&mut cpu, &[0xE9, 0x35]);
    assert_eq!(cpu.a, 0x00);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_sbc_immediate_sets_negative_flag() {
    let mut cpu = new_cpu();
    cpu.a = 0x35;
    cpu.p.set(cpu::flags::Flag::C);
    let cycles = run_program(&mut cpu, &[0xE9, 0x36]);
    assert_eq!(cpu.a, 0xFF);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_sbc_immediate_negative_overflow() {
    let mut cpu = new_cpu();
    cpu.a = 0b1000_0000;
    cpu.p.set(cpu::flags::Flag::C);
    let cycles = run_program(&mut cpu, &[0xE9, 0x01]);
    assert_eq!(cpu.a, 0b0111_1111);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_sbc_immediate_positive_overflow() {
    let mut cpu = new_cpu();
    cpu.a = 0b0111_1111;
    cpu.p.set(cpu::flags::Flag::C);
    let cycles = run_program(&mut cpu, &[0xE9, 0b1111_1111]);
    assert_eq!(cpu.a, 0b1000_0000);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_sbc_zero_page() {
    let mut cpu = new_cpu();
    cpu.a = 0x35;
    cpu.p.set(cpu::flags::Flag::C);
    load_data(&mut cpu.memory, 0x0024, &[0x23]);
    let cycles = run_program(&mut cpu, &[0xE5, 0x24]);
    assert_eq!(cpu.a, 0x12);
    assert_eq!(cycles, 3);
}

#[test]
fn test_sbc_zero_page_indexed() {
    let mut cpu = new_cpu();
    cpu.a = 0x35;
    cpu.x = 0x11;
    cpu.p.set(cpu::flags::Flag::C);
    load_data(&mut cpu.memory, 0x0024, &[0x23]);
    let cycles = run_program(&mut cpu, &[0xF5, 0x13]);
    assert_eq!(cpu.a, 0x12);
    assert_eq!(cycles, 4);
}

#[test]
fn test_sbc_absolute() {
    let mut cpu = new_cpu();
    cpu.a = 0x35;
    cpu.p.set(cpu::flags::Flag::C);
    load_data(&mut cpu.memory, 0xBEEF, &[0x23]);
    let cycles = run_program(&mut cpu, &[0xED, 0xEF, 0xBE]);
    assert_eq!(cpu.a, 0x12);
    assert_eq!(cycles, 4);
}

#[test]
fn test_sbc_absolute_x() {
    let mut cpu = new_cpu();
    cpu.a = 0x35;
    cpu.x = 0x11;
    cpu.p.set(cpu::flags::Flag::C);
    load_data(&mut cpu.memory, 0xBEEF, &[0x23]);
    let cycles = run_program(&mut cpu, &[0xFD, 0xDE, 0xBE]);
    assert_eq!(cpu.a, 0x12);
    assert_eq!(cycles, 4);
}

#[test]
fn test_sbc_absolute_y() {
    let mut cpu = new_cpu();
    cpu.a = 0x35;
    cpu.y = 0x11;
    cpu.p.set(cpu::flags::Flag::C);
    load_data(&mut cpu.memory, 0xBEEF, &[0x23]);
    let cycles = run_program(&mut cpu, &[0xF9, 0xDE, 0xBE]);
    assert_eq!(cpu.a, 0x12);
    assert_eq!(cycles, 4);
}

#[test]
fn test_sbc_indexed_indirect() {
    let mut cpu = new_cpu();
    cpu.a = 0x35;
    cpu.x = 0x11;
    cpu.p.set(cpu::flags::Flag::C);
    load_data(&mut cpu.memory, 0x0098, &[0xEF, 0xBE]);
    load_data(&mut cpu.memory, 0xBEEF, &[0x23]);
    let cycles = run_program(&mut cpu, &[0xE1, 0x87]);
    assert_eq!(cpu.a, 0x12);
    assert_eq!(cycles, 6);
}

#[test]
fn test_sbc_indirect_indexed() {
    let mut cpu = new_cpu();
    cpu.a = 0x35;
    cpu.y = 0x11;
    cpu.p.set(cpu::flags::Flag::C);
    load_data(&mut cpu.memory, 0x0087, &[0xDE, 0xBE]);
    load_data(&mut cpu.memory, 0xBEEF, &[0x23]);
    let cycles = run_program(&mut cpu, &[0xF1, 0x87]);
    assert_eq!(cpu.a, 0x12);
    assert_eq!(cycles, 5);
}

#[test]
fn test_sbc_immediate_bcd() {
    let mut cpu = new_cpu();
    // 39 - 23 = 16
    cpu.a = 0b0011_1001;
    cpu.p.set(cpu::flags::Flag::C);
    cpu.p.set(cpu::flags::Flag::D);
    run_program(&mut cpu, &[0xE9, 0b0010_0011]);
    assert_eq!(cpu.a, 0b0001_0110);
}

#[test]
fn test_sbc_immediate_bcd_with_borrow() {
    let mut cpu = new_cpu();
    // 39 - 23 - 1= 15
    cpu.a = 0b0011_1001;
    cpu.p.clear(cpu::flags::Flag::C);
    cpu.p.set(cpu::flags::Flag::D);
    run_program(&mut cpu, &[0xE9, 0b0010_0011]);
    assert_eq!(cpu.a, 0b0001_0101);
}

#[test]
fn test_sbc_immediate_bcd_causing_borrow() {
    let mut cpu = new_cpu();
    // 19 - 23 = -4 (96 + borrow)
    cpu.a = 0b0001_1001;
    cpu.p.set(cpu::flags::Flag::C);
    cpu.p.set(cpu::flags::Flag::D);
    run_program(&mut cpu, &[0xE9, 0b0010_0011]);
    assert_eq!(cpu.a, 0b1001_0110);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
}
