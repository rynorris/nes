use emulator::cpu;

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

#[test]
fn test_bcc_no_branch() {
    let mut cpu = new_cpu();
    cpu.p.set(cpu::flags::Flag::C);
    load_program(&mut cpu, &[0x90, 0xFF]);

    let pc_before = cpu.pc;
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, pc_before + 2);
    assert_eq!(cycles, 2);
}

#[test]
fn test_bcc_branch_positive() {
    let mut cpu = new_cpu();
    cpu.p.clear(cpu::flags::Flag::C);
    load_program(&mut cpu, &[0x90, 0x08]);

    let pc_before = cpu.pc;
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, pc_before + 2 + 8);
    assert_eq!(cycles, 3);
}

#[test]
fn test_bcc_branch_negative_page_boundary() {
    let mut cpu = new_cpu();
    cpu.p.clear(cpu::flags::Flag::C);
    load_program(&mut cpu, &[0x90, 0b1111_1100]); // -4

    let pc_before = cpu.pc;
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, pc_before + 2 - 4);
    assert_eq!(cycles, 4);
}

#[test]
fn test_bcs_branch_positive() {
    let mut cpu = new_cpu();
    cpu.p.set(cpu::flags::Flag::C);
    load_program(&mut cpu, &[0xB0, 0x08]);

    let pc_before = cpu.pc;
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, pc_before + 2 + 8);
    assert_eq!(cycles, 3);
}

#[test]
fn test_beq_branch_positive() {
    let mut cpu = new_cpu();
    cpu.p.set(cpu::flags::Flag::Z);
    load_program(&mut cpu, &[0xF0, 0x08]);

    let pc_before = cpu.pc;
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, pc_before + 2 + 8);
    assert_eq!(cycles, 3);
}

#[test]
fn test_bmi_branch_positive() {
    let mut cpu = new_cpu();
    cpu.p.set(cpu::flags::Flag::N);
    load_program(&mut cpu, &[0x30, 0x08]);

    let pc_before = cpu.pc;
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, pc_before + 2 + 8);
    assert_eq!(cycles, 3)
}

#[test]
fn test_bne_branch_positive() {
    let mut cpu = new_cpu();
    cpu.p.clear(cpu::flags::Flag::Z);
    load_program(&mut cpu, &[0xD0, 0x08]);

    let pc_before = cpu.pc;
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, pc_before + 2 + 8);
    assert_eq!(cycles, 3)
}

#[test]
fn test_bpl_branch_positive() {
    let mut cpu = new_cpu();
    cpu.p.clear(cpu::flags::Flag::N);
    load_program(&mut cpu, &[0x10, 0x08]);

    let pc_before = cpu.pc;
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, pc_before + 2 + 8);
    assert_eq!(cycles, 3)
}

#[test]
fn test_bvc_branch_positive() {
    let mut cpu = new_cpu();
    cpu.p.clear(cpu::flags::Flag::V);
    load_program(&mut cpu, &[0x10, 0x08]);

    let pc_before = cpu.pc;
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, pc_before + 2 + 8);
    assert_eq!(cycles, 3)
}

#[test]
fn test_bvs_branch_positive() {
    let mut cpu = new_cpu();
    cpu.p.set(cpu::flags::Flag::V);
    load_program(&mut cpu, &[0x10, 0x08]);

    let pc_before = cpu.pc;
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, pc_before + 2 + 8);
    assert_eq!(cycles, 3)
}

