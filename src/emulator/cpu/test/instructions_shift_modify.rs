use emulator::cpu;

use emulator::cpu::test::new_cpu;
use emulator::cpu::test::run_program;

#[test]
fn test_lsr_accumulator_no_carry() {
    let mut cpu = new_cpu();
    cpu.a = 0b0001_0010;
    let cycles = run_program(&mut cpu, &[0x4A]);
    assert_eq!(cpu.a, 0b0000_1001);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cycles, 2);
}

#[test]
fn test_lsr_accumulator_carry() {
    let mut cpu = new_cpu();
    cpu.a = 0b1000_0001;
    let cycles = run_program(&mut cpu, &[0x4A]);
    assert_eq!(cpu.a, 0b0100_0000);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_lsr_accumulator_sets_zero_flag() {
    let mut cpu = new_cpu();
    cpu.a = 0b0000_0001;
    let cycles = run_program(&mut cpu, &[0x4A]);
    assert_eq!(cpu.a, 0b0000_0000);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_lsr_accumulator_clears_negative_flag() {
    let mut cpu = new_cpu();
    cpu.a = 0b1001_0010;
    cpu.p.set(cpu::flags::Flag::N);
    let cycles = run_program(&mut cpu, &[0x4A]);
    assert_eq!(cpu.a, 0b0100_1001);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), false);
    assert_eq!(cycles, 2);
}

#[test]
fn test_lsr_zero_page() {
    let mut cpu = new_cpu();
    cpu.store_memory(0x0034, 0b0001_0011);
    let cycles = run_program(&mut cpu, &[0x46, 0x34]);
    assert_eq!(cpu.load_memory(0x0034), 0b0000_1001);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 5);
}

#[test]
fn test_lsr_zero_page_x() {
    let mut cpu = new_cpu();
    cpu.store_memory(0x0034, 0b0001_0011);
    cpu.x = 0x10;
    let cycles = run_program(&mut cpu, &[0x56, 0x24]);
    assert_eq!(cpu.load_memory(0x0034), 0b0000_1001);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 6);
}

#[test]
fn test_lsr_absolute() {
    let mut cpu = new_cpu();
    cpu.store_memory(0xBEEF, 0b0001_0011);
    let cycles = run_program(&mut cpu, &[0x4E, 0xEF, 0xBE]);
    assert_eq!(cpu.load_memory(0xBEEF), 0b0000_1001);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 6);
}

#[test]
fn test_lsr_absolute_x() {
    let mut cpu = new_cpu();
    cpu.store_memory(0xBEEF, 0b0001_0011);
    cpu.x = 0x10;
    let cycles = run_program(&mut cpu, &[0x5E, 0xDF, 0xBE]);
    assert_eq!(cpu.load_memory(0xBEEF), 0b0000_1001);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 7);
}

#[test]
fn test_asl_accumulator_no_carry() {
    let mut cpu = new_cpu();
    cpu.a = 0b0001_0010;
    let cycles = run_program(&mut cpu, &[0x0A]);
    assert_eq!(cpu.a, 0b0010_0100);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cycles, 2);
}

#[test]
fn test_asl_accumulator_carry() {
    let mut cpu = new_cpu();
    cpu.a = 0b1000_0001;
    let cycles = run_program(&mut cpu, &[0x0A]);
    assert_eq!(cpu.a, 0b0000_0010);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_asl_accumulator_sets_zero_flag() {
    let mut cpu = new_cpu();
    cpu.a = 0b1000_0000;
    let cycles = run_program(&mut cpu, &[0x0A]);
    assert_eq!(cpu.a, 0b0000_0000);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_asl_accumulator_sets_negative_flag() {
    let mut cpu = new_cpu();
    cpu.a = 0b0101_0010;
    let cycles = run_program(&mut cpu, &[0x0A]);
    assert_eq!(cpu.a, 0b1010_0100);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_asl_zero_page() {
    let mut cpu = new_cpu();
    cpu.store_memory(0x0034, 0b1001_0010);
    let cycles = run_program(&mut cpu, &[0x06, 0x34]);
    assert_eq!(cpu.load_memory(0x0034), 0b0010_0100);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 5);
}

#[test]
fn test_asl_zero_page_x() {
    let mut cpu = new_cpu();
    cpu.store_memory(0x0034, 0b1001_0010);
    cpu.x = 0x10;
    let cycles = run_program(&mut cpu, &[0x16, 0x24]);
    assert_eq!(cpu.load_memory(0x0034), 0b0010_0100);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 6);
}

#[test]
fn test_asl_absolute() {
    let mut cpu = new_cpu();
    cpu.store_memory(0xBEEF, 0b1001_0010);
    let cycles = run_program(&mut cpu, &[0x0E, 0xEF, 0xBE]);
    assert_eq!(cpu.load_memory(0xBEEF), 0b0010_0100);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 6);
}

#[test]
fn test_asl_absolute_x() {
    let mut cpu = new_cpu();
    cpu.store_memory(0xBEEF, 0b1001_0010);
    cpu.x = 0x10;
    let cycles = run_program(&mut cpu, &[0x1E, 0xDF, 0xBE]);
    assert_eq!(cpu.load_memory(0xBEEF), 0b0010_0100);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 7);
}

