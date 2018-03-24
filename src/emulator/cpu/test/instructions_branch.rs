use emulator::cpu;

use emulator::cpu::test::load_data;
use emulator::cpu::test::load_program;
use emulator::cpu::test::new_cpu;
use emulator::cpu::test::run_instructions;
use emulator::cpu::test::run_program;

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
    load_program(&mut cpu, &[0x50, 0x08]);

    let pc_before = cpu.pc;
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, pc_before + 2 + 8);
    assert_eq!(cycles, 3)
}

#[test]
fn test_bvs_branch_positive() {
    let mut cpu = new_cpu();
    cpu.p.set(cpu::flags::Flag::V);
    load_program(&mut cpu, &[0x70, 0x08]);

    let pc_before = cpu.pc;
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, pc_before + 2 + 8);
    assert_eq!(cycles, 3)
}

#[test]
fn test_cmp_immediate_lt() {
    let mut cpu = new_cpu();
    cpu.a = 0x15;
    let cycles = run_program(&mut cpu, &[0xC9, 0x25]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 2);
}

#[test]
fn test_cmp_immediate_lt_overflow() {
    let mut cpu = new_cpu();
    cpu.a = 0x15;
    let cycles = run_program(&mut cpu, &[0xC9, 0xFF]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 2);
}

#[test]
fn test_cmp_immediate_eq() {
    let mut cpu = new_cpu();
    cpu.a = 0x15;
    let cycles = run_program(&mut cpu, &[0xC9, 0x15]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 2);
}

#[test]
fn test_cmp_immediate_gt() {
    let mut cpu = new_cpu();
    cpu.a = 0x15;
    let cycles = run_program(&mut cpu, &[0xC9, 0x05]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 2);

}

#[test]
fn test_cmp_immediate_gt_overflow() {
    let mut cpu = new_cpu();
    cpu.a = 0xFF;
    let cycles = run_program(&mut cpu, &[0xC9, 0x05]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 2);
}

#[test]
fn test_cmp_zero_page_lt() {
    let mut cpu = new_cpu();
    cpu.a = 0x15;
    load_data(&mut cpu.memory, 0x00AB, &[0x25]);
    let cycles = run_program(&mut cpu, &[0xC5, 0xAB]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 3);
}

#[test]
fn test_cmp_zero_page_indexed_lt() {
    let mut cpu = new_cpu();
    cpu.a = 0x15;
    cpu.x = 0x10;
    load_data(&mut cpu.memory, 0x00AB, &[0x25]);
    let cycles = run_program(&mut cpu, &[0xD5, 0x9B]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 4);
}

#[test]
fn test_cmp_absolute_lt() {
    let mut cpu = new_cpu();
    cpu.a = 0x15;
    load_data(&mut cpu.memory, 0xBEEF, &[0x25]);
    let cycles = run_program(&mut cpu, &[0xCD, 0xEF, 0xBE]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 4);
}

#[test]
fn test_cmp_absolute_x_lt() {
    let mut cpu = new_cpu();
    cpu.a = 0x15;
    cpu.x = 0x10;
    load_data(&mut cpu.memory, 0xBEEF, &[0x25]);
    let cycles = run_program(&mut cpu, &[0xDD, 0xDF, 0xBE]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 4);
}

#[test]
fn test_cmp_absolute_y_lt() {
    let mut cpu = new_cpu();
    cpu.a = 0x15;
    cpu.y = 0x10;
    load_data(&mut cpu.memory, 0xBEEF, &[0x25]);
    let cycles = run_program(&mut cpu, &[0xD9, 0xDF, 0xBE]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 4);
}

#[test]
fn test_cmp_indexed_indirect() {
    let mut cpu = new_cpu();
    cpu.a = 0x15;
    cpu.x = 0x10;
    load_data(&mut cpu.memory, 0x0046, &[0xEF, 0xBE]);
    load_data(&mut cpu.memory, 0xBEEF, &[0x25]);
    let cycles = run_program(&mut cpu, &[0xC1, 0x36]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 6);
}

#[test]
fn test_cmp_indirect_indexed() {
    let mut cpu = new_cpu();
    cpu.a = 0x15;
    cpu.y = 0x10;
    load_data(&mut cpu.memory, 0x0046, &[0xDF, 0xBE]);
    load_data(&mut cpu.memory, 0xBEEF, &[0x25]);
    let cycles = run_program(&mut cpu, &[0xD1, 0x46]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::N), true);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::Z), false);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::V), false);
    assert_eq!(cycles, 5);
}

