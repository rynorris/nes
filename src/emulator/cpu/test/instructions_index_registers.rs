use emulator::cpu;

use emulator::cpu::test::load_data;
use emulator::cpu::test::new_cpu;
use emulator::cpu::test::run_program;

#[test]
fn test_ldx_immediate() {
    let mut cpu = new_cpu();
    let cycles = run_program(&mut cpu, &[0xA2, 0xDE]);
    assert_eq!(cpu.x, 0xDE);
    assert_eq!(cycles, 2);
}

#[test]
fn test_ldx_immediate_sets_zero_flag() {
    let mut cpu = new_cpu();
    let cycles = run_program(&mut cpu, &[0xA2, 0x00]);
    assert_eq!(cpu.x, 0x00);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_ldx_immediate_sets_negative_flag() {
    let mut cpu = new_cpu();
    let cycles = run_program(&mut cpu, &[0xA2, 0xFF]);
    assert_eq!(cpu.x, 0xFF);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_ldx_zero_page() {
    let mut cpu = new_cpu();
    load_data(&mut cpu.memory, 0x0024, &[0xDE]);
    let cycles = run_program(&mut cpu, &[0xA6, 0x24]);
    assert_eq!(cpu.x, 0xDE);
    assert_eq!(cycles, 3);
}

#[test]
fn test_ldx_zero_page_y() {
    let mut cpu = new_cpu();
    cpu.y = 0x10;
    load_data(&mut cpu.memory, 0x0034, &[0xDE]);
    let cycles = run_program(&mut cpu, &[0xB6, 0x24]);
    assert_eq!(cpu.x, 0xDE);
    assert_eq!(cycles, 4);
}

#[test]
fn test_ldx_absolute() {
    let mut cpu = new_cpu();
    load_data(&mut cpu.memory, 0xBEEF, &[0xDE]);
    let cycles = run_program(&mut cpu, &[0xAE, 0xEF, 0xBE]);
    assert_eq!(cpu.x, 0xDE);
    assert_eq!(cycles, 4);
}

#[test]
fn test_ldx_absolute_y() {
    let mut cpu = new_cpu();
    cpu.y = 0x10;
    load_data(&mut cpu.memory, 0xBEEF, &[0xDE]);
    let cycles = run_program(&mut cpu, &[0xBE, 0xDF, 0xBE]);
    assert_eq!(cpu.x, 0xDE);
    assert_eq!(cycles, 4);
}

#[test]
fn test_ldy_immediate() {
    let mut cpu = new_cpu();
    let cycles = run_program(&mut cpu, &[0xA0, 0xDE]);
    assert_eq!(cpu.y, 0xDE);
    assert_eq!(cycles, 2);
}

#[test]
fn test_ldy_immediate_sets_zero_flag() {
    let mut cpu = new_cpu();
    let cycles = run_program(&mut cpu, &[0xA0, 0x00]);
    assert_eq!(cpu.y, 0x00);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_ldy_immediate_sets_negative_flag() {
    let mut cpu = new_cpu();
    let cycles = run_program(&mut cpu, &[0xA0, 0xFF]);
    assert_eq!(cpu.y, 0xFF);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_ldy_zero_page() {
    let mut cpu = new_cpu();
    load_data(&mut cpu.memory, 0x0024, &[0xDE]);
    let cycles = run_program(&mut cpu, &[0xA4, 0x24]);
    assert_eq!(cpu.y, 0xDE);
    assert_eq!(cycles, 3);
}

#[test]
fn test_ldy_zero_page_x() {
    let mut cpu = new_cpu();
    cpu.x = 0x10;
    load_data(&mut cpu.memory, 0x0034, &[0xDE]);
    let cycles = run_program(&mut cpu, &[0xB4, 0x24]);
    assert_eq!(cpu.y, 0xDE);
    assert_eq!(cycles, 4);
}

#[test]
fn test_ldy_absolute() {
    let mut cpu = new_cpu();
    load_data(&mut cpu.memory, 0xBEEF, &[0xDE]);
    let cycles = run_program(&mut cpu, &[0xAC, 0xEF, 0xBE]);
    assert_eq!(cpu.y, 0xDE);
    assert_eq!(cycles, 4);
}

#[test]
fn test_ldy_absolute_x() {
    let mut cpu = new_cpu();
    cpu.x = 0x10;
    load_data(&mut cpu.memory, 0xBEEF, &[0xDE]);
    let cycles = run_program(&mut cpu, &[0xBC, 0xDF, 0xBE]);
    assert_eq!(cpu.y, 0xDE);
    assert_eq!(cycles, 4);
}

#[test]
fn test_stx_zero_page() {
    let mut cpu = new_cpu();
    cpu.x = 0x34;
    let cycles = run_program(&mut cpu, &[0x86, 0x67]);
    assert_eq!(cpu.x, 0x34);
    assert_eq!(cpu.memory.load(0x0067), 0x34);
    assert_eq!(cycles, 3);
}

#[test]
fn test_stx_zero_page_y() {
    let mut cpu = new_cpu();
    cpu.x = 0x34;
    cpu.y = 0x08;
    let cycles = run_program(&mut cpu, &[0x96, 0x67]);
    assert_eq!(cpu.x, 0x34);
    assert_eq!(cpu.memory.load(0x006F), 0x34);
    assert_eq!(cycles, 4);
}

#[test]
fn test_stx_absolute() {
    let mut cpu = new_cpu();
    cpu.x = 0x34;
    let cycles = run_program(&mut cpu, &[0x8E, 0x67, 0x45]);
    assert_eq!(cpu.x, 0x34);
    assert_eq!(cpu.memory.load(0x4567), 0x34);
    assert_eq!(cycles, 4);
}

#[test]
fn test_sty_zero_page() {
    let mut cpu = new_cpu();
    cpu.y = 0x34;
    let cycles = run_program(&mut cpu, &[0x84, 0x67]);
    assert_eq!(cpu.y, 0x34);
    assert_eq!(cpu.memory.load(0x0067), 0x34);
    assert_eq!(cycles, 3);
}

#[test]
fn test_sty_zero_page_y() {
    let mut cpu = new_cpu();
    cpu.y = 0x34;
    cpu.x = 0x08;
    let cycles = run_program(&mut cpu, &[0x94, 0x67]);
    assert_eq!(cpu.y, 0x34);
    assert_eq!(cpu.memory.load(0x006F), 0x34);
    assert_eq!(cycles, 4);
}

#[test]
fn test_sty_absolute() {
    let mut cpu = new_cpu();
    cpu.y = 0x34;
    let cycles = run_program(&mut cpu, &[0x8C, 0x67, 0x45]);
    assert_eq!(cpu.y, 0x34);
    assert_eq!(cpu.memory.load(0x4567), 0x34);
    assert_eq!(cycles, 4);
}

#[test]
fn test_inx_no_wrap() {
    let mut cpu = new_cpu();
    cpu.x = 0x34;
    let cycles = run_program(&mut cpu, &[0xE8]);
    assert_eq!(cpu.x, 0x35);
    assert_eq!(cycles, 2);
}

#[test]
fn test_inx_wrap() {
    let mut cpu = new_cpu();
    cpu.x = 0xFF;
    let cycles = run_program(&mut cpu, &[0xE8]);
    assert_eq!(cpu.x, 0x00);
    assert_eq!(cycles, 2);
}

#[test]
fn test_iny_no_wrap() {
    let mut cpu = new_cpu();
    cpu.y = 0x34;
    let cycles = run_program(&mut cpu, &[0xC8]);
    assert_eq!(cpu.y, 0x35);
    assert_eq!(cycles, 2);
}

#[test]
fn test_iny_wrap() {
    let mut cpu = new_cpu();
    cpu.y = 0xFF;
    let cycles = run_program(&mut cpu, &[0xC8]);
    assert_eq!(cpu.y, 0x00);
    assert_eq!(cycles, 2);
}

#[test]
fn test_dex_no_wrap() {
    let mut cpu = new_cpu();
    cpu.x = 0x34;
    let cycles = run_program(&mut cpu, &[0xCA]);
    assert_eq!(cpu.x, 0x33);
    assert_eq!(cycles, 2);
}

#[test]
fn test_dex_wrap() {
    let mut cpu = new_cpu();
    cpu.x = 0x00;
    let cycles = run_program(&mut cpu, &[0xCA]);
    assert_eq!(cpu.x, 0xFF);
    assert_eq!(cycles, 2);
}

#[test]
fn test_dey_no_wrap() {
    let mut cpu = new_cpu();
    cpu.y = 0x34;
    let cycles = run_program(&mut cpu, &[0x88]);
    assert_eq!(cpu.y, 0x33);
    assert_eq!(cycles, 2);
}

#[test]
fn test_dey_wrap() {
    let mut cpu = new_cpu();
    cpu.y = 0x00;
    let cycles = run_program(&mut cpu, &[0x88]);
    assert_eq!(cpu.y, 0xFF);
    assert_eq!(cycles, 2);
}

#[test]
fn test_cpx_immediate_lt() {
    let mut cpu = new_cpu();
    cpu.x = 0x15;
    let cycles = run_program(&mut cpu, &[0xE0, 0x25]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 2);
}

#[test]
fn test_cpx_zero_page_lt() {
    let mut cpu = new_cpu();
    cpu.x = 0x15;
    load_data(&mut cpu.memory, 0x00AB, &[0x25]);
    let cycles = run_program(&mut cpu, &[0xE4, 0xAB]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 3);
}

#[test]
fn test_cpx_absolute_lt() {
    let mut cpu = new_cpu();
    cpu.x = 0x15;
    load_data(&mut cpu.memory, 0xBEEF, &[0x25]);
    let cycles = run_program(&mut cpu, &[0xEC, 0xEF, 0xBE]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 4);
}

#[test]
fn test_cpy_immediate_lt() {
    let mut cpu = new_cpu();
    cpu.y = 0x15;
    let cycles = run_program(&mut cpu, &[0xC0, 0x25]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 2);
}

#[test]
fn test_cpy_zero_page_lt() {
    let mut cpu = new_cpu();
    cpu.y = 0x15;
    load_data(&mut cpu.memory, 0x00AB, &[0x25]);
    let cycles = run_program(&mut cpu, &[0xC4, 0xAB]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 3);
}

#[test]
fn test_cpy_absolute_lt() {
    let mut cpu = new_cpu();
    cpu.y = 0x15;
    load_data(&mut cpu.memory, 0xBEEF, &[0x25]);
    let cycles = run_program(&mut cpu, &[0xCC, 0xEF, 0xBE]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 4);
}