#[test]
fn test_ror_accumulator_no_carry() {
    let mut cpu = new_cpu();
    cpu.a = 0b0001_0010;
    let cycles = run_program(&mut cpu, &[0x6A]);
    assert_eq!(cpu.a, 0b0000_1001);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cycles, 2);
}

#[test]
fn test_ror_accumulator_carry() {
    let mut cpu = new_cpu();
    cpu.a = 0b1000_0001;
    let cycles = run_program(&mut cpu, &[0x6A]);
    assert_eq!(cpu.a, 0b0100_0000);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_ror_accumulator_sets_zero_flag() {
    let mut cpu = new_cpu();
    cpu.a = 0b0000_0001;
    let cycles = run_program(&mut cpu, &[0x6A]);
    assert_eq!(cpu.a, 0b0000_0000);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_ror_accumulator_sets_negative_flag() {
    let mut cpu = new_cpu();
    cpu.a = 0b1001_0010;
    cpu.p.set(cpu::flags::Flag::C);
    let cycles = run_program(&mut cpu, &[0x6A]);
    assert_eq!(cpu.a, 0b1100_1001);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cycles, 2);
}

#[test]
fn test_ror_zero_page() {
    let mut cpu = new_cpu();
    cpu.store_memory(0x0034, 0b0001_0011);
    let cycles = run_program(&mut cpu, &[0x66, 0x34]);
    assert_eq!(cpu.load_memory(0x0034), 0b0000_1001);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 5);
}

#[test]
fn test_ror_zero_page_x() {
    let mut cpu = new_cpu();
    cpu.store_memory(0x0034, 0b0001_0011);
    cpu.x = 0x10;
    let cycles = run_program(&mut cpu, &[0x76, 0x24]);
    assert_eq!(cpu.load_memory(0x0034), 0b0000_1001);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 6);
}

#[test]
fn test_ror_absolute() {
    let mut cpu = new_cpu();
    cpu.store_memory(0xBEEF, 0b0001_0011);
    let cycles = run_program(&mut cpu, &[0x6E, 0xEF, 0xBE]);
    assert_eq!(cpu.load_memory(0xBEEF), 0b0000_1001);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 6);
}

#[test]
fn test_ror_absolute_x() {
    let mut cpu = new_cpu();
    cpu.store_memory(0xBEEF, 0b0001_0011);
    cpu.x = 0x10;
    let cycles = run_program(&mut cpu, &[0x7E, 0xDF, 0xBE]);
    assert_eq!(cpu.load_memory(0xBEEF), 0b0000_1001);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 7);
}

#[test]
fn test_rol_accumulator_no_carry() {
    let mut cpu = new_cpu();
    cpu.a = 0b0001_0010;
    let cycles = run_program(&mut cpu, &[0x2A]);
    assert_eq!(cpu.a, 0b0010_0100);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cycles, 2);
}

#[test]
fn test_rol_accumulator_carry() {
    let mut cpu = new_cpu();
    cpu.a = 0b1000_0001;
    let cycles = run_program(&mut cpu, &[0x2A]);
    assert_eq!(cpu.a, 0b0000_0010);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_rol_accumulator_sets_zero_flag() {
    let mut cpu = new_cpu();
    cpu.a = 0b1000_0000;
    let cycles = run_program(&mut cpu, &[0x2A]);
    assert_eq!(cpu.a, 0b0000_0000);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_rol_accumulator_sets_negative_flag() {
    let mut cpu = new_cpu();
    cpu.a = 0b0101_0000;
    let cycles = run_program(&mut cpu, &[0x2A]);
    assert_eq!(cpu.a, 0b1010_0000);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cycles, 2);
}

#[test]
fn test_rol_accumulator_incoming_carry() {
    let mut cpu = new_cpu();
    cpu.a = 0b0000_0000;
    cpu.p.set(cpu::flags::Flag::C);
    let cycles = run_program(&mut cpu, &[0x2A]);
    assert_eq!(cpu.a, 0b0000_0001);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cycles, 2);
}

#[test]
fn test_rol_zero_page() {
    let mut cpu = new_cpu();
    cpu.store_memory(0x0034, 0b1001_0010);
    let cycles = run_program(&mut cpu, &[0x26, 0x34]);
    assert_eq!(cpu.load_memory(0x0034), 0b0010_0100);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 5);
}

#[test]
fn test_rol_zero_page_x() {
    let mut cpu = new_cpu();
    cpu.store_memory(0x0034, 0b1001_0010);
    cpu.x = 0x10;
    let cycles = run_program(&mut cpu, &[0x36, 0x24]);
    assert_eq!(cpu.load_memory(0x0034), 0b0010_0100);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 6);
}

#[test]
fn test_rol_absolute() {
    let mut cpu = new_cpu();
    cpu.store_memory(0xBEEF, 0b1001_0010);
    let cycles = run_program(&mut cpu, &[0x2E, 0xEF, 0xBE]);
    assert_eq!(cpu.load_memory(0xBEEF), 0b0010_0100);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 6);
}

#[test]
fn test_rol_absolute_x() {
    let mut cpu = new_cpu();
    cpu.store_memory(0xBEEF, 0b1001_0010);
    cpu.x = 0x10;
    let cycles = run_program(&mut cpu, &[0x3E, 0xDF, 0xBE]);
    assert_eq!(cpu.load_memory(0xBEEF), 0b0010_0100);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 7);
}

